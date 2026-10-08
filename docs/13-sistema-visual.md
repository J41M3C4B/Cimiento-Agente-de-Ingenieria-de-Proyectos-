# 13 · Sistema visual

**Estado:** vigente (ADR-025). Aprobado tras validar el prototipo con la institución. Es el contrato visual de Cimiento: `src/index.css` guarda los tokens, `src/components/ui/` los componentes, y la prueba `src/design.test.ts` impide saltarse las reglas. Cualquier cambio de interfaz empieza aquí (ver §14).

Este documento recoge la dirección visual tomada de las referencias que la institución eligió (un tablero de tareas en bandejas blancas sobre un marco gris, con color sólido y tranquilo) y la traduce a reglas que se puedan aplicar a Cimiento. Es una especificación de diseño: no cambia flujos, datos ni textos (`docs/08-estilo-redaccion.md` sigue mandando).

## 1. Idea

Cimiento acompaña a personas que dan asistencia social, con poco tiempo y poca experiencia con programas. La interfaz debe sentirse **amable, ordenada y viva**, no como una herramienta de oficina.

Tres ideas sostienen todo lo demás:

1. **Color con significado.** Cada cosa que importa tiene su color sólido y lo conserva en toda la app. El color ayuda a reconocer dónde se está y qué sigue; las monjas que probaron las referencias lo señalaron como lo que más les facilita el uso. Nunca es el único dato: siempre va con palabras.
2. **Bandejas sobre el fondo.** No hay una hoja ni un marco entre la ventana y el contenido (parecería una aplicación dentro de otra): cada tema es una bandeja blanca, grande, con mucho aire, directamente sobre el fondo de la ventana. Una pantalla es un tablero de bandejas, no una página larga.
3. **Una cosa a la vez.** Cada bandeja responde una pregunta. Lo que se puede hacer ahora es lo más grande y lo más oscuro de la pantalla.

Corolario para Inicio y para cualquier pantalla de entrada: **mostrar menos**. Lo que se hace con el asistente se hace en su pantalla, no se duplica en miniatura; lo que es mantenimiento (la ficha de la institución) se resume en una línea y un enlace; y cada dato aparece una sola vez.

## 2. Superficies

Tres niveles, de atrás hacia adelante; **no hay un nivel de «hoja» o «marco» entre el fondo y las bandejas**. La profundidad la da el cambio de tono y una sombra muy suave, no los bordes.

| Nivel | Qué es | Claro | Oscuro | Radio |
|---|---|---|---|---|
| 0 · Fondo | La ventana misma (token `canvas`) | `#F2F2F4` | `#16171B` | — |
| 1 · Bandeja | Cada módulo | `#FFFFFF` | `#1F2025` | 28 px |
| 2 · Interior | Elementos dentro de una bandeja (burbujas, calendario, campos) | `#F6F6F8` | `#26272D` | 18–20 px |

Sombras:

- Bandeja: `0 1px 1px rgb(20 20 30 / .03), 0 10px 30px -12px rgb(20 20 30 / .12)`.
- Flotante (menús, ventanas): `0 24px 60px -16px rgb(20 20 30 / .3)`.
- Las bandejas con pestaña usan `filter: drop-shadow(...)` en lugar de `box-shadow`, para que la sombra siga la forma.

Líneas: solo cuando hace falta separar filas, 1 px `#E6E6EA` (oscuro `#2E3037`).

## 3. Color

### 3.1 Neutros

| Token | Claro | Oscuro | Uso |
|---|---|---|---|
| `ink` | `#121216` | `#F3F3F5` | Texto principal, botón principal, casillas marcadas |
| `ink-2` | `#55555E` | `#B4B4BD` | Texto secundario |
| `ink-3` | `#8C8C96` | `#82828C` | Etiquetas, ayudas, celdas vacías |
| `line` | `#E6E6EA` | `#2E3037` | Separadores |

Texto de cuerpo: `ink` sobre bandeja (≥ 15:1). `ink-3` solo para etiquetas de 12–13 px, nunca para datos.

### 3.2 Acentos sólidos

Ocho colores **frescos y vivos, idénticos en modo claro y oscuro**, y **la misma letra negra** (`#121216`) sobre todos, en cualquier tema: etiquetas, avatares, íconos de color, pestañas de carpeta, píldoras seleccionadas y avisos. Una sola regla, sin excepciones por color ni por tema. Concuerda además con el botón negro principal de la interfaz.

Todos pasan 7:1 con la letra negra, y permiten un amarillo verdadero. (Se evaluó una paleta «profunda» con letra blanca; se descartó porque obligaba a oscurecer los colores, el amarillo se volvía cobre y el naranja resultante se sentía pesado junto a la vibra fresca del resto.)

| Token | Valor | Contraste con la letra | Significa en Cimiento |
|---|---|---|---|
| `sky` | `#6FA0FF` | 7.3:1 | Enlaces y selección; color de proyecto |
| `violet` | `#A592FF` | 7.3:1 | Personas que atendemos; color de proyecto |
| `rose` | `#FF86B8` | 8.3:1 | Color de proyecto |
| `red` | `#FF7D73` | 7.5:1 | Error, borrar, bloquear |
| `amber` | `#FFD43D` | 13.1:1 | Falta algo, atención («Regular»); fecha de entrega; nómina |
| `green` | `#4ED08A` | 9.5:1 | Listo, confirmado, correcto |
| `teal` | `#3DD2C2` | 10.0:1 | Personal |
| `cyan` | `#4FCBF2` | 9.9:1 | Fuentes de ingreso; color de proyecto |

Sobre fondo blanco los colores claros rinden menos como línea fina (el amarillo es 1.4:1 contra blanco); por eso las barras de avance dejan lo que falta en un tinte del mismo color, y los estados siempre llevan palabra. Sobre superficies oscuras todos pasan de 6:1.

### 3.2.1 Azul de marca

La herramienta se llama **SociAI** y su azul es el del logotipo: `brand` (`#0a76fc`), con letra blanca encima (`on-brand`). Es **solo de identidad** y siempre **liso**: sin degradados, brillos ni sombras. Se usa en el panel izquierdo del primer inicio (`.onb`, pantalla dividida con los formularios a la derecha), donde el logo va todo en blanco (`<Logo onBrand>`). No es un acento de estado ni significa nada (§3.4). En ese panel la «AI» del logo va en `#000c25` y el isotipo (`public/SVG/isotipo.svg`) se usa como imagen grande en un azul más claro, liso. El programa abre siempre en tema claro; el oscuro solo se aplica si la persona lo elige.

