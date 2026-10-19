---
adr: 39
estado: aceptada
fecha: 2026-10-14
tema: [ia, nucleo, pantalla]
reemplaza_en_parte: ["[[ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento]]"]
tags: [adr]
---
# ADR-039 · El mapa de lugares sale del manual y de los formularios, no de listas escritas a mano

**Estado:** aceptada (2026-10-14, a pedido de la institución tras probar C3). Aplicada en el Asistente.

## Contexto

Al probar C3, la institución preguntó «¿Cómo hago un respaldo?» desde «Mi institución». La respuesta decía «vaya a Seguridad», pero el botón decía «Ir a Mi institución» y no llevaba a ningún lado. Tenía tres causas:

1. **Seguridad no estaba en la lista de lugares del Asistente** (`GO_TO`). El modelo no podía elegirla y eligió la más cercana.
2. **El botón llevaba a la pantalla donde ya estaba la persona.** Nada lo impedía, así que el botón no hacía nada.
3. **Los lugares estaban escritos a mano en seis sitios**, que se desfasaban: `GO_TO` y `SCREENS` en `assistant.rs`, el `match` de `place_of_field`, la lista del oficio, `es.assistant.places` en `es-MX.ts` y el tipo `Page` de `Shell.tsx`. Seguridad, Ayuda automática y Administración estaban en la barra, pero no en la lista del Asistente.

La institución pidió una metodología por defecto: que el índice de lugares se arme solo, a partir del enlace que ya tiene cada botón o campo. Así, agregar una página, un formulario o un campo no obliga a anotarlo en otra lista.

## Decisión

### 1. Un solo mapa de lugares (`core/places.rs`), armado con lo que ya existe

- **Una página es una pantalla del manual** (`screen.<página>` en `docs/manual/`). Su nombre es el título de esa entrada. `screen.institution` pasa a llamarse `screen.profile`, como la página.
- **Una ventana de «Mi institución» es la lista de formularios que contiene** (`WINDOWS`). Su nombre es el título de su primer formulario en el manual. Es lo único que se escribe en el mapa, y una prueba falla si un formulario del catálogo no tiene ventana.
- **Cada entrada del manual sabe dónde se hace lo que explica:**
  - un campo va a la ventana de su formulario;
  - un documento del Expediente (`record.*`) va al Expediente;
  - una duda de uso va al lugar que nombra su encabezado con `@` (`# Cambiar de computadora o hacer un respaldo {#howto.backup @screen.security}`).
- **Quién puede abrir cada página** (ADR-028) es una tabla corta (`NEEDS`). Una prueba la compara con lo que esconde la barra.

Del mapa salen:

- la lista de lugares del oficio del Asistente (el marcador `{places}` del oficio v8);
- el esquema de su respuesta;
- la pantalla y el campo que van en su contexto;
- el nombre de cada botón «Ir a…», que la pantalla pide con `app_places`.

`es.assistant.places` desaparece.

### 2. El botón lo elige el código (`places::choose`)

- Nunca se ofrece un lugar que la persona no puede abrir, ni la página donde ya está. Una ventana de esa página sí se ofrece, porque se abre.
- **Manda la entrada del manual en que se basó la respuesta:**
  - si la IA eligió ese lugar, o uno dentro de él, se respeta;
  - si eligió la página que lo contiene, el botón va a la ventana;
  - si eligió otro lugar, el botón sigue a la entrada.
- Si la respuesta no se basó en una entrada con lugar, va el lugar que eligió la IA.

### 3. La conversación dice de qué documentos salió cada respuesta

Cada vuelta del Asistente guarda en la pantalla los documentos que citó (sus id). En la siguiente pregunta, el contexto dice «(lo tomó de D3 Registro ante la Junta)»: la clave y el tipo, nunca el nombre del archivo. El oficio v8 dice que «ese mismo» se le pregunta al Lector con la clave de ese documento, aunque la persona pregunte qué poner en una ventana. También pide no nombrar los botones en el texto: el programa los pone debajo, cada uno con su nombre.

## Cómo se agrega un lugar (la metodología por defecto)

| Para agregar… | Se hace… | Las pruebas revisan |
|---|---|---|
| Una página | su entrada `screen.<página>` en `docs/manual/` y la página en `Shell.tsx` | que las páginas de la barra y las del manual sean las mismas, y quién puede abrirlas |
| Una ventana de «Mi institución» | su línea en `WINDOWS` y que `ProfilePage` la abra | que cada formulario esté en una sola ventana y que la pantalla la abra |
| Un campo o un formulario | nada más: su lugar sale del catálogo | que cada campo tenga ventana |
| Una duda de uso con lugar | `@screen.…` (o `@` un formulario o un campo) en su encabezado del manual | que el `@` lleve a un lugar |

## Consecuencias

- El Asistente ya puede llevar a Seguridad, Ayuda automática (solo a quien puede abrirla) y Administración (solo a la persona administradora).
- Un botón nunca lleva a la pantalla donde ya está la persona.
- La regla también corrige un texto viejo: el botón «Ir a «Inteligencia artificial»» decía un nombre que la barra ya no usa. Ahora dice «Ayuda automática».
- Queda escrita a mano la tabla `FILLABLE` del Capturista y del Lector: qué ventanas puede llenar la IA, que es una decisión y no un enlace. Una prueba revisa que sus ventanas y nombres sean los del mapa.
