---
tipo: mapa
actualizado: 2026-10-15
---

# Mapa del proyecto

La página de inicio: dónde estamos, qué sigue, qué espera tu respuesta y dónde está cada cosa. **Claude la actualiza al cerrar cada bloque.**

> [!info] Hoy (2026-10-15)
> **C4 empezó. Fase 1 hecha: el Capturista tiene su batería** ([ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)):
> - 12 casos de lo que una persona cuenta en el chat, con trampas: sumas, edades supuestas, cifras con palabras, datos protegidos;
> - el código ahora revisa que una cifra esté en tus palabras: «20 camas arriba y 15 abajo» ya no se vuelve «35»;
> - lo que dice el Capturista después de llenar ya lo escribe el programa: qué dejó, qué no tomó y qué falta contar. Antes la IA podía prometer algo que el código había tirado.
>
> Con el modelo más barato: 68 de 69 casos bien, nada inventado, unos $0.02 MXN por caso.
>
> **Siguiente:** fase 2 de C4, las tres preguntas del Asistente que salen bien solo 1 de 2 veces.
>
> **Antes (2026-10-14): arreglos tras tu prueba de C3** ([ADR-039](adr/ADR-039-mapa-de-lugares-desde-el-manual.md)):
> - el botón «Ir a…» ya no se escribe a mano: sale de un mapa armado con el manual y los formularios. Una página nueva solo necesita su entrada en el manual;
> - el respaldo desde «Mi institución» lleva a **Seguridad**, y nunca se ofrece un botón a la pantalla donde ya estás;
> - «con base en ese mismo» ahora le pregunta al Lector por el documento de la respuesta anterior, en vez de contestar de memoria (1 de 2 rondas; la otra ofrece «Llenar con mis documentos»).
>
> Batería de 14 preguntas con el modelo más barato, dos rondas: 25 de 28, nada inventado, $1.14 MXN.
>
> **Antes (2026-10-13): C3 está hecho: el Asistente le pregunta al Lector del Expediente** ([ADR-036, precisiones C3](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)):
> - el Asistente ya no lee documentos: le pregunta al Lector, que contesta con la frase exacta, y el programa muestra el documento y la página;
> - cada pregunta lleva el mapa del Expediente: qué documentos hay y cómo están (nunca qué dicen);
> - la IA ve el tipo del documento, nunca el nombre del archivo (tu decisión), también en C2;
> - «Llenar con mis documentos» también aparece bajo una respuesta del Asistente, como botón.
>
> Con el modelo más barato: **26 de 26 preguntas bien en dos rondas**, nada inventado, unos $0.04 MXN por pregunta. Nuestro protocolo JSON le ganó a las funciones nativas de Gemini (26 contra 25 bien, 35 % más barato): se queda.
>
> **Siguiente:** que lo pruebes (pasos abajo); después, **C4**: la revisión completa del núcleo.

## Qué es SociAI, en una línea

Un programa de escritorio, local y con IA, para que asilos y casas hogar ordenen sus datos y armen proyectos para pedir donativos. Lo usan directivas y religiosas sin perfil técnico. Ver [Visión](00-vision.md).

## Qué sigue

|     | Bloque                | Qué hace                                                                                                                                                                                                                                                                       | Estado                |
| --- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------- |
| 1   | **C1**                | Cada documento va a un solo cajón; Documentos es la bandeja que pregunta a qué cajón va (por ahora solo el Expediente); la búsqueda siempre dentro de un cajón                                                                                                                 | **Hecho, por probar** |
| —   | **Lectura y llenado** | ADR-035, 037 y 038: una sola política de llenado, datos propios solos, Pdfium para texto y tablas, OCR (letra chica, hojas giradas, páginas mixtas, imágenes en Word y Excel, prioridad baja), lectura en segundo plano, el Expediente tapa y no rechaza, avisos de contraseña | **Hecho, por probar** |
| 2   | **C2**                | El Lector del Expediente: llena todo «Mi institución» buscando en el Expediente campo por campo, con la cita de cada dato; botón «Llenar con mis documentos»                                                                                                                   | **Hecho, por probar** |
| 3   | **C3**                | El Asistente pregunta al Lector del Expediente y sabe qué documentos existen («según mi registro, ¿qué nombre legal pongo?»)                                                                                                                                                   | **Hecho y probado**   |
| 4   | **C4**                | Revisión completa del núcleo: cada agente aprobado con el modelo más barato y cada pantalla probada por ti. Fase 1 (Capturista) hecha; siguen el Asistente, la corrida final, el guion por pantalla y la metodología de agentes                                               | **En curso**          |
| 5   | **N5**                | El expediente hacia Proyectos, la IA e Inicio                                                                                                                                                                                                                                  | Pendiente             |
| —   | Módulos               | Finanzas (facturas), Personal y Beneficiarios (Lector solo con reglas), Instalaciones, Donantes: cada uno con su cajón, su Lector y su batería, siguiendo la [metodología de los Lectores](agents/metodologia-lectores.md)                                                     | Esperan al núcleo     |