### 3.3 Etiquetas sólidas y tintes

Las etiquetas de estado y de categoría son **sólidas**, del mismo color vivo que el resto de la interfaz; un fondo pastel con texto oscuro se ve apagado junto a los avatares, íconos y bandejas de color. La letra de encima es siempre negra (3.2): una sola regla para etiquetas, avatares, íconos de color, pestañas de carpeta, píldoras seleccionadas y avisos.

El **tinte** (acento al 18 % sobre la bandeja, texto en el acento mezclado al 58 % con tinta) queda solo para datos informativos de bajo énfasis, como «Gastos de inversión». Si algo es estado, es sólido.

### 3.4 Reglas de uso

- **El color nombra.** Un color pertenece a un significado y no se reutiliza para otro. La tabla de 3.2 es la lista completa.
- **Un proyecto, un color.** Cada proyecto conserva el color que su dueña le puso (ya existe en la base, `project.color`) y **todo lo suyo va en ese color**: la pestaña de la carpeta, la etiqueta del paso, las barras de avance, el recuadro «Lo siguiente» y los puntos de avance de la cápsula superior. No se mezclan colores dentro de una carpeta: mezclarlos pesa, cuesta leer de qué proyecto se trata y rompe la organización. Los **pasos no tienen color propio**: se distinguen por su nombre y por cuántos están llenos.
- **Las personas** (personal, beneficiarias) se muestran con iniciales sobre un color sólido de la serie `sky, violet, teal, cyan, rose, green, amber`, siempre el mismo para el mismo nombre. Solo en este equipo; no sale en documentos ni a la IA (`docs/03-gobernanza-datos.md`).
- **Estados siempre con texto:** «Listo», «Por confirmar», «Mal estado». Etiqueta sólida, no solo punto.
- **Un solo color grande por carpeta.** El resto de la pantalla es blanco y neutro. Varias carpetas de colores distintos conviven porque cada una es de un solo color.
- **Rojo solo para lo destructivo o lo que falló.**

## 4. Tipografía

Una sola familia: **Plus Jakarta Sans** (variable, 400–800), de formas geométricas y abiertas, legible en pantallas pequeñas y con buen soporte de acentos. Respaldo: `"Segoe UI Variable", system-ui, sans-serif`. Cifras con `font-variant-numeric: tabular-nums` cuando se alinean.

| Rol | Tamaño / interlínea | Peso | Uso |
|---|---|---|---|
| Cifra grande | 72 / 1 | 400 | Fecha del calendario, total, pendientes |
| Título de página | 28 / 1.15 | 700 | Nombre de la institución, del proyecto |
| Título de bandeja | 18 / 1.25 | 700 | «Mi institución», «Asistente» |
| Cuerpo | 15 / 1.6 | 500 | Chat, textos |
| Interfaz | 14 / 1.45 | 600 | Botones, filas, valores |
| Etiqueta | 12.5 / 1.3 | 600 | Encabezados de tabla, ayudas |

Mínimo 14 px para todo lo que se lee o se pulsa; 12.5 solo para etiquetas. Pesos: 400, 500, 600, 700, 800. El texto base va en 500 (no en 400) para que se lea con firmeza en pantallas pequeñas y a distancia; las cifras grandes, en 400; los títulos, en 700; los avatares, en 800.

## 5. Espacio, radios y tamaños

- Escala de espacio (px): 4 · 8 · 12 · 16 · 24 · 32 · 48. Entre bandejas: 16. Relleno interior de una bandeja: 24 (20 en las pequeñas).
- Radios: bandeja 28 · interior 18–20 · campo 14 · etiqueta y botón de ícono: círculo completo (999).
- Altura de controles: 40 (botón), 44 (botón circular de ícono), 36 (etiqueta y campo compacto). Botón principal de una bandeja: 44.
- Íconos: trazo de 1.8 px (los actuales de `icons.tsx`), 18–20 px.

### 5.1 Tokens: cómo se llaman en el código

Los valores de §2–§5 viven en **un solo lugar**, `src/index.css`, en tres niveles. Se usan por su nombre; nunca se escribe un valor suelto (`#fff`, `13px`, `rounded-[20px]`) en una pantalla.

| Nivel | Qué es | Dónde | Ejemplos |
|---|---|---|---|
| 1 · Primitivos | Los ocho acentos y la letra sobre ellos; no cambian con el tema | `@theme` | `sky violet rose red amber green teal cyan`, `onc` |
| 2 · Semánticos | Superficies, tinta y líneas; cambian con el tema claro/oscuro | `:root` y `[data-theme]` (variables), expuestos en `@theme inline` | `canvas card inset line ink ink-2 ink-3 on-ink` |
| 3 · De componente | Medidas y formas que un componente comparte con todos los demás | `@theme` (radios, alturas, tipografía, sombras) y clases de componente | `rounded-field`, `h-field`, `text-ui`, `shadow-card`, `.btn`, `.field`, `.tag` |

Nombres de las superficies: **`card`** es la bandeja blanca (nivel 2 de §2), **`inset`** el interior (nivel 3).

| Familia | Tokens (clase de Tailwind) | Valor |
|---|---|---|
| Texto de estado | `text-sky-ink` · `text-red-ink` · `text-amber-ink` · `text-green-ink` | el acento al 45 % mezclado con `ink`; ≥ 5:1 sobre la bandeja en claro y oscuro. Para palabras sueltas de estado (un error bajo un campo, «Por revisar»); lo demás usa etiquetas sólidas |
| Tipografía | `text-caption` · `text-small` · `text-ui` · `text-body` · `text-heading` · `text-subtitle` · `text-title` · `text-hero` · `text-display` | 12.5 · 13 · 14 · 15 · 18 · 22 · 28 · 32 · 72 px |
| Peso | `font-normal` (cifras) · `font-medium` (base) · `font-semibold` · `font-bold` (títulos, botones, etiquetas) · `font-extrabold` (avatares, pestañas) | 400 · 500 · 600 · 700 · 800 |
| Radios | `rounded-tick` · `rounded-field` · `rounded-inset` · `rounded-card` · `rounded-pill` | 8 · 14 · 20 · 28 · 999 px |
| Alturas | `h-tag` · `h-ctl-sm` · `h-ctl` · `h-field` | 28 · 36 · 44 · 48 px |
| Sombras | `shadow-card` · `shadow-float` | ver §2 |
| Espacio | escala de 4 px de Tailwind, solo 1 · 2 · 3 · 4 · 6 · 8 · 12 (4, 8, 12, 16, 24, 32, 48) | §5 |

Los tamaños, radios, colores y sombras de Tailwind que no están en estas tablas **no existen** (se vaciaron en `@theme`): una clase como `text-sm`, `rounded-xl` o `bg-stone-100` no genera nada.

## 6. Estructura de pantalla

```
┌ Fondo de la ventana ────────────────────────────────────────────┐
│  [logo] [cápsula: dónde estoy · avance]            (☾)  Institución (C) │
│  (⌂)   ┌ bandeja ┐  ┌ bandeja ┐  ┌ bandeja con pestaña ──────┐ │
│  (▣)   │         │  │         │  │                           │ │
│  (♜)   └─────────┘  └─────────┘  └───────────────────────────┘ │
│  (▤)   ┌ bandeja ──────┐  ┌ chat ───────────┐  ┌ cifra ─────┐ │
│  ...   └───────────────┘  └─────────────────┘  └────────────┘ │
│  (⏻)                                                            │
└─────────────────────────────────────────────────────────────────┘
```

- **Ventana:** el contenido ocupa toda la ventana con 16 px de margen y 1440 px como máximo, **directamente sobre el fondo**: no hay marco, hoja ni sombra entre el fondo y las bandejas.
- **Barra superior:** a la izquierda el **logotipo** (`Logo`, 32 px de alto) y una **cápsula** (píldora blanca) que dice dónde se está: proyecto, paso y su avance en seis puntos del color del proyecto. A la derecha, botones circulares con borde de 1 px: avisos, buscar, menú y la institución.
- **Riel izquierdo:** botones circulares de 44 px, en columna: Inicio, Mis proyectos, Mi institución, Documentos; separador; Ayuda automática, Seguridad; al fondo Ayuda y **Bloquear en rojo**. El activo va relleno de `ink`. Con etiqueta al pasar el cursor (tooltip a la derecha), no desplegable.
- **Pasos de una ficha o de un asistente (`StepNav`):** círculos sobre una línea y **una sola palabra** debajo (nunca dos líneas de texto). El círculo lleva el número; verde con una palomita es un paso hecho, negro el paso donde está la persona, y un anillo que se llena en verde dice cuánto del paso está capturado. Un punto ámbar en la esquina avisa que el paso pide revisar algo. La línea entre círculos se pinta de verde mientras los pasos van hechos. En pantallas angostas solo quedan los círculos y, debajo, el nombre completo del paso actual.
- **Respuestas cortas:** una pregunta de sí / no / no sé, o de hasta tres respuestas (En trámite…), se contesta con un solo grupo sencillo (`Segmented`: una pastilla gris con la elegida en blanco), sin puntos ni colores. Las listas de más opciones y las de varias respuestas usan etiquetas (`Choice`).
- **Campos en una fila:** los campos que comparten fila se alinean por arriba; si algo no cabe en una línea, la fila se parte en dos líneas (el nombre completo arriba y los demás debajo) en lugar de cortar el texto. Las tarjetas de un resumen se acomodan en columnas que se llenan hacia abajo, sin dejar huecos por tener alturas distintas.
- **Marco fijo:** la cabecera (logotipo, cápsula y menú de la persona) y el riel no se mueven al desplazar la página: solo se desplaza el contenido. El riel queda por encima del contenido (sus etiquetas nunca quedan debajo de una página) y la cabecera por encima del riel; los avisos y ventanas, por encima de todo. En ventanas angostas (< 860 px) o bajas (< 640 px) ambos vuelven a su lugar normal.
- **Avisos de una ficha:** una ficha que tiene cosas por revisar las dice al abrirse, en un aviso arriba (`IssueSummary`): cada una con sus palabras, el paso donde está y «Ver» para ir allí; en la navegación de pasos, el paso lleva una marca ámbar. Lo que impide guardar va aparte, en rojo.
- **Cuadrícula:** 12 columnas, separación de 16. Las bandejas ocupan 3, 4, 5, 6 o 8 columnas; en pantallas angostas (< 900 px) pasan a una columna y el riel se vuelve una fila horizontal.
- **Bandeja con pestaña:** el título vive en una pestaña recortada en la esquina superior izquierda (esquinas cóncavas); sirve para bandejas con un tema («Mi institución», «Resumen del proyecto»). Las pestañas **de una página** (Mi institución, Administración) no llevan bandeja gris: la pestaña activa es blanca y se une a la bandeja de su contenido con esquinas cóncavas, y al cambiar de pestaña se mueve la unión.
- **Bandeja de proyecto como carpeta:** cada proyecto se muestra en una bandeja con **pestaña de carpeta**: 46 px de alto, **una sola línea** con el título del proyecto, esquinas superiores de 18 px y una esquina cóncava de 18 px a la derecha. Una pestaña más alta (con dos líneas) se ve como una gorra, no como una pestaña. La pestaña lleva **el color del proyecto**, el que su dueña le asignó (`project.color`), con el texto de 3.2. **Quien convoca va dentro del cuerpo, a la derecha de la etiqueta del paso**, en `ink-3`. El título que no cabe se corta con puntos suspensivos y el completo va en el atributo `title`. El **paso** va como etiqueta sólida **en el mismo color del proyecto**; los pasos no tienen color propio. El cuerpo es blanco, con la esquina superior izquierda recta bajo la pestaña, y la sombra va con `drop-shadow` para seguir la forma. En la bandeja del proyecto en curso, **la fecha de cierre va dentro de la pestaña**, a la derecha del título, como una cápsula blanca («Cierra el 15 oct»; en pantallas angostas solo «15 oct»); en las tarjetas resumidas la fecha va en el pie. No hay botón «Continuar» en la pestaña.

## 7. Componentes