Detalle de cada bloque en el [Roadmap](09-roadmap.md), «Fase 10».

## Lo que espera tu respuesta

- **Probar el Capturista:** en «Mi institución», abre el Asistente y pulsa «Llenarlo por mí» en «Capacidad y cifras rápidas». Escribe «Tenemos 20 camas arriba y 15 abajo y nos ayudan 6 voluntarios».
  - Debe dejar solo los voluntarios y decir que lo que habría tenido que calcular no lo dejó.
  - Debe decir qué más puedes contarle.
- Borrar el archivo suelto `E:\Git\search_live3.txt` (lo dejó un error mío; el sistema no me deja borrarlo).
- Confirmar el precio de `gemini-embedding-2` en la página oficial de Google (anotado 0.20 USD por millón de tokens, tomado de sitios que comparan precios).

## Cómo probar C3 · el Asistente le pregunta al Lector

1. Sube al Expediente varios documentos con texto: acta, constancia, poder, reglamento… y al menos una foto o un escaneo.
2. En **Mi institución**, abre el Asistente y pregunta **«Según mi registro, ¿qué nombre legal pongo?»**.
   - Responde con el nombre legal y abajo dice «Lo tomé de sus documentos: ‹archivo›, página N».
   - Ofrece «Ir a Datos legales y fiscales» y, si sirve, **«Llenar con mis documentos»**.
3. Pregunta **«¿Qué documentos tengo?»**: dice por su nombre cuáles tienes y cuáles faltan, y ofrece ir al Expediente.
4. **Nombra un archivo:** «¿Qué dice ‹tu archivo›.pdf?», y también con el nombre de la foto o del escaneo.
   - Contesta con lo que dice ese documento.
   - Nunca dice que no puede ver imágenes.
5. Pregunta algo que **ningún documento dice**: debe decir que no lo encontró, sin inventar.
6. Pide **«Llena los datos legales con mis documentos»** y pulsa el botón «Llenar con mis documentos».
   - El panel dice cuántas sugerencias dejó, con «Revisarlas».
   - Nada se guarda sin «Usar» y «Guardar».
7. Pregunta **«¿Cómo hago un respaldo?»**: responde con el manual y muestra su fuente.

Lo que no debe pasar: que diga el nombre de un archivo como si lo hubiera leído la IA (el nombre solo aparece en la línea «Lo tomé de sus documentos», y lo pone el programa) o que cite un documento que no habla del tema.

**Pruebas:**
- `cargo test core::agents::assistant` y `cargo test core::reader::ask`;
- la corrida real `cargo test assistant_battery_live -- --ignored --nocapture > assistant_battery_live.txt` (unos $0.50 MXN por ronda).

## Cómo probar C2 · «Llenar con mis documentos»

1. Sube al Expediente varios documentos con texto: acta, constancia, informe anual, reglamento…
2. **Mi institución → Expediente → «Llenar todo con mis documentos».**
   - Mientras trabaja dice «Buscando en sus documentos…».
   - Al terminar, un aviso dice cuántas sugerencias dejó y en cuántas ventanas.
3. Abre cada ventana.
   - Cada sugerencia dice «Lo leímos en «documento», página N».
   - «Usar» la pone en su campo; nada se guarda hasta «Guardar».
4. **Revisa que no invente.**
   - Ningún campo que ya tenías escrito recibe sugerencia.
   - Una cifra sugerida está escrita tal cual en el documento.
   - Lo que ningún documento dice queda vacío.
5. **El botón de una ventana:** «Llenar con mis documentos».
   - Con todo escrito, avisa «Todos los campos que puedo llenar en esta ventana ya están escritos» y no gasta nada.
   - Sin documentos, avisa que primero hay que subirlos.