- **Botón principal:** fondo `ink`, texto blanco, 44 px, radio 999. Uno por bandeja como máximo. Secundario: blanco con borde de 1 px; terciario: texto.
- **Botón circular de ícono:** 44 px, blanco, borde 1 px `line`; activo en `ink`; peligro en `red` sólido.
- **Cápsula:** píldora blanca de 52 px de alto con estado, fecha y avance.
- **Etiqueta (tag):** fondo sólido del acento, texto según 3.3, radio 999, 28 px de alto, 13 px, peso 700. Variantes: `soft` (tinte) para contexto y `line` (borde de 1.5 px, sin relleno) para datos neutros como «Sin redactar». Siempre con palabras.
- **Casilla:** 22 px, radio 7; marcada = `ink` con palomita blanca; el texto hecho se tacha y baja a `ink-3`.
- **Avatar de iniciales:** círculo de 36 px (28 en filas densas) de color sólido; pila de avatares solapados 10 px con anillo de 2 px del color de la bandeja.
- **Tarjeta de persona:** interior (nivel 3) con avatar, nombre, dato secundario y estado a la derecha.
- **Contenedor de documentos:** carpeta con **pestaña negra** (`ink`, texto `onink`) y el conteo en una cápsula blanca dentro de la pestaña. Dentro, de arriba abajo: una frase que dice qué contiene, una fila de herramientas (búsqueda de 42 px con radio completo y el control segmentado «Agrupar por») y los grupos. Cada grupo es una **subcarpeta**: tiene su **viñeta** (pestaña gris tintada de 40 px con flecha, ícono o avatar de color con iniciales si agrupa por donante, nombre y conteo) y, desplegada, un **borde de 2 px** del mismo tono que agrupa sus documentos (esquina superior izquierda recta, para que la viñeta «nazca» del borde). Las subcarpetas cuelgan de una **línea de tronco** vertical con un ramal corto hacia cada viñeta, sangradas unos 38 px respecto al contenedor para que se lea la estructura. Se **pliegan y despliegan** (solo la primera arranca abierta; con búsqueda se abren todas; el estado se conserva al reagrupar). Al final de cada subcarpeta abierta va el botón **«Subir a «Nombre»»** (contorno discontinuo) y la pista «o arrastre un archivo aquí» (se oculta en pantallas angostas); el botón puede ocupar dos líneas si el nombre es largo. Arrastrar un archivo sobre una subcarpeta la resalta con borde discontinuo y fondo `inset`. Si la búsqueda no encuentra nada, dice «No encontramos documentos con eso.»
- **Ícono de archivo:** cuadro de 44 px (38 en listas compactas), radio 14, con la extensión en negrita de 11 px, de **color según el tipo**: PDF `red`, Word `sky`, Excel y CSV `green`, PowerPoint `amber`, cualquier otro neutro. La letra es la negra de siempre.
- **Fila de documento:** ícono de archivo, nombre con extensión (en negrita, con puntos suspensivos si no cabe), una línea de datos en `ink-3` (tipo · tamaño · año, o tipo · tamaño · «Usado en N convocatorias»), la etiqueta de estado («Leído» en tinte verde; «Leyendo…» en sólido `amber` mientras el asistente lo procesa) y, al pasar el cursor o enfocarla, la acción de quitar. Quitar pide confirmación **en la misma fila** («Quitar» en rojo y «Cancelar»); en pantallas angostas el estado «Leído» se oculta para dejar sitio al nombre.
- **Subida de archivo (campo de formulario):** zona de arrastre con borde discontinuo; al elegir el archivo se transforma en una fila con su ícono de color, el nombre, el tamaño y «pulse para cambiarlo». Los campos pueden depender de otro: el de «Donante» solo aparece si el documento es de un donante. **Subir desde una subcarpeta:** el destino ya se conoce, así que el formulario se titula «Subir a «Nombre»», muestra una ruta («De los donantes › Nacional Monte de Piedad») y solo pide lo que falta: el archivo y, si hace falta, el donante o el tipo (opcional). Los archivos arrastrados entran directo con «Por clasificar» o «Por asignar» como valores por defecto.
- **Tarjeta de proyecto (resumida):** la misma carpeta que la bandeja principal, en pequeño, para saber el estado sin abrir el proyecto. Pestaña con el título (color del proyecto); en el cuerpo, de arriba abajo: etiqueta sólida con el paso en el color del proyecto («Paso 3 · Objetivo», o «Listo» con palomita) y, a la derecha, quien convoca; los seis pasos como barras del color del proyecto (los hechos y el actual sólidos, los que faltan en tinte), un interior con «Lo siguiente» (o «Lo que sigue» si ya está listo) con la acción concreta, y el pie con la fecha de cierre, los días que faltan y el botón («Continuar» o «Abrir la guía»). El botón del proyecto más urgente es el principal; los demás, secundarios. Las pestañas tienen alto fijo (46 px) para que el cuerpo empiece a la misma altura en todas las tarjetas. Ancho mínimo 360 px; se reparten solas en la cuadrícula.
- **Casilla para agregar:** rectángulo con borde discontinuo de 1.5 px, radio 20, con «+»; sustituye a los botones «Agregar» sueltos.
- **Cifra de bandeja:** número a 72 px peso 300 sobre un interior, con su etiqueta arriba y su explicación abajo.
- **Calendario:** interior con día de la semana, mes y la fecha en cifra grande. Se usa para el cierre de la convocatoria.
- **Chat:** mensajes de la persona en burbuja de interior alineada a la derecha con su avatar; los del asistente en burbuja de interior a la izquierda con avatar `ink` «AI». Caja de escribir: interior con campo, botón de micrófono (si existe) y envío circular `ink`.
- **Barra de pasos:** seis puntos o píldoras con el color de su paso; lo hecho lleno, lo actual con anillo, lo que sigue vacío con el color al 25 %.
- **Barra de composición:** segmentos continuos de 8 px de alto con los colores de las fuentes (ingresos, avance).
- **Tabla:** sin cabecera rellena; filas de 52 px con línea fina; la primera columna en peso 700; etiquetas de color para la categoría.
- **Formulario (dentro de una ventana):**
  - Secciones con etiqueta en mayúsculas pequeñas y una línea fina; los campos van en dos columnas (una en pantallas angostas) y los largos ocupan las dos.
  - Campo: 48 px de alto, radio 14, borde de 1.5 px `line`; al enfocarlo, el borde pasa a `ink`, sin aro ni sombra exterior. Etiqueta arriba en peso 700; el asterisco rojo marca lo obligatorio.
  - Selector: mismo campo con flecha a la derecha y «Seleccionar» en `ink-3` mientras está vacío. Dinero: «$» fijo a la izquierda y la unidad («al año») a la derecha; solo acepta dígitos.
  - Interruptor: 48 × 28, verde cuando está activo.
  - Opciones cortas (estado, sí/no): **píldoras seleccionables** con el color del estado; la elegida se rellena de sólido. Opciones con explicación (tipo de institución): **tarjetas** con círculo de selección; la elegida lleva borde `ink` y fondo de interior.
  - Error: el campo se marca con borde `red`, el mensaje va debajo con ícono, en palabras de la persona («Escriba el nombre de la persona.»), y el foco pasa al primer campo con error. El mensaje se quita al escribir.
  - Pie con «Cancelar», «Guardar y agregar otra» (solo al agregar) y «Guardar» o «Guardar cambios» como botón principal. `Esc` o tocar fuera cierran; el foco vuelve al botón que abrió la ventana.
  - Al guardar, un aviso breve abajo («Persona agregada») y las cifras de la ficha se actualizan al momento.
- **Ventana (modal):** bandeja de 28 con título de 18, secciones con etiqueta en mayúsculas pequeñas y botones al pie; fondo atenuado.

### 7.1 Medidas canónicas (un componente, una medida)

Cada fila es **la única medida** de ese componente en toda la interfaz. Si una pantalla necesita algo distinto, se cambia aquí y en el componente, no en la pantalla.

| Componente (código) | Alto | Radio | Borde / sombra | Texto | Notas |
|---|---|---|---|---|---|
| `Button` `md` | 44 | pill | — | `text-ui` 700 | relleno horizontal 24; variantes `primary` (ink), `secondary` (card + borde 1 px `line`), `soft` (inset), `danger` (texto rojo), `destructive` (rojo sólido, solo para lo que no se puede deshacer), `ghost`, `plain` |
| `Button` `sm` | 36 | pill | — | `text-small` 700 | relleno 16; para filas, tablas y tarjetas |
| `IconButton` | 44 (`sm` 36) | círculo | 1 px `line` | — | activo en `ink`; peligro en `red` sólido |
| Campo (`TextInput`, `Select`, `TextArea`, `Search`) | 48 (`TextArea` mínimo 96) | `field` (`Search`: pill) | 1.5 px `line`; foco: borde `ink` (sin aro ni sombra); error: borde `red` | `text-ui` 500, relleno 16, vacío en `ink-3` | etiqueta arriba en `text-ui` 700 con 8 px de separación; ayuda en `text-small` `ink-3` **debajo** del campo (para que los campos de una fila queden alineados aunque unos lleven ayuda y otros no); error en `text-small` 700 con ícono |
| `Tag` | 28 | pill | — | `text-small` 700, relleno 13 | `solid` (acento + `onc`), `soft` (acento al 18 % + texto mezclado), `line` (borde 1.5 px) |
| `Avatar` | 36 (`sm` 28) | círculo | — | 12 (10.5) px 800 | color por nombre |
| `Tile` (cuadro de ícono) | 40 (`sm` 32) | `field` | — | — | acento sólido, ícono `onc` |
| `PageHeader` | — | — | — | título `text-title` 700; línea `text-ui` `ink-2` | en todas las páginas igual: título + una línea + acción principal a la derecha |
| `FormSection` | — | — | línea 1 px `line` | `Eyebrow` | agrupa campos en una ventana; puede llevar un `Tile` pequeño con un ícono antes del título |
| `Card` (bandeja) | — | `card` | `shadow-card` | — | relleno 24 (20 en pequeñas) |
| `Inset` (interior) | — | `inset` | — | — | fondo `inset`, relleno 16–20 |
| `Folder` (carpeta con pestaña) | pestaña 46 | pestaña 18, cuerpo `card` | `drop-shadow` | título `text-body` 800 | color del proyecto en `--pc`/`--pt` |
| `Dock` y `DockTab` (pestañas de página, efecto carpeta) | 52 | 24 (esquinas cóncavas) | — | `text-ui` 700 | la pestaña activa es **blanca y se une a la bandeja de su contenido** (la primera bandeja bajo las pestañas lleva `dock-attach`); al cambiar de pestaña cambia la pestaña unida. No hay ninguna superficie gris detrás: pestaña y bandeja son la misma pieza |
| `Segmented` (control segmentado) | 44 (pista) / 36 (opción) | pill | — | `text-small` 700 | opción activa en `card` con `shadow-card` |
| `StepNav` | círculo 32 + una palabra | pill (círculo) | — | `text-small` 700 + `text-caption` | pasos de un formulario largo, navegables: cada uno muestra su avance real («9 de 12»), «No aplica» si no hay nada que llenar, y se vuelve verde con palomita al completarse; en pantallas angostas solo las barras y el nombre del paso actual |
| `MaskedField` | 48 | `field` | 1.5 px `line` | `text-ui` 700 | valor tapado de un identificador (CURP, RFC…) en la misma caja que cualquier campo, con «Mostrar» y «Cambiar» a la derecha |
| `Check` | 24 | `tick` | 2 px `line` | — | marcada = `ink` |
| `Switch` | 28 × 48 | pill | — | — | activo en `green` |
| `Choice` (píldora seleccionable) | 44 | pill | 1.5 px `line` | `text-ui` 700 | elegida = color sólido |
| `RadioCard` | auto | `inset` | 1.5 px `line`; elegida: `ink` | `text-ui` 700 + `text-small` `ink-3` | |
| `Alert` | auto | `inset` | — | `text-ui` | fondo del estado al 18 %, círculo sólido con ícono |
| `Modal` | — | `card` | `shadow-float` | título `text-subtitle` 700 | anchos 640 / 768 / 896 (`wide`, para fichas largas) / 1024; `fixed` fija la altura para que un flujo de pasos no crezca y se encoja |
| `Table` | filas 52 | — | línea 1 px `line` | `text-ui`; encabezado `text-caption` `ink-3` | primera columna 700 |
| `Logo` / `LogoMark` | por altura (`h-8` en la cabecera, `h-10` en la entrada y la bienvenida) | — | — | el nombre es texto en Plus Jakarta Sans 800 | logotipo de SociAI (S + nombre; `LogoMark` es solo la S). Se pinta con los tokens `--logo-mark`, `--logo-word` y `--logo-ai`, que siguen el tema: sobre fondo claro, S azul y nombre azul marino; sobre fondo oscuro, todo blanco con «AI» azul. `inverse` es para superficies pintadas con `ink` (el lado oscuro del primer inicio), donde los colores se invierten. Los tokens `--logo-*` son solo del logotipo, no son acentos de la interfaz |
| `Tooltip` | auto | pill | — | `text-small` 700 en `on-ink` sobre `ink` | |
| `Toast` | auto | pill | `shadow-float` | `text-ui` 700 | fondo `ink` |