6. Descarta una sugerencia y vuelve a pulsar el botón: no vuelve a salir.
7. El teléfono, el correo, el domicilio, el RFC, los folios y el representante nunca salen de este botón: los leen las reglas al subir el archivo.

**Pruebas:**
- `cargo test core::reader::fill`;
- la corrida real `cargo test reader_live -- --ignored --nocapture > reader_live.txt` (4 llamadas, unos $0.14 MXN).

## Cómo probar la lectura y el llenado (ADR-035, 037 y 038)

**Antes:** corre una vez `powershell -ExecutionPolicy Bypass -File scripts/fetch-ocr.ps1`, que ahora también baja el modelo que endereza las hojas giradas.

**La ventana del propio documento se llena sola**
1. **Mi institución → Expediente → Poder del representante legal → Subir archivo**, con tu poder (o el acta, la constancia, el oficio de donataria, la 32-D).
   - La ventana dice «Leyendo… Puede cerrar esta ventana»: ciérrala.
   - La fila dice «Leyendo: N de M páginas».
   - Al terminar, un aviso dice cómo fue.
2. Vuelve a abrir el poder.
   - Las fechas de emisión y de vigencia y el número de escritura están llenos.
   - Debajo de cada uno dice «Leído del archivo, página N».
   - En una 32-D, la vigencia dice «Calculado con la fecha de emisión del archivo».
3. Corrige a mano la vigencia, guarda y sube otra vez el archivo: la vigencia que escribiste no cambia; la fecha de emisión y el número sí se leen del archivo nuevo.

**«Mi institución» solo recibe sugerencias en campos vacíos**
4. En «Datos legales y fiscales», escribe a mano el nombre legal y guarda: su sugerencia desaparece.
   - Sube otro documento que traiga el nombre: no vuelve a sugerirlo.
5. Descarta una sugerencia y vuelve a subir el mismo archivo: no vuelve.

**Archivos que no van o no se pueden abrir**
6. Sube la constancia en el lugar del acta.
   - Debe guardarse y avisar «Este archivo parece «Constancia de situación fiscal», no este documento».
   - No llena nada.
7. Un PDF con contraseña (`D:\Agente Proyectos\pdf_pruebas\protected.pdf`) avisa «tiene contraseña», también en **Mis proyectos → nueva convocatoria**.

**Escaneos y fotos**
8. Un acta escaneada larga: la computadora no se congela mientras se lee.
9. Una hoja escaneada de lado se lee bien, y una página con encabezado impreso y cuerpo escaneado también.
10. Un Word con una hoja escaneada pegada: su texto aparece en Documentos.

**Pruebas:** `cargo test core::reader documents:: core::records scanner` y `pnpm test`.

## Cómo probar C1

1. Empieza con una base vacía o vuelve a subir tus documentos: los de antes de C1 no tienen cajón y no aparecen.
2. **Documentos → «Subir documento»:** debe preguntar «¿A qué cajón va?» (solo Expediente) y «¿Qué documento es?» (los del Expediente y «Otro documento»). Al seguir, se abre la misma ventana del Expediente de «Mi institución»; ahí pones el archivo.
3. El documento aparece en Documentos con el nombre de su documento del Expediente, agrupado por clase. El lápiz vuelve a abrir su ventana.
4. Sube el mismo documento desde «Mi institución» → Expediente: debe verse igual en Documentos.
5. Reemplaza el archivo de un documento: en Documentos debe quedar solo el nuevo.
6. «Buscar en el expediente» encuentra por nombre y por dentro.
7. Acepta una sugerencia que venga de un documento y luego bórralo con el bote de basura: debe borrarse sin error y el dato aceptado se queda.

## Lo último que se hizo