## 8. Movimiento

- Transiciones de 160–220 ms con `ease-out` en color, sombra y posición. Nada rebota.
- Entrada de mensajes: aparecen 8 px más abajo y suben (ya existe `anim-rise`).
- Pestañas y cambios de pantalla: fundido de 150 ms; sin desplazamientos largos.
- Se respeta `prefers-reduced-motion`.

## 9. Accesibilidad y uso real

- Contraste: texto normal ≥ 4.5:1, texto grande y componentes ≥ 3:1. El texto sobre `cyan` y `amber` es `ink`; sobre los demás, blanco.
- Nunca solo color: cada estado lleva palabra.
- Objetivo de toque ≥ 40 px; foco visible: en campos, el borde pasa a `ink`; en botones y demás controles, un contorno limpio de 2 px `ink` con 2 px de separación (nunca resplandores).
- Densidad: se prefiere menos bandejas con más aire. Si una bandeja necesita más de siete filas, se pagina o se abre en ventana.
- Modo claro y oscuro con los mismos tokens; el claro es el principal.

## 10. Aplicación a las pantallas de Cimiento

| Pantalla | Estructura |
|---|---|
| **Inicio** (nueva) | Responde solo dos preguntas: «¿qué hago ahora?» y «¿se me viene un plazo?». Un saludo corto con «Empezar un proyecto nuevo» como botón principal a la derecha; la bandeja principal del proyecto en curso (pestaña de carpeta con el título, en el color del proyecto, y la cápsula con el cierre dentro de la pestaña; en el cuerpo la etiqueta con el paso, también del color del proyecto, y a la derecha quien convoca; los seis pasos con su nombre, todos del color del proyecto, **un único bloque destacado «Lo siguiente»** que contiene **el único botón de continuar de la bandeja**: un círculo negro con una flecha que, al pasar el cursor o enfocarlo, se abre en una píldora con la palabra «Continuar» dentro del mismo botón (nunca un letrero aparte, que se leería como un segundo botón); el bloque entero también es pulsable, dos filas de «Después, en este paso», una línea de lo ya terminado y un enlace de ayuda al asistente); «Fechas clave» con el calendario del cierre; una tarjeta pequeña de la institución (estado de la ficha y una razón para tenerla al día, sin listas ni botones de alta); y «Otros proyectos» con una tarjeta resumida por proyecto (ver «Tarjeta de proyecto» en 7). Sin chat, sin pestañas y sin casillas: el avance lo decide el asistente, no la persona |
| **Mis proyectos** | Todos los proyectos con la misma tarjeta resumida que Inicio (la del proyecto más urgente incluida), ordenados por fecha de cierre; «Empezar un proyecto nuevo» como botón principal en el encabezado |
| **Mi institución** | Una sola bandeja de cabecera en dos mitades: a la izquierda la identidad, sin ícono de perfil, en tres alturas: «Mi institución» como etiqueta pequeña arriba, el nombre (32 px) y la misión centrados verticalmente con mucho aire entre ellos, y la etiqueta de estado con «Editar» pegadas al fondo; a la derecha un grupo de 2×2 con las cifras en interiores (personas `violet` con barra de ocupación, personal `teal`, nómina `amber`, cuotas `green`), cada una con su ícono en cuadro de color, etiqueta, cifra de 30 px y detalle. Debajo, las pestañas con efecto de carpeta (la activa se une a la bandeja de su contenido, sin superficie gris detrás); datos en bandejas con filas; **dinero (ADR-026):** el balance (superávit en verde, déficit en ámbar, «Falta un dato» en neutro, con la barra de ingresos contra egresos y de dónde sale el egreso), los **ingresos por tipo** (fijos y variables, barra con un color por tipo, filas con tipo, al mes si así se escribió y «Del padrón» sin acciones para las cuotas que calcula el programa; las líneas que no suman, atenuadas y con su aviso) y los **egresos** (lista por concepto más la nómina con prestaciones que calcula el programa; para empezar, un «gasto aproximado» que la lista reemplaza); **Personal (ADR-027):** lista con nombre y «N cosas por revisar» en ámbar, puesto como etiqueta de color, modalidad, situación (punto verde si trabaja; etiqueta suave si está de vacaciones, con permiso, incapacidad o ya no trabaja) y una barra de «ficha completa» con su porcentaje; filtro por puesto, botón «Puestos» con una etiqueta ámbar de «N plazas sin cubrir», y la ficha en cuatro pasos navegables (`StepNav`) con identificadores tapados (`MaskedField`); el catálogo de puestos muestra personas contra plazas con una barra; el indicador de nómina muestra sueldos más prestaciones y avisa si a alguien le falta el contrato o el año de ingreso; en Personal, Beneficiarios e Instalaciones, el botón principal «Agregar …» va arriba a la derecha de la barra de herramientas **y** la casilla discontinua al final de la lista (los dos); ingresos con barra de composición; tablas con avatares y etiquetas. Por debajo de 1000 px la bandeja pasa a una columna y por debajo de 520 px el grupo también |
| **Espacio de trabajo (Asistente)** | La misma carpeta de proyecto que en Inicio, a ocho columnas: pestaña con el título y la cápsula de cierre, etiqueta del paso con quien convoca, los seis pasos y el chat (burbujas, respuestas rápidas como píldoras, caja con envío circular negro). A cuatro columnas, un panel con **tres pestañas**: **Proyecto** (objetivo, partes con su estado y el «Siguiente paso» con la barra «3 de 14» y el botón de revisión, que explica por qué está cerrado), **Convocatoria** (resumen y lo más importante) y **Fuentes** (los documentos que el asistente consulta para este proyecto: la convocatoria, los del donante y los de la institución, con un enlace a Documentos para administrarlos). Todo en el color del proyecto; solo los estados usan `amber` o `green` |
| **Documentos** | Dos contenedores de carpeta con pestaña negra, lado a lado: **«De mi institución»** (acta, estados financieros, reportes) y **«De los donantes»** (reglas, indicadores, formatos que casi no cambian y valen para varias convocatorias). Cada uno se **agrupa** a elección con un control segmentado: el de la institución por **tipo** (Legales, Financieros, Programas) o **año**; el de los donantes por **donante** o **tipo** (Reglas, Indicadores, Guías y formatos). Los grupos son subcarpetas con viñeta y borde, plegables, cada una con su botón «Subir a «…»» y arrastre (ver Componentes), y hay búsqueda en vivo. «Subir documento» (botón principal del encabezado) abre un formulario: de quién es (tarjetas), archivo, donante (si aplica, con sugerencias de los ya existentes) y tipo. Tras subirlo aparece como «Leyendo…» y pasa a «Leído» cuando el asistente termina |
| **Primer inicio (ADR-031)** | Todas las pantallas del primer inicio (bienvenida, puesta en marcha, asistente y revisión) son **la misma bandeja, del mismo tamaño** (1080 × 736 px como máximo, más baja si la ventana lo es): a la izquierda un lado oscuro con el nombre del programa, el título, lo que promete (se guarda cada paso, los datos se quedan aquí, usted responde con sus palabras) y dos círculos decorativos; a la derecha, lo que cambia. Arriba de la derecha se queda fijo lo que ayuda a ubicarse (barras de la bienvenida o los pasos del asistente con el encabezado del paso: ícono de color, «Paso N de 7», título y lo que se pide); en medio, el formulario, que se desplaza por dentro si es largo; y abajo, siempre en el mismo lugar, los botones. **Bienvenida:** tres pantallas con un color y un ícono cada una y «Paso N de 3». **Puesta en marcha** (solo administración): «N de 3 pasos» y una fila por cosa (ayuda automática, cuentas, datos) con una etiqueta «Lista» (verde) o «Pendiente» (ámbar). **Revisión:** un aviso que dice si todo está listo o cuántos pasos faltan y cada paso como interior con su etiqueta, sus datos (rótulo arriba, valor abajo, «Sin dato» atenuado) y «Editar». **En Inicio:** si el administrador dejó los datos a la dirección, un aviso ámbar con «Llenarlos ahora»; al terminar, «Siguientes pasos» (personal, personas atendidas, espacios y ayuda automática), una fila por paso pendiente con su ícono y «Ir» —lleva a la pestaña que corresponde—, y «N de M listos» con barras; cada paso desaparece al hacerse y la bandeja entera cuando no queda ninguno. **Cifras aproximadas** (Mi institución): mientras no haya fichas, el indicador muestra «≈ 32» con la etiqueta ámbar «Cifra aproximada» y una línea que dice que se calcula sola al registrar a cada persona |
| **Instalaciones (ADR-030)** | Cuatro vistas con `Segmented` (Espacios, Inmueble, Equipo y Tablero, con conteos que salen del módulo). **Cómo están** se dibuja siempre igual: una barra cortada por estado (`StateBar`) en los colores de los estados —bien `green`, regular `amber`, mal `red`, no sirve negro (`ink`) y lo que no se ha revisado, gris atenuado— con sus palabras debajo (4 bien · 1 regular · 2 sin revisar). Espacios y Equipo abren con el resumen de todo en una barra y siguen con la tabla (cuántos, la barra y las fallas como etiquetas suaves ámbar con «+N»). **Captura de los cuatro estados:** un contador por estado (`Stepper`, con su marca de color, su ayuda, «−», el número y «+»), la barra que se llena mientras se cuentan y «Todos están bien»; lo que no se cuenta queda «sin revisar» y el «+» se apaga cuando ya se contaron todos. **Inmueble:** tres bloques de lectura (inmueble, servicios y seguridad) con «N de M capturados» y «Editar»; los sí/no van como etiqueta suave (sí `green`, no `amber`) y las frecuencias de menos a más grave (nunca `green`, a veces `amber`, seguido `red`); sus ventanas se agrupan por secciones con ícono. **Tablero:** cuatro cifras, «Lo que falta» (tarjetas con «Completar» que lleva a la vista donde se captura), los hallazgos con su marca (`Findings`, el mismo bloque que Beneficiarios), el estado de todos los espacios y de todo el equipo, el estado por tipo y lo que más falla |
| **Beneficiarios (ADR-029)** | Tres vistas con `Segmented` dentro de la pestaña (con conteos): **Personas** (misma lista que Personal: nombre y «N cosas por revisar» en ámbar, grupo como etiqueta de color, edad, situación con punto verde o etiqueta suave, barra de «ficha completa» con su porcentaje, y la casilla «Agregar persona» al final), **Tablero** y **Lista de espera**. La ficha es el mismo formulario fijo y ancho de Personal en cinco pasos con `StepNav` (avance por paso), encabezado con la persona y su avance total, una línea de qué se pide en cada paso e íconos por sección; «No sé la fecha exacta» es un interruptor que cambia la fecha por la edad aproximada; la CURP va tapada (`MaskedField`) con «Mostrar» y «Cambiar»; las opciones de varias respuestas (motivos de ingreso, discapacidad, salud, programas) son etiquetas que se activan; los campos propios de asilo o casa hogar aparecen según el tipo de institución. **Tablero:** cuatro cifras (`Figures`), «Lo que dicen sus datos» en dos columnas de tarjetas, cada hallazgo con ícono y marca (violeta «La ayuda automática también lo sabe» o neutra con candado «Solo para usted»), y debajo la pirámide por edad y sexo (mujeres a la izquierda en `rose`, hombres a la derecha en `sky`, con el conteo junto a cada barra) y barras ordenadas por categoría, cada una de un color propio (apoyo `violet`, movilidad `sky`, discapacidad `teal`, salud `rose`, motivos `amber`, programas `green`, situación legal `cyan`), con cantidad y porcentaje; dos columnas que se llenan alternadas para no dejar huecos. **Lista de espera:** tabla con solicitud (avatar o ícono si no hay nombre, teléfono), fecha y «Lleva N días esperando» (ámbar desde 90 días), datos, motivo y acciones («Darle ingreso», editar; «Quitar de la lista» solo para el administrador y pide confirmación en la fila) |
| **Entrada y cuentas (ADR-028)** | Las pantallas de entrada son una bandeja centrada sobre el fondo: el nombre del programa arriba, un ícono en cuadro `ink`, el título, una línea de ayuda, el formulario y, abajo, «Sus datos se quedan en esta computadora». Crear la cuenta de administración usa una bandeja más ancha con los campos en dos columnas y, si la computadora tenía PIN, un interior para pedirlo. El código de recuperación se muestra grande y fácil de copiar, con una etiqueta ámbar «No se vuelve a mostrar». La pantalla bloqueada muestra a la persona (avatar, nombre, rol y «Bloqueado»). Arriba a la derecha, un **menú de la persona** (avatar, nombre, rol y «Cambiar mi contraseña»); en el riel quedan «Bloquear» y «Cerrar sesión»; «Administración» y «Ayuda automática» solo aparecen con el permiso. **Administración**: pestañas Personal y cuentas (conteos, cuentas con rol y estado, «Dar acceso», contraseña temporal, activar y desactivar), Solicitudes (pendientes como interiores con «Devolverlo» y «Borrar por completo» en rojo sólido, que pide confirmación; historial), Bitácora (filtro, fecha y hora, quién, etiqueta de color por tipo de evento) y Recuperación (renovar el código) |
| **Ventanas** | Igual que hoy, con el estilo de bandeja |

## 11. Cambios frente al sistema actual

| Hoy | Propuesta |
|---|---|
| Lienzo gris + una hoja blanca + barra superior de texto | Riel de íconos y cápsula superior, con bandejas directamente sobre el fondo de la ventana (sin hoja ni marco) |
| Casi sin color (tinta + un azul) | Ocho acentos sólidos y vivos, iguales en claro y oscuro, con significado fijo |
| Inter | Plus Jakarta Sans |
| Pestañas segmentadas | Pestañas con efecto de carpeta: la activa se une a su bandeja (sin superficie gris detrás) |
| Etiquetas apagadas | Etiquetas sólidas con el texto de 3.2 |
| Radios 10 / 20 | 14 / 20 / 28 / 32 |

## 12. Riesgos y decisiones abiertas

- **Pestañas recortadas** (esquinas cóncavas) necesitan CSS propio y `drop-shadow`; se resuelven con un componente `Tray` que concentra el truco. Riesgo bajo.
- **Más color puede cansar.** La regla de «un color grande por pantalla» es lo que lo evita; hay que revisarla con las usuarias.
- **Riel de íconos:** sin etiqueta visible cuesta reconocer las secciones. Mitigación: tooltip inmediato y etiqueta bajo el ícono activo. Decidir en la prueba con las monjas.
- **Avatares de color para personas:** vuelven a aparecer nombres de personal en pantalla; es lo mismo que hoy (datos locales) pero hay que mantener que no salgan en documentos.
- **Fuente:** `@fontsource-variable/plus-jakarta-sans` se empaqueta con la app (sin red); confirmar el peso extra (~100 KB).
- **Ventana pequeña:** por debajo de 900 px el riel pasa a fila y las bandejas a una columna; verificar en la ventana real de Tauri.