| Fecha | Qué | Dónde leerlo |
|---|---|---|
| 2026-10-15 | **C4, fase 1 · batería del Capturista:** 12 casos con trampas en las cuatro ventanas; el código revisa que cada cifra esté en las palabras de la persona; la IA ya no escribe la nota, el programa dice qué dejó, qué no tomó y qué falta contar. 68 de 69 casos bien, nada inventado | [ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-14 | **El mapa de lugares y «ese mismo»:** los botones «Ir a…» salen de un mapa armado con el manual y los formularios (sin listas a mano); el código elige el botón y nunca ofrece la pantalla donde ya estás; el respaldo lleva a Seguridad; la pregunta que sigue a un documento va al Lector con la clave de ese documento; el mapa del Expediente trae sus cifras para que la IA no cuente. Batería de 14: 25 de 28 en dos rondas, nada inventado | [ADR-039](adr/ADR-039-mapa-de-lugares-desde-el-manual.md) |
| 2026-10-13 | **C3 · el Asistente como orquestador:** le pregunta al Lector del Expediente (`record_reader`), que contesta con su cita revisada por el código; el mapa del Expediente en cada pregunta; la IA ve el tipo del documento, nunca el nombre del archivo; «Llenar con mis documentos» bajo una respuesta. Batería de 13 preguntas: 26 de 26 en dos rondas con el modelo más barato, $0.04 MXN por pregunta. Se queda el protocolo JSON (las funciones nativas de Gemini: 25 de 26 y 52 % más caras) | [ADR-036, precisiones C3](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **C2 · «Llenar con mis documentos»:** el Lector busca en el Expediente los pasajes de cada campo vacío y hace una llamada por ventana con el modelo más barato; el código revisa cita, página, cifra y valor. 18 de 18 campos bien, nada inventado, $0.14 MXN las cuatro ventanas. Las opciones van a la IA con su nombre en la pantalla | [ADR-036, precisiones C2](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **Lectura y llenado sin huecos (ADR-038):** una sola política de llenado (datos propios solos, sugerencias solo en campos vacíos, nunca lo rechazado, autoridad por campo, lugar equivocado); Pdfium lee también las tablas (salen `pdf-extract` y `lopdf`); OCR con prioridad baja, hojas giradas, páginas mixtas e imágenes en Word y Excel; el Expediente lee en segundo plano; la metodología para los módulos | [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md), [metodología](agents/metodologia-lectores.md) |
| 2026-10-13 | **Pdfium lee el texto de todo PDF:** palabras enteras (antes «Thi s report»), párrafos por la distancia entre renglones, guion de corte; de 5 a 20 veces más rápido. Un PDF, Word o Excel con contraseña lo dice en el Expediente y en las convocatorias | [ADR-037, bloque 2](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **Pdfium contra `pdf-extract` (medición):** en 24 PDF de muestra sacan las mismas letras, pero `pdf-extract` parte palabras («Thi s report outli nes») y es de 5 a 20 veces más lento; se decide Pdfium como único motor | [ADR-037](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **El Expediente tapa, no rechaza:** ningún archivo del Expediente se rechaza por «lista de personas» ni pide «Tapar y seguir»; el escáner tapa todo, la pantalla dice cuántos datos y la bitácora guarda los conteos. Las convocatorias conservan el rechazo | [ADR-036, precisión del 2026-10-13](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **OCR · letra chica:** una imagen con letra de menos de 20 píxeles se agranda hasta que mida 30 (a lo más ×4) y se lee otra vez; una imagen chica sin ninguna palabra, ×3. Una captura ficticia pasa de 4 errores y sin título a leerse completa con su correo; una foto chica, de cero palabras a completa | [ADR-035, precisión 1](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-13 | **OCR con documentos reales (solo medición):** agrandar la letra chica lleva una captura de 13 palabras mal leídas a 2; dos actas reales se rechazan como «lista de personas» por dos reglas toscas del escáner; 37 páginas tardan 171 s en un equipo de 4 núcleos; no se entrena Tesseract ni se usa la nube | [ADR-035, precisiones 2026-10-13](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-12 | **C1:** cada documento lleva su cajón; Documentos es la bandeja (pregunta qué documento del Expediente es y abre su ventana); la búsqueda siempre en un cajón; quitar o reemplazar un archivo saca su texto del cajón; «Borrar esto y todo…» ya no falla con un dato aceptado | [ADR-036, precisiones C1](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **ADR-036:** cajones con dueño, un agente por cajón, el Asistente como orquestador, cada agente validado con el modelo más barato; primero el núcleo | [ADR-036](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **IA6a con documentos reales:** el Lector entiende un acta real de 14 páginas y la ficha de la Junta (nombre, figura, año, objeto, número de institución); el objeto nunca incluye nombres del patronato | [Roadmap, IA6a](09-roadmap.md) |
| 2026-10-12 | **IA6a:** la clase y el tipo de cada documento; sus datos llegan como sugerencias con documento y página; un dato protegido solo por reglas | [ADR-034, precisiones IA6a](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-12 | **IA5:** búsqueda dentro de los documentos por palabras y por significado (vectores); medido antes de activarla: de 50 % a 100 % de aciertos con preguntas dichas con otras palabras | [ADR-034, precisiones IA5](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-10 | **OCR:** leer páginas escaneadas y fotos | [ADR-035](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-10 | **N4:** el expediente de la institución con vigencia | [ADR-033 §5](adr/ADR-033-nucleo-expediente-y-formularios-universales.md) |
| 2026-10-10 | **IA3–IA4:** el Asistente en toda la app y el Capturista («Llenarlo por mí») | [ADR-034](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |

## Cómo está armado, en un dibujo

```
            Documentos  (la bandeja: aquí se sube y se ve todo)      ← ADR-036
                 │  «¿A qué cajón va?»
   ┌─────────────┼──────────────┬──────────────┬──────────────┐
Expediente     Personal       Finanzas       Proyectos     Beneficiarios,
(núcleo)       (módulo)       (módulo)       (convocatorias) Instalaciones…
 └ su Lector    └ solo reglas  └ su Lector    └ lectura canónica
 └ llena «Mi institución»

Asistente: la puerta de entrada en toda la app; pregunta al agente del cajón que corresponde.
Capas (ADR-032): base → módulos → núcleo → proyectos → pantallas; cada capa solo mira hacia abajo.
```

## Dónde está cada cosa

| Para saber… | Lee |
|---|---|
| Qué es y para quién | [Visión](00-vision.md) |
| Cómo está construido (capas, núcleo y módulos) | [Arquitectura](01-arquitectura.md) · [ADR-032](adr/ADR-032-monolito-modular-nucleo-y-modulos.md) |
| Qué datos nunca salen y cómo se cuidan | [Gobernanza de datos](03-gobernanza-datos.md) · [Escáner](04-escaner-datos-sensibles.md) |
| Qué tablas hay y de quién es cada una | [Modelo de datos](05-modelo-datos.md) |
| Cómo usamos la IA y cuánto cuesta | [IA y costos](07-ia-y-costos.md) |
| Cómo se escriben los textos de la pantalla | [Estilo de redacción](08-estilo-redaccion.md) |
| Qué se ha hecho y qué falta, bloque por bloque | [Roadmap](09-roadmap.md) |
| Cómo se ve el programa | [Sistema visual](13-sistema-visual.md) |
| Qué dice la ayuda del programa | [Manual](manual/programa.md) · [Mi institución](manual/mi-institucion.md) |
| Las reglas para trabajar en el código | [Principios](agents/principios-de-ingenieria.md) · [Flujo de trabajo](agents/flujo-de-trabajo.md) · [Comandos y pruebas](agents/comandos-y-pruebas.md) |
| Cómo un cajón lee sus archivos y llena sus formularios (para cada módulo) | [Metodología de los Lectores](agents/metodologia-lectores.md) · [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md) |
| Por qué se decidió cada cosa | Los ADR, abajo · [Índice de ADR](adr/README.md) |

## Tus observaciones

Cómo dejarlas: [Notas y observaciones](notas/README.md). Una nota por observación, con la plantilla `plantillas/observacion`.

Abiertas (necesita el complemento Dataview):

```dataview
TABLE WITHOUT ID file.link AS "Nota", sobre AS "Sobre", fecha AS "Fecha"
FROM "notas"
WHERE estado = "abierta"
SORT fecha DESC
```

Observaciones dentro de los ADR: búsquelas con la búsqueda de Obsidian (Ctrl + Shift + F) escribiendo `"[!nota]" path:adr`.

## Las decisiones (ADR)

Las vigentes, de la más nueva a la más vieja:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha", tema AS "Tema", reemplaza_en_parte AS "Reemplaza en parte"
FROM "adr"
WHERE adr AND estado != "reemplazada"
SORT adr DESC
```

Las que ya no rigen, o que otra cambió en parte:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", reemplazado_por AS "La reemplaza", reemplazado_en_parte_por AS "La cambia en parte"
FROM "adr"
WHERE adr AND (estado = "reemplazada" OR reemplazado_en_parte_por)
SORT adr DESC
```

Por tema (para ver solo lo de IA, del núcleo o de documentos):

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha"
FROM "adr"
WHERE contains(tema, "ia")
SORT adr DESC
```

Cambie `"ia"` por `"nucleo"`, `"documentos"`, `"proyectos"`, `"modulos"`, `"seguridad"`, `"interfaz"`, `"arquitectura"`, `"infraestructura"` o `"datos"`.