## 13. Validación

1. Prototipo navegable de todas las pantallas (Inicio, Mis proyectos, Mi institución, Documentos, Asistente y sus ventanas), aprobado por la institución. Era la condición de la propuesta original.
2. Aplicación por capas, con las pruebas, `tsc` y la compilación en verde en cada una: **(1)** este documento y el ADR → **(2)** tokens y componentes base → **(3)** marco, riel y cápsula → **(4)** pantalla por pantalla.
3. Prueba corta con dos o tres personas de la institución: ¿encuentran «Continuar»?, ¿entienden qué significa cada color?

## 14. Cómo cambiar la interfaz

Reglas para quien toque la interfaz (persona o herramienta). La prueba `src/design.test.ts` revisa las que se pueden revisar con texto.

1. **Empieza por el documento.** Un componente nuevo o una medida nueva se escribe primero en §7.1 y, si hace falta, en §5.1.
2. **Un token, un lugar.** Colores, tamaños de letra, radios, alturas y sombras se definen solo en `src/index.css`. En las pantallas se usan por nombre (`bg-card`, `text-ink-2`, `rounded-inset`, `text-ui`).
3. **Las pantallas componen, no inventan.** Un campo, botón, etiqueta, avatar, bandeja, ventana o tabla sale de `src/components/ui/`. Si no existe el componente, se crea ahí (con su fila en §7.1) y se usa; nunca se copia el estilo en una pantalla.
4. **Sin valores sueltos.** No hay colores hexadecimales, ni `text-[13px]`, `rounded-[20px]`, `shadow-[…]` ni escalas de Tailwind (`stone-`, `blue-`, `text-sm`, `rounded-xl`) fuera de `src/index.css`. Las medidas de **disposición** (`grid-cols-[200px_1fr]`, `max-w-[640px]`) sí pueden ser arbitrarias porque no son identidad.
5. **El color nombra** (§3.4): un color, un significado. Estado = acento sólido + palabra. Un proyecto, un color.
6. **El texto de la interfaz** va en `src/i18n/es-MX.ts` y sigue `docs/08-estilo-redaccion.md`.
7. **Accesibilidad:** foco visible (el contorno es global), objetivo táctil ≥ 36 px (44 para lo principal), contraste según §9, y siempre palabras junto al color.
8. **Tema claro y oscuro:** todo se resuelve con los tokens semánticos; si algo se ve mal en uno de los dos temas, el defecto está en el token, no en la pantalla.

## 15. Nombres que se conservan por compatibilidad

- **Color de proyecto.** La base y `src-tauri` guardan `blue · violet · teal · green · amber · orange · pink · red` (`PROJECT_COLORS`). La interfaz los muestra con los acentos de §3.2 mediante una sola tabla (`src/lib/palette.ts`): `blue→sky`, `pink→rose`, `orange→cyan`, el resto con su mismo nombre. En el selector de color no se ofrecen `amber` ni `red` (están reservados para estado, §3.4), pero un proyecto que ya los tenga los sigue mostrando.
- **`orange`.** En la paleta del prototipo se llamó así al amarillo; en el código se llama `amber`.
