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
| `ink` | `#000C25` | `#F3F3F5` | Texto principal, botón principal, casillas marcadas |
| `ink-2` | `#55555E` | `#B4B4BD` | Texto secundario |
| `ink-3` | `#7E7E89` | `#8A8A94` | Etiquetas, ayudas, celdas vacías |
| `line` | `#E6E6EA` | `#2E3037` | Separadores |

Texto de cuerpo: `ink` sobre bandeja (≥ 15:1). `ink-3` solo para etiquetas de 12–13 px, nunca para datos.

### 3.2 Acentos sólidos

Ocho colores **frescos y vivos, idénticos en modo claro y oscuro**, y **la misma letra azul oscuro** (`#000C25`) sobre todos, en cualquier tema: etiquetas, avatares, íconos de color, pestañas de carpeta, píldoras seleccionadas y avisos. Una sola regla, sin excepciones por color ni por tema. Concuerda además con el botón principal azul oscuro de la interfaz.

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

La herramienta se llama **SociAI** y su azul es el del logotipo: `brand` (`#0a76fc`), con letra blanca encima (`on-brand`). Es **solo de identidad** y siempre **liso**: sin degradados, brillos ni sombras. Se usa en el panel izquierdo del primer inicio (`.onb`, pantalla dividida con los formularios a la derecha), donde el logo va todo en blanco (`<Logo onBrand>`). No es un acento de estado ni significa nada (§3.4). Fuera del primer inicio es el acento de la institución (Inicio, Mi institución y Documentos, §3.7). En ese panel el logo va todo en blanco y el isotipo (`public/SVG/isotipo.svg`) se usa como imagen grande en un azul más claro, liso. El programa abre siempre en tema claro; el oscuro solo se aplica si la persona lo elige.

**Roles del azul en el primer inicio** (cada color, un papel; como mucho tres azules visibles por pantalla):

| Rol | Token | Uso |
|---|---|---|
| Marca | `brand` | Botón principal, elegido, avance hecho, ícono clave, cifra destacada, la palabra «AI» |
| Azul oscuro | `brand-ink` | Títulos, texto fuerte, el paso donde se está |
| Texto | `.onb` redefine `ink`, `ink-2`, `ink-3`, `line`, `inset` y el borde de campo sobre el azul oscuro (100 %, 74 %, 60 %, 16 %…) | Todo el texto y las líneas heredan su temperatura sin tocar los componentes; en el tema oscuro no cambia |
| Tinte | `brand` al 5–9 % | Superficies informativas y la opción elegida |
| Estado | hecho = `brand` con palomita · falta = `amber` · falló = `red` | Siempre con palabra. Solo los acentos decorativos (verde, violeta, cielo…) se vuelven azules; **ámbar y rojo no**, para que «hecho» y «falta» nunca se vean iguales |

Sobre el azul liso, el blanco da 4.2:1: pasa para texto grande y componentes, no llega a 4.5 en texto chico. Por eso el texto del panel va en 14 px o más, nunca por debajo del 100 % de opacidad y **nunca sobre la figura del isotipo** (queda en la parte alta del panel; ahí no hay texto).

### 3.3 Etiquetas sólidas y tintes

Las etiquetas de estado y de categoría son **sólidas**, del mismo color vivo que el resto de la interfaz; un fondo pastel con texto oscuro se ve apagado junto a los avatares, íconos y bandejas de color. La letra de encima es siempre negra (3.2): una sola regla para etiquetas, avatares, íconos de color, pestañas de carpeta, píldoras seleccionadas y avisos.

El **tinte** (acento al 18 % sobre la bandeja, texto en el acento mezclado al 58 % con tinta) queda solo para datos informativos de bajo énfasis, como «Gastos de inversión». Si algo es estado, es sólido.

### 3.4 Reglas de uso

- **El color nombra.** Un color pertenece a un significado y no se reutiliza para otro. La tabla de 3.2 es la lista completa.
- **Un proyecto, un color.** Cada proyecto conserva el color que su dueña le puso (ya existe en la base, `project.color`) y **todo lo suyo va en ese color**: la pestaña de la carpeta, la etiqueta del paso, las barras de avance y el recuadro «Lo siguiente». No se mezclan colores dentro de una carpeta: mezclarlos pesa, cuesta leer de qué proyecto se trata y rompe la organización. Los **pasos no tienen color propio**: se distinguen por su nombre y por cuántos están llenos.
- **Las personas** (personal, beneficiarias) se muestran con iniciales sobre un color sólido de la serie `sky, violet, teal, cyan, rose, green, amber`, siempre el mismo para el mismo nombre. Solo en este equipo; no sale en documentos ni a la IA (`docs/03-gobernanza-datos.md`).
- **Estados siempre con texto:** «Listo», «Por confirmar», «Mal estado». Etiqueta sólida, no solo punto.
- **Un solo color grande por carpeta.** El resto de la pantalla es blanco y neutro. Varias carpetas de colores distintos conviven porque cada una es de un solo color.
- **Rojo solo para lo destructivo o lo que falló.**

### 3.5 Estados sin cápsula

Un estado (`Status`) se dice con **una marca redonda de 20 px y su palabra**, sin fondo ni cápsula detrás: ✓ «Lista» (marca `--st-ok`: verde en el programa, azul de marca en el primer inicio), 🕓 «Pendiente» (un reloj, `amber`), ⚠ «Cuidado» (`amber`, para lo que puede salir mal), ! «No se pudo» (`red`) y un círculo discontinuo para «Sin empezar». La palabra va en `text-small` 700, en la tinta del estado (`amber-ink`, `red-ink`) o en `ink` cuando está hecho.

Un aviso (`Alert`) dentro del primer inicio es **una regla de 3 px del color del estado, su marca y el texto**, sin caja de fondo. La cápsula sólida (`Tag`) queda para categorías (un puesto, un grupo, un tipo de archivo), no para decir si algo está bien o mal. Fuera del primer inicio los estados siguen como estaban hasta que cada pantalla se pase a `Status`.

### 3.6 Identidad de los módulos y logotipo (decisión del 2026-10-08)

Se probaron isotipos de color por módulo, tintes y aros propios y varias formas de tarjeta, y el resultado mezcló demasiados colores, íconos y superficies. Se volvió a lo que ya funcionaba y se fijó así:

- **Un módulo se reconoce por su nombre, su ícono de línea (`MODULE_META`) y el acento de su página (§3.7).** Nada más: sin isotipos ni logotipos por módulo, sin fondos, aros ni tintes propios de módulo. Un ícono o un color de módulo nunca se escribe a mano en una pantalla.
- **Todos los íconos de la interfaz son de línea** (`src/components/icons.tsx`), de una sola tinta. La única imagen que no lo es es el dibujo de burbujas de la bienvenida del primer inicio.
- **El logotipo en la interfaz diaria** es solo la S (`LogoMark accent`), arriba, y toma el color de acento de la página en que está la persona (§3.7): el de cada módulo y el de Documentos, y el azul de marca en Inicio, Mi institución y los ajustes; el color cambia con una transición suave. El logotipo completo y el isotipo grande viven solo en el primer inicio (§3.2.1).
- Los archivos `public/SVG/isotipo_*.svg` y `*_logo.svg` se conservan como material de marca para documentos y presentaciones; la interfaz no los usa.
- Un color nuevo se agrega a la tabla de 3.2 con su significado, o no se agrega. La prueba `src/design.test.ts` impide el azul de marca y los archivos de `public/SVG` fuera del primer inicio.

### 3.7 Acento por pantalla (decisión del 2026-10-09)

Referencia: un tablero claro donde casi todo es blanco, gris y negro y **un solo color** marca lo importante. Esa es la regla.

- **La estructura es neutra:** `ink` (azul oscuro), blanco y grises. El botón principal, la sección activa de la barra y los íconos de interfaz son `ink`.
- **Cada página lleva un solo acento**, el de la sección donde se está. La ventana lleva la clase `.accent-*` (`accentOf`, en `components/modules.ts`) y las piezas con `tone="ac"` (pestaña de carpeta, barras, etiquetas) lo leen. No se escribe un color de módulo en ninguna pantalla.

| Sección | Acento | Valor |
|---|---|---|
| Inicio, Mi institución y la configuración | Azul eléctrico (el de la marca, `brand`) | `#0A76FC` |
| Documentos | Azul aciano (decisión del 2026-10-09) | `#6495ED` |
| Proyectos | Menta | `#5DE2B6` |
| Personal | Coral | `#FF6F61` |
| Beneficiarios | Lila | `#B57EDC` |
| Instalaciones | Naranja mandarina | `#FF9C2F` |
| Finanzas | Verde lima | `#A1DB58` |

Paleta de módulos del 2026-10-09: tonos más suaves, elegidos para que la letra azul oscuro encima se lea bien (todos pasan de 6:1). Proyectos pasó de `#A4E158` a menta `#5DE2B6` porque se confundía con el verde de Finanzas.

- **Qué lleva el acento:** la pestaña de la carpeta de la página del módulo, el relleno de las barras de avance que están en curso y el marcador de la cifra principal. **Qué no:** botones, texto, líneas finas ni fondos grandes. Los claros (lima, menta, cian, naranja) casi no se leen sobre blanco, igual que el amarillo de la referencia, así que siempre son un **relleno con letra azul oscuro encima**; solo el azul puede ir como texto.
- **Los estados** (verde «Listo», ámbar «Falta algo», rojo «Falló») son solo de estado: no se usan de acento ni al revés. Ningún acento queda a menos de unos 20° de tono de un color de estado (el verde de Finanzas, `#A1DB58`, es amarillento y queda a unos 60° del verde de «Listo»).
- **Las categorías** (un puesto, un tipo de ingreso) van en gris, no en colores. Las personas conservan sus iniciales de color (§3.4). Un proyecto sigue con su propio color dentro de su carpeta (§3.4).
- **Las cifras que cruzan módulos** (Mi institución, Inicio) llevan su ícono en `ink`; el acento de la pantalla solo aparece en su barra.

## 4. Tipografía

Una sola familia: **Plus Jakarta Sans** (variable, 400–800), de formas geométricas y abiertas, legible en pantallas pequeñas y con buen soporte de acentos. Respaldo: `"Segoe UI Variable", system-ui, sans-serif`. Cifras con `font-variant-numeric: tabular-nums` cuando se alinean.

| Rol | Tamaño / interlínea | Peso | Uso |
|---|---|---|---|
| Cifra de gráfica | 56 / 1 | 300 | Porcentaje en el hueco de una dona |
| Leyenda de gráfica (`legend`) | 10 / 1.3 | 300 | «Vacíos», «Ingreso», «Egresos», «Disponible», «Listos» |
| Cifra grande | 72 / 1 | 400 | Fecha del calendario, total, pendientes |
| Título de bienvenida (`headline`) | 40 / 1.1 | 700 | Solo las tres pantallas de bienvenida del primer inicio |
| Título de página | 28 / 1.15 | 700 | Nombre de la institución, del proyecto |
| Título de bandeja | 18 / 1.25 | 700 | «Mi institución», «Asistente» |
| Cuerpo | 15 / 1.6 | 500 | Chat, textos |
| Interfaz | 14 / 1.45 | 600 | Botones, filas, valores |
| Etiqueta | 12.5 / 1.3 | 600 | Encabezados de tabla, ayudas |

Mínimo 14 px para todo lo que se lee o se pulsa; 12.5 solo para etiquetas. Pesos (decisión del 2026-10-09: la misma letra, más fina; se probó el 200 y se perdía en pantalla, así que se subió un paso): 300, 400, 500, 600. El texto base va en 300 (Light); las cifras grandes, en 300; los títulos, botones y etiquetas, en 500; los avatares y pestañas, en 600. Antes era 400–800.

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
| Peso | `font-normal` (cifras) · `font-medium` (base) · `font-semibold` · `font-bold` (títulos, botones, etiquetas) · `font-extrabold` (avatares, pestañas) | 300 · 300 · 400 · 500 · 600 |
| Radios | `rounded-tick` · `rounded-field` · `rounded-inset` · `rounded-card` · `rounded-pill` | 8 · 14 · 20 · 28 · 999 px |
| Alturas | `h-tag` · `h-ctl-sm` · `h-ctl` · `h-field` | 28 · 36 · 44 · 48 px |
| Sombras | `shadow-card` · `shadow-float` | ver §2 |
| Espacio | escala de 4 px de Tailwind, solo 1 · 2 · 3 · 4 · 6 · 8 · 12 (4, 8, 12, 16, 24, 32, 48) | §5 |

Los tamaños, radios, colores y sombras de Tailwind que no están en estas tablas **no existen** (se vaciaron en `@theme`): una clase como `text-sm`, `rounded-xl` o `bg-stone-100` no genera nada.

## 6. Estructura de pantalla

```
┌ Fondo de la ventana ────────────────────────────────────────────┐
│  (S)                                                (☾)  Institución (C) │
│  (⌂)   ┌ bandeja ┐  ┌ bandeja ┐  ┌ bandeja con pestaña ──────┐ │
│  (▣)   │         │  │         │  │                           │ │
│  (♜)   └─────────┘  └─────────┘  └───────────────────────────┘ │
│  (▤)   ┌ bandeja ──────┐  ┌ chat ───────────┐  ┌ cifra ─────┐ │
│  ...   └───────────────┘  └─────────────────┘  └────────────┘ │
│  (⏻)                                                            │
└─────────────────────────────────────────────────────────────────┘
```

- **Ventana:** el contenido ocupa toda la ventana con 16 px de margen y 1440 px como máximo, **directamente sobre el fondo**: no hay marco, hoja ni sombra entre el fondo y las bandejas.
- **Barra superior (decisión del 2026-10-09, reemplaza al riel izquierdo):** en una sola fila, a la izquierda el logotipo (`LogoMark accent`, 36 px de alto: la misma altura que el círculo del avatar de la persona, solo la S, **del color de acento de la página**: el de cada módulo, y el azul de marca en Inicio y Mi institución); al centro, **las secciones con su nombre** directamente sobre el fondo, sin bandeja ni píldora detrás (`topnav`), con 28 px de aire arriba y 28 px entre la barra y el contenido; a la derecha, el botón del tema y la configuración (engrane) **solo con su ícono, sin círculo** (`IconButton variant="plain"`), y la persona (su avatar, que sí conserva su círculo de color). No hay logotipo completo ni cápsula de búsqueda o de proyecto: el proyecto en curso vive en Inicio.
- **Secciones de la barra (ADR-032):** tres de la institución, **Inicio, Mi institución y Documentos**, una línea fina, y los cinco módulos, **Proyectos, Personal, Beneficiarios, Instalaciones y Finanzas**. Cada una es una pestaña de texto de 36 px; la de la página donde se está va rellena de `ink` con letra `on-ink`, las demás en `ink-2`. Los nombres siempre están a la vista (el riel de solo íconos obligaba a adivinar). **El engrane** abre un menú con lo que no es una sección del trabajo: Ayuda automática, Seguridad, Administración (según el permiso, ADR-028) y Ayuda. **El color del avatar es de cada cuenta:** cada persona lo elige en su menú («Mi color», siete colores de la serie) y queda guardado por usuario en este equipo (`useAvatarTone`); es el mismo en todas las pantallas y no depende de la sección en que se esté (morado si no ha elegido). **El menú de la persona** trae nombre, rol, «Mi color», cambiar contraseña, bloquear y cerrar sesión en rojo. Con ocho secciones la barra necesita unos 1100 px: por debajo, las secciones pasan a una segunda fila que se desplaza de lado y deja a la vista la sección actual. El ícono de línea de cada módulo (`MODULE_META`) se usa en la pestaña de su página y en sus cifras, no en la barra.
- **Pasos de una ficha o de un asistente (`StepNav`):** círculos sobre una línea y **una sola palabra** debajo (nunca dos líneas de texto). El círculo lleva el número; verde con una palomita es un paso hecho, negro el paso donde está la persona, y un anillo que se llena en verde dice cuánto del paso está capturado. **En el primer inicio (`brand`) hay un solo color de avance:** los pasos hechos son azul de marca con palomita y el paso actual también azul de marca, con su número y un aro suave alrededor (`.step-here`); el azul oscuro queda solo para texto. Se comparó con el paso actual en azul oscuro (se leía como un punto negro ajeno y apagado) y con un círculo hueco (pesaba poco y se confundía con los pasos que faltan). Un punto ámbar en la esquina avisa que el paso pide revisar algo. La línea entre círculos se pinta de verde mientras los pasos van hechos. En pantallas angostas solo quedan los círculos y, debajo, el nombre completo del paso actual.
- **Respuestas cortas:** una pregunta de sí / no / no sé, o de hasta tres respuestas (En trámite…), se contesta con un solo grupo sencillo (`Segmented`: una pastilla gris con la elegida en blanco), sin puntos ni colores. Las listas de más opciones y las de varias respuestas usan etiquetas (`Choice`).
- **Campos en una fila:** los campos que comparten fila se alinean por arriba; si algo no cabe en una línea, la fila se parte en dos líneas (el nombre completo arriba y los demás debajo) en lugar de cortar el texto. Las tarjetas de un resumen se acomodan en columnas que se llenan hacia abajo, sin dejar huecos por tener alturas distintas.
- **Botones sin relleno (2026-10-09):** los botones de solo ícono (`IconButton`) no llevan fondo ni borde en reposo ni al pasar el mouse: no sale ningún círculo detrás. **El ícono sigue siendo de línea fina (trazo de 1.8) en `ink`**: se probó sólido y el azul oscuro lleno pesaba demasiado. El «hover» y el foco son una animación apenas perceptible: el ícono crece un 6 % en 0.24 s y al pulsar se encoge un 4 %. Los botones de texto sin fondo (`ghost`, `plain`) y las secciones de la barra solo cambian la tinta de `ink-2` a `ink`. Solo conservan relleno el botón activo (`ink`), el de color de módulo (`tone`) y el de peligro.
- **Marco fijo:** la barra superior no se mueve al desplazar la página: solo se desplaza el contenido. Queda por encima del contenido (sus menús nunca quedan debajo de una página); los avisos y ventanas, por encima de todo.
- **Avisos de una ficha:** una ficha que tiene cosas por revisar las dice al abrirse, en un aviso arriba (`IssueSummary`): cada una con sus palabras, el paso donde está y «Ver» para ir allí; en la navegación de pasos, el paso lleva una marca ámbar. Lo que impide guardar va aparte, en rojo.
- **Cuadrícula:** 12 columnas, separación de 16. Las bandejas ocupan 3, 4, 5, 6 o 8 columnas; en pantallas angostas (< 900 px) pasan a una columna y las secciones pasan a una segunda fila.
- **Bandeja con pestaña:** el título vive en una pestaña recortada en la esquina superior izquierda (esquinas cóncavas); sirve para bandejas con un tema («Mi institución», «Resumen del proyecto»). Las pestañas **de una página** (Mi institución, Administración) no llevan bandeja gris: la pestaña activa es blanca y se une a la bandeja de su contenido con esquinas cóncavas, y al cambiar de pestaña se mueve la unión.
- **Bandeja de proyecto como carpeta:** cada proyecto se muestra en una bandeja con **pestaña de carpeta**: 46 px de alto, **una sola línea** con el título del proyecto, esquinas superiores de 18 px y una esquina cóncava de 18 px a la derecha. Una pestaña más alta (con dos líneas) se ve como una gorra, no como una pestaña. La pestaña lleva **el color del proyecto**, el que su dueña le asignó (`project.color`), con el texto de 3.2. **Quien convoca va dentro del cuerpo, a la derecha de la etiqueta del paso**, en `ink-3`. El título que no cabe se corta con puntos suspensivos y el completo va en el atributo `title`. El **paso** va como etiqueta sólida **en el mismo color del proyecto**; los pasos no tienen color propio. El cuerpo es blanco, con la esquina superior izquierda recta bajo la pestaña, y la sombra va con `drop-shadow` para seguir la forma. En la bandeja del proyecto en curso, **la fecha de cierre va dentro de la pestaña**, a la derecha del título, como una cápsula blanca («Cierra el 15 oct»; en pantallas angostas solo «15 oct»); en las tarjetas resumidas la fecha va en el pie. No hay botón «Continuar» en la pestaña.

### 6.1 Anatomía universal de una página (decisión del 2026-10-09)

Toda sección —las tres de la institución y los cinco módulos— se arma igual, de arriba abajo. Lo único que cambia es el contenido y el acento (§3.7).

| Zona | Qué es | Componente | Regla |
|---|---|---|---|
| 1 · Cabecera | Sobre el fondo, sin bandeja: título (`text-title`), **una línea** que dice para qué sirve y, a la derecha, **la acción principal** (un solo botón `ink`) | `PageHeader` | La lleva **toda** página. Inicio la usa con el saludo; Mi institución es la única excepción (abre con su propia ficha, que hace de cabecera) |
| 2 · Cuerpo | Una de dos formas, o las dos una tras otra | ver abajo | 24 px entre la cabecera y el cuerpo; 16 entre bandejas |
| 3 · Avisos | Aviso breve abajo después de guardar | `Toast` | Nunca dentro del cuerpo |

**Formas del cuerpo.**

- **Tablero** (Inicio, Mi institución, Ayuda, Seguridad): una cuadrícula de bandejas blancas. Cada bandeja abre con su título (ícono de línea en `ink` + nombre, a la izquierda) y, si lo tiene, un solo enlace o botón a la derecha. Cada dato aparece una sola vez.
- **Carpeta** (Documentos, Proyectos vacío, Personal, Beneficiarios, Instalaciones, Finanzas, Administración): una bandeja con **pestaña de carpeta** (`Folder`) en el acento de la sección. **Las vistas de la página son las pestañas** (`Folder tabs`): la elegida lleva el acento y el ícono del módulo y se une al cuerpo con esquinas cóncavas; las demás son solo texto sobre el fondo, con su conteo. Un módulo con una sola vista tiene una sola pestaña, con el nombre de lo que contiene («Equipo», «Balance del año»). Nunca hay un control gris (`Segmented`) para cambiar de vista de la página: ese control es solo para cambiar **cómo se ve** lo mismo (agrupar por…).
- Una página puede poner **bandejas sueltas debajo de la carpeta** (Finanzas: Ingresos y Egresos).
- Un lugar con **su propio color** (un proyecto) conserva su carpeta de ese color; lo demás usa el acento de la sección.

**Dentro de la carpeta**, de arriba abajo: la barra de herramientas (búsqueda, filtros y el «Agregar…» de la vista, a la derecha), el contenido y, al final, la casilla discontinua de agregar. La línea de ayuda de una vista, si hace falta, va arriba de la barra y nunca repite la de la cabecera.

**Estado vacío.** Siempre igual: ícono en círculo, una frase que dice qué falta, una línea de cómo se llena y **el mismo botón principal de la cabecera**, dentro de la carpeta (o la bandeja) de la página.

**Qué lleva el acento en esta estructura:** la pestaña elegida, las barras de avance en curso y el marcador de la cifra principal. Nada más.

## 7. Componentes

- **Botón principal:** fondo `ink`, texto blanco, 44 px, radio 999. Uno por bandeja como máximo. Secundario: blanco con borde de 1 px; terciario: texto.
- **Botón circular de ícono:** 44 px, blanco, borde 1 px `line`; activo en `ink`; peligro en `red` sólido.
- **Etiqueta (tag):** fondo sólido del acento, texto según 3.3, radio 999, 28 px de alto, 13 px, peso 700. Variantes: `soft` (tinte) para contexto y `line` (borde de 1.5 px, sin relleno) para datos neutros como «Sin redactar». Siempre con palabras.
- **Casilla:** 22 px, radio 7; marcada = `ink` con palomita blanca; el texto hecho se tacha y baja a `ink-3`.
- **Avatar de iniciales:** círculo de 36 px (28 en filas densas) de color sólido; pila de avatares solapados 10 px con anillo de 2 px del color de la bandeja.
- **Tarjeta de persona:** interior (nivel 3) con avatar, nombre, dato secundario y estado a la derecha.
- **Contenedor de documentos:** carpeta con **pestaña del acento de la sección** (azul, §3.7) y el conteo en una cápsula blanca dentro de la pestaña. Dentro, de arriba abajo: una frase que dice qué contiene, una fila de herramientas (búsqueda de 42 px con radio completo y el control segmentado «Agrupar por») y los grupos. Cada grupo es una **subcarpeta**: tiene su **viñeta** (pestaña gris tintada de 40 px con flecha, ícono o avatar de color con iniciales si agrupa por donante, nombre y conteo) y, desplegada, un **borde de 2 px** del mismo tono que agrupa sus documentos (esquina superior izquierda recta, para que la viñeta «nazca» del borde). Las subcarpetas cuelgan de una **línea de tronco** vertical con un ramal corto hacia cada viñeta, sangradas unos 38 px respecto al contenedor para que se lea la estructura. Se **pliegan y despliegan** (solo la primera arranca abierta; con búsqueda se abren todas; el estado se conserva al reagrupar). Al final de cada subcarpeta abierta va el botón **«Subir a «Nombre»»** (contorno discontinuo) y la pista «o arrastre un archivo aquí» (se oculta en pantallas angostas); el botón puede ocupar dos líneas si el nombre es largo. Arrastrar un archivo sobre una subcarpeta la resalta con borde discontinuo y fondo `inset`. Si la búsqueda no encuentra nada, dice «No encontramos documentos con eso.»
- **Ícono de archivo:** cuadro de 44 px (38 en listas compactas), radio 14, con la extensión en negrita de 11 px, de **color según el tipo**: PDF `red`, Word `sky`, Excel y CSV `green`, PowerPoint `amber`, cualquier otro neutro. La letra es la negra de siempre.
- **Fila de documento:** ícono de archivo, nombre con extensión (en negrita, con puntos suspensivos si no cabe), una línea de datos en `ink-3` (tipo · tamaño · año, o tipo · tamaño · «Usado en N convocatorias»), la etiqueta de estado («Leído» en tinte verde; «Leyendo…» en sólido `amber` mientras el asistente lo procesa) y, al pasar el cursor o enfocarla, la acción de quitar. Quitar pide confirmación **en la misma fila** («Quitar» en rojo y «Cancelar»); en pantallas angostas el estado «Leído» se oculta para dejar sitio al nombre.
- **Subida de archivo (campo de formulario):** zona de arrastre con borde discontinuo; al elegir el archivo se transforma en una fila con su ícono de color, el nombre, el tamaño y «pulse para cambiarlo». Los campos pueden depender de otro: el de «Donante» solo aparece si el documento es de un donante. **Subir desde una subcarpeta:** el destino ya se conoce, así que el formulario se titula «Subir a «Nombre»», muestra una ruta («De los donantes › Nacional Monte de Piedad») y solo pide lo que falta: el archivo y, si hace falta, el donante o el tipo (opcional). Los archivos arrastrados entran directo con «Por clasificar» o «Por asignar» como valores por defecto.
- **Tarjeta de proyecto (resumida):** la misma carpeta que la bandeja principal, en pequeño, para saber el estado sin abrir el proyecto. Pestaña con el título (color del proyecto); en el cuerpo, de arriba abajo: etiqueta sólida con el paso en el color del proyecto («Paso 3 · Objetivo», o «Listo» con palomita) y, a la derecha, quien convoca; los seis pasos como barras del color del proyecto (los hechos y el actual sólidos, los que faltan en tinte), un interior con «Lo siguiente» (o «Lo que sigue» si ya está listo) con la acción concreta, y el pie con la fecha de cierre, los días que faltan y el botón («Continuar» o «Abrir la guía»). El botón del proyecto más urgente es el principal; los demás, secundarios. Las pestañas tienen alto fijo (46 px) para que el cuerpo empiece a la misma altura en todas las tarjetas. Ancho mínimo 360 px; se reparten solas en la cuadrícula.
- **Casilla para agregar:** rectángulo con borde discontinuo de 1.5 px, radio 20, con «+»; sustituye a los botones «Agregar» sueltos.
- **Tarjeta de cifra (`Metric`, 2026-10-09):** el ícono va **limpio** sobre la superficie de la tarjeta, de línea y en `ink`, **sin cuadro de color detrás**, y el título en `ink` (no en gris). El número, su explicación y la barra no cambian. Los demás `Tile` del programa siguen como estaban hasta revisarlos.
- **Tarjetas de cifras de Inicio (`FigureCard`, `Waffle`, `Gauge`, 2026-10-09):** cada cifra es **su propia bandeja blanca, sin cajón gris adentro**, de **tamaño fijo** (248 px de alto; no crece con lo que lleva) y se abre completa hacia su módulo. Arriba, el ícono limpio de línea y el título en `ink`. Las gráficas usan **solo tres colores**: azul oscuro (`ink`), azul de marca y el gris del fondo; nunca un color solo (cada uno lleva su nombre) y cada gráfica tiene su `aria-label`.
  - **Disposición (según el dibujo del 2026-10-09):** el título va arriba; lo demás se acomoda **hacia abajo** de la tarjeta.
  - **Personas que atendemos:** a la izquierda, **el número grande de lugares ocupados** (72 px) y **debajo dos columnas** separadas por una línea fina vertical: «● 30 Vacíos» (punto, cifra y la palabra a su derecha) y **solo un chevrón** con su cifra a la derecha, sin palabra: ⌃ verde si lo último fue una entrada, ⌄ rojo si fue una salida, la última que ocurrió (`latest_movement`, que calcula Rust; si empatan, entradas), con la cantidad de este año; si este año no hubo ninguna, sigue la flecha de la última y la cantidad es 0. **Nunca una frase**: las columnas llevan una cifra y, como mucho, una palabra (regla de las tarjetas: no hay leyendas largas, porque aplastan el contenido). Con la cifra rápida del primer inicio, el aviso es el «≈» antes del número. A la derecha, **el waffle, de la misma altura que el número con sus columnas**, que llena el ancho que queda: un cuadro por lugar, azul de marca = ocupado, gris = vacío, llenado de izquierda a derecha y de abajo hacia arriba; arriba de su borde de lo ocupado, una tarjetita pequeña azul de marca con el porcentaje («25 % ocupado»). Con más de 100 lugares cada cuadro vale varios (se dibujan 100); una persona siempre se ve como un cuadro. Sin capacidad capturada no hay waffle ni tarjetita.
  - **Balance del año:** arriba, **tres valores en tres columnas iguales, cada leyenda centrada bajo su número**, con un puntito de 4 px del color (el mismo en todas las leyendas de las tarjetas) y la leyenda en **10 px** (`text-legend`, el tamaño de las leyendas de las gráficas): Ingreso, Egresos y Disponible. Debajo, **la media dona** (hasta 250 px de ancho, centrada) con las puntas redondeadas, también las de cada color: **el arco entero es el ingreso (gris)**, los egresos toman su parte en azul oscuro y lo disponible sigue en azul de marca, con aire entre los colores; un color muy chico se ve como una punta redonda completa. **En el hueco va el porcentaje en grande** (56 px, `text-figure`) con una línea pequeña que dice qué es: por defecto «29 % del presupuesto disponible». **Al pasar el mouse por un color** (o por su valor de arriba) los otros se atenúan y el hueco dice qué es: «97 % del ingreso se va en egresos», «100 % es todo lo que entra en el año». Si los egresos pasan del ingreso, todo el arco es azul oscuro y el disponible sale negativo. Sin ingresos o egresos capturados, solo el arco gris y el aviso de lo que falta.
  - **Proyectos en curso:** el número grande abajo a la izquierda y, debajo, la columna «Listos» y su cifra.
  - Las proporciones son solo de dibujo: las sumas las hace Rust (`gaugeShares` y `waffleShape` no calculan cifras). `Metric` (con su ícono limpio y el título en `ink`) sigue siendo la tarjeta de «Mi institución».
- **Cifra de bandeja:** número a 72 px peso 400 sobre un interior, con su etiqueta arriba y su explicación abajo.
- **Calendario:** interior con día de la semana, mes y la fecha en cifra grande. Se usa para el cierre de la convocatoria.
- **Chat:** mensajes de la persona en burbuja de interior alineada a la derecha con su avatar; los del asistente en burbuja de interior a la izquierda con avatar `ink` «AI». Caja de escribir: interior con campo, botón de micrófono (si existe) y envío circular `ink`.
- **Barra de pasos:** seis píldoras con el color de su paso; lo hecho lleno, lo actual lleno y lo que sigue vacío con el color al 25 % (`Steps`).
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
| `Tag` | 28 | pill | — | `text-small` 700, relleno 13 | `solid` (acento + `onc`), `soft` (acento al 18 % + texto mezclado), `line` (borde 1.5 px). Para categorías; los estados usan `Status` (§3.5) |
| `Status` | 20 (marca) | círculo | — | `text-small` 700 | `ok` · `pending` (reloj) · `warn` · `error` · `idle`; marca redonda + palabra, sin cápsula. `ok` toma `--st-ok` (verde; azul de marca en `.onb`) |
| `Avatar` | 36 (`sm` 28) | círculo | — | 12 (10.5) px 800 | color por nombre |
| `Tile` (cuadro de ícono) | 40 (`sm` 32) | `field` | — | — | acento sólido, ícono `onc` |
| `PageHeader` | — | — | — | título `text-title` 700; línea `text-ui` `ink-2` | en todas las páginas igual: título + una línea + acción principal a la derecha |
| `FormSection` | — | — | línea 1 px `line` | `Eyebrow` | agrupa campos en una ventana; puede llevar un `Tile` pequeño con un ícono antes del título |
| `Card` (bandeja) | — | `card` | `shadow-card` | — | relleno 24 (20 en pequeñas) |
| `Inset` (interior) | — | `inset` | — | — | fondo `inset`, relleno 16–20 |
| `Folder` (carpeta con pestaña) | pestaña 46 | pestaña 18, cuerpo `card` | `drop-shadow` | título `text-body` 800 | color del proyecto en `--pc`/`--pt`. Con `tabs` (varias vistas, §6.1): una pestaña por vista, la elegida en `--pc` con esquinas cóncavas y las demás como texto `ink-2`; si la primera no es la elegida, el cuerpo redondea su esquina superior izquierda |
| `Dock` y `DockTab` (pestañas de página, efecto carpeta) | 52 | 24 (esquinas cóncavas) | — | `text-ui` 700 | la pestaña activa es **blanca y se une a la bandeja de su contenido** (la primera bandeja bajo las pestañas lleva `dock-attach`); al cambiar de pestaña cambia la pestaña unida. No hay ninguna superficie gris detrás: pestaña y bandeja son la misma pieza |
| `Segmented` (control segmentado) | 44 (pista) / 36 (opción) | pill | — | `text-small` 700 | opción activa en `card` con `shadow-card` |
| `StepNav` | círculo 32 + una palabra | pill (círculo) | — | `text-small` 700 + `text-caption` | pasos de un formulario largo, navegables: cada uno muestra su avance real («9 de 12»), «No aplica» si no hay nada que llenar, y se vuelve verde con palomita al completarse; en pantallas angostas solo las barras y el nombre del paso actual |
| `MaskedField` | 48 | `field` | 1.5 px `line` | `text-ui` 700 | valor tapado de un identificador (CURP, RFC…) en la misma caja que cualquier campo, con «Mostrar» y «Cambiar» a la derecha |
| `Check` | 24 | `tick` | 2 px `line` | — | marcada = `ink` |
| `Switch` | 28 × 48 | pill | — | — | activo en `green` |
| `Choice` (píldora seleccionable) | 44 | pill | 1.5 px `line` | `text-ui` 700 | elegida = color sólido |
| `RadioCard` | auto | `inset` | 1.5 px `line`; elegida: `ink` | `text-ui` 700 + `text-small` `ink-3` | |
| `Alert` | auto | `inset` | — | `text-ui` | fondo del estado al 18 %, círculo sólido con ícono. En el primer inicio (`.onb`): sin fondo, regla de 3 px del color del estado, marca de 20 px y el texto |
| `Modal` | — | `card` | `shadow-float` | título `text-subtitle` 700 | anchos 640 / 768 / 896 (`wide`, para fichas largas) / 1024; `fixed` fija la altura para que un flujo de pasos no crezca y se encoja |
| `Table` | filas 52 | — | línea 1 px `line` | `text-ui`; encabezado `text-caption` `ink-3` | primera columna 700 |
| `Logo` / `LogoMark` | por altura (`LogoMark` `h-10` en la cabecera; `Logo` `h-10` en la entrada y la bienvenida) | — | — | el nombre es texto en Plus Jakarta Sans 800 | logotipo de SociAI (S + nombre; `LogoMark` es solo la S). Se pinta con los tokens `--logo-mark`, `--logo-word` y `--logo-ai`, que siguen el tema: sobre fondo claro, S azul y nombre azul marino; sobre fondo oscuro, todo blanco con «AI» azul. `inverse` es para superficies pintadas con `ink` (el lado oscuro del primer inicio), donde los colores se invierten. Los tokens `--logo-*` son solo del logotipo, no son acentos de la interfaz |
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
| **Inicio** | Responde «¿qué hago ahora?» y «¿se me viene un plazo?». Un saludo corto con «Empezar un proyecto nuevo» a la derecha y, abajo, **tres columnas de 248 px de alto**: «Personas que atendemos» y «Balance del año» (`GlobalFigures`, cada una abre su módulo; si nadie está registrado, la cifra rápida del primer inicio la sustituye con «≈») y, en la tercera, **dos tarjetas apiladas** (`fig-stack`): **«Por hacer»** (`TodoCard`, bandeja blanca; por ahora solo el marco, sus tareas se desarrollan después) y **«Mi institución»** (`InstitutionCard`, tarjeta de `ink`, azul oscuro): mientras los datos no estén completos muestra el nombre, una barra con el avance (`N % de sus datos`) y cuántos datos faltan; con los datos completos **no lleva ícono ni estado**, solo el nombre y las personas que tienen acceso (sus avatares, de `access_team`); toda la tarjeta abre Mi institución. Debajo, **una carpeta a todo el ancho con la estructura de los módulos** (`Ongoing`, `Folder` con pestañas): **«En curso»** (con la cuenta de lo que sigue en marcha) es donde se retoma lo que se estaba haciendo —hoy el proyecto en curso, en su color, con la etiqueta del paso, los seis pasos y el bloque «Lo siguiente»; al lado, el calendario del cierre y los otros proyectos como filas—, y a futuro lo de los demás módulos; **«Notificaciones»** queda con su estado vacío «Todo al día» hasta que haya avisos. Si hay datos por llenar o pasos siguientes, aparecen debajo de la carpeta. Sin chat y sin casillas: el avance lo decide el asistente, no la persona |
| **Mis proyectos** | Todos los proyectos con la misma tarjeta resumida que Inicio (la del proyecto más urgente incluida), ordenados por fecha de cierre; «Empezar un proyecto nuevo» como botón principal en el encabezado |
| **Mi institución** | Una sola bandeja de cabecera en dos mitades: a la izquierda la identidad, sin ícono de perfil, en tres alturas: «Mi institución» como etiqueta pequeña arriba, el nombre (32 px) y la misión centrados verticalmente con mucho aire entre ellos, y, pegados al fondo, el **estado del llenado** (`Status`: «Datos completos» en verde, o «Faltan N datos por llenar» con reloj ámbar y «Ver qué falta», que despliega cada dato pendiente como un botón que lleva a donde se llena; sale de lo que Rust dice que falta en cada paso de los datos, más las personas y los espacios que nadie ha registrado) y «Editar». **No hay estado de «revisado» ni botón «Confirmar»**: lo que se guarda aquí se confirma solo (ADR-031). Inicio muestra el mismo estado en su tarjeta de la institución; a la derecha un grupo de 2×2 con **una cifra por módulo** (Beneficiarios, Personal, Instalaciones y Finanzas), con el mismo `Metric` de Inicio (`ModuleCard`): su ícono de línea en un cuadro `ink`, el nombre, la cifra de 30 px y su detalle, con la barra en el acento; toda la tarjeta abre el módulo; mientras un módulo esté vacío, la cifra rápida del primer inicio la sustituye con «≈» y la etiqueta «Cifra aproximada». Ninguna cifra sale dos veces en la pantalla: las de un solo módulo viven en su página, y las que cruzan módulos (ocupación, balance, proyectos) en Inicio. Debajo, las pestañas con efecto de carpeta (la activa se une a la bandeja de su contenido, sin superficie gris detrás); datos en bandejas con filas; **dinero (ADR-026):** el balance (superávit en verde, déficit en ámbar, «Falta un dato» en neutro, con la barra de ingresos contra egresos y de dónde sale el egreso), los **ingresos por tipo** (fijos y variables, barra con un color por tipo, filas con tipo, al mes si así se escribió y «Del padrón» sin acciones para las cuotas que calcula el programa; las líneas que no suman, atenuadas y con su aviso) y los **egresos** (lista por concepto más la nómina con prestaciones que calcula el programa; para empezar, un «gasto aproximado» que la lista reemplaza); **Personal (ADR-027):** lista con nombre y «N cosas por revisar» en ámbar, puesto como etiqueta de color, modalidad, situación (punto verde si trabaja; etiqueta suave si está de vacaciones, con permiso, incapacidad o ya no trabaja) y una barra de «ficha completa» con su porcentaje; filtro por puesto, botón «Puestos» con una etiqueta ámbar de «N plazas sin cubrir», y la ficha en cuatro pasos navegables (`StepNav`) con identificadores tapados (`MaskedField`); el catálogo de puestos muestra personas contra plazas con una barra; el indicador de nómina muestra sueldos más prestaciones y avisa si a alguien le falta el contrato o el año de ingreso; en Personal, Beneficiarios e Instalaciones, el botón principal «Agregar …» va arriba a la derecha de la barra de herramientas **y** la casilla discontinua al final de la lista (los dos); ingresos con barra de composición; tablas con avatares y etiquetas. Por debajo de 1000 px la bandeja pasa a una columna y por debajo de 520 px el grupo también **Dinero (ADR-026, ADR-032):** a la derecha de los datos va «Capacidad» (cuántas personas caben y las notas; se edita en su propia ventana, junto con las cifras rápidas que las tarjetas de módulos usan mientras no haya registros). Debajo, el **Balance** a todo lo ancho: la cifra y su estado a la izquierda y, a la derecha, ingresos y egresos dibujados a escala con un tramo punteado que muestra lo que falta (ámbar) o lo que sobra (verde). Después **Ingresos** y **Egresos**: cifra del año, barra cortada por tipo con su leyenda y porcentaje, y cada línea con su parte del total; la nómina va en `amber`, lo que va a personas en `teal`, el resto en `ink`. El «Gasto anual aproximado» tiene su ventana, con la lista de lo que debe incluir. **Lo que lleva la institución (ADR-032):** debajo de los datos, una tarjeta con una ficha por módulo, cada una en el color de su módulo (tinte suave, ícono en cuadro de color, la cifra grande —personas, espacios, balance— y «Abrir»); toda la ficha es el botón. |
| **Espacio de trabajo (Asistente)** | La misma carpeta de proyecto que en Inicio, a ocho columnas: pestaña con el título y la cápsula de cierre, etiqueta del paso con quien convoca, los seis pasos y el chat (burbujas, respuestas rápidas como píldoras, caja con envío circular negro). A cuatro columnas, un panel con **tres pestañas**: **Proyecto** (objetivo, partes con su estado y el «Siguiente paso» con la barra «3 de 14» y el botón de revisión, que explica por qué está cerrado), **Convocatoria** (resumen y lo más importante) y **Fuentes** (los documentos que el asistente consulta para este proyecto: la convocatoria, los del donante y los de la institución, con un enlace a Documentos para administrarlos). Todo en el color del proyecto; solo los estados usan `amber` o `green` |
| **Documentos** | Dos contenedores de carpeta con pestaña azul (el acento), lado a lado: **«De mi institución»** (acta, estados financieros, reportes) y **«De los donantes»** (reglas, indicadores, formatos que casi no cambian y valen para varias convocatorias). Cada uno se **agrupa** a elección con un control segmentado: el de la institución por **tipo** (Legales, Financieros, Programas) o **año**; el de los donantes por **donante** o **tipo** (Reglas, Indicadores, Guías y formatos). Los grupos son subcarpetas con viñeta y borde, plegables, cada una con su botón «Subir a «…»» y arrastre (ver Componentes), y hay búsqueda en vivo. «Subir documento» (botón principal del encabezado) abre un formulario: de quién es (tarjetas), archivo, donante (si aplica, con sugerencias de los ya existentes) y tipo. Tras subirlo aparece como «Leyendo…» y pasa a «Leído» cuando el asistente termina |
| **Primer inicio (ADR-031)** | Pantalla dividida a ventana completa (ventana por defecto 1200×800, mínimo 900×640). **Izquierda:** el azul liso de la marca con el logo, el logo colgado de la esquina de arriba (fuera del flujo) y, **centrado en la altura de todo el panel**, un bloque que **en todas las pantallas tiene la misma forma**: un título grande (`headline`, 40 px, con las palabras clave en un azul más claro), una línea de información (15 px) y dos o tres datos con su ícono en un círculo de contorno blanco. La figura del isotipo es grande, un poco más clara que el panel (no más del 10 % de blanco), está anclada a la esquina inferior izquierda y sangra por la izquierda y por abajo; el texto va encima. En la bienvenida el bloque dice qué es SociAI («Su institución, organizada en un solo lugar», Todo conectado, Crece a su ritmo, Menos trabajo manual); en la puesta en marcha y en cada paso del asistente dice **qué se está haciendo ahí**, como información y no como instrucción (título del paso, para qué sirve, y los dos datos fijos «Se guarda en cada paso» y «Datos de personas, solo aquí»). Los textos viven en `onboarding.about` y `onboarding.facts`. **Derecha:** superficie blanca con cuatro zonas fijas: **A** arriba, el avance (tres barras en la bienvenida, `StepNav` en el asistente) a 96 px; **B** el título, siempre a la misma altura; **C** el contenido, que se desplaza con un desvanecido en los bordes; **D** los botones, limpios, sin línea divisoria, con el secundario o «Atrás» a la izquierda y el principal a la derecha, en las tres pantallas. El logo y el avance comparten línea central, y el último texto del panel y los botones, línea de fondo. En ventana angosta (< 1024 px) el azul queda como una franja con el logo, la ayuda del paso pasa bajo el título y los botones siguen fijos abajo. **Bienvenida:** tres pantallas; el título, la línea de ayuda y el dibujo forman **un solo grupo, centrado en la zona C y siempre de la misma altura** (título de una línea de 40 px en azul oscuro, línea de ayuda de a lo más 110 caracteres con sitio para dos, dibujo en un escenario de 21 rem), así que ni el título salta ni quedan huecos entre bloques. El nombre «SociAI» va **entero, en azul de marca y en peso 800** (más grueso que el resto), solo en la primera. El dibujo de la primera son **burbujas flotantes**: «Mi institución» en el centro y, alrededor, un círculo por módulo con **su isotipo real** (`public/SVG/isotipo_<módulo>.svg`, cada uno en su color: Personal/Capital Humano `#ca287a`, Beneficiarios `#8143ae`, Finanzas `#77c053`, Donantes `#fcb71a`, Proyectos `#07bfa2`, Documentos `#4597fd`), su nombre y una línea. Entran una por una, se mecen apenas y un foco (el círculo crece y emite un aro de su color) pasa de una a la siguiente cada 2.6 s; con «reducir movimiento» todo aparece de golpe y quieto. Los otros dos dibujos: lo que se queda en la computadora (nombres borrosos) frente a lo que ve la ayuda automática (una cifra); y la página real de «Ayuda» con sus preguntas. **Puesta en marcha:** una fila blanca con borde por tarea (ícono, nombre, detalle y su `Status` a la derecha) y, abajo, el bloque de ejemplos solo en desarrollo, con borde discontinuo. **Asistente:** siete pantallas (Institución, Ubicación y contacto, Datos legales, Personas y equipo, Dinero, Casa y Revisar) que reagrupan los seis pasos de Rust; no hay asteriscos, lo opcional va plegado en «Datos opcionales» y cada pantalla está lista cuando a Rust no le falta nada de lo suyo. El programa abre siempre en tema claro. |
| **Páginas de módulo (ADR-032)** | Personal, Beneficiarios, Instalaciones y Finanzas se abren con el mismo marco (`components/ModulePage.tsx`, anatomía en §6.1): la cabecera de toda página sobre el fondo (título, una línea y la acción principal) y debajo una carpeta **en el color del módulo** cuyas **pestañas son las vistas del módulo** (la elegida lleva el acento y el ícono del módulo). Personal y Finanzas tienen una sola vista y por eso una sola pestaña («Equipo», «Balance del año»). No hay otra tarjeta dentro de la bandeja. **Finanzas** pone el balance —la cifra y los dos lados a escala— como cuerpo de la pestaña verde y, debajo, sobre la ventana, las tarjetas de Ingresos y Egresos. |
| **Instalaciones (ADR-030)** | Cuatro vistas, que son las pestañas de la carpeta (Espacios, Inmueble, Equipo y Tablero, con conteos que salen del módulo). **Cómo están** se dibuja siempre igual: una barra cortada por estado (`StateBar`) en los colores de los estados —bien `green`, regular `amber`, mal `red`, no sirve negro (`ink`) y lo que no se ha revisado, gris atenuado— con sus palabras debajo (4 bien · 1 regular · 2 sin revisar). Espacios y Equipo abren con el resumen de todo en una barra y siguen con la tabla (cuántos, la barra y las fallas como etiquetas suaves ámbar con «+N»). **Captura de los cuatro estados:** un contador por estado (`Stepper`, con su marca de color, su ayuda, «−», el número y «+»), la barra que se llena mientras se cuentan y «Todos están bien»; lo que no se cuenta queda «sin revisar» y el «+» se apaga cuando ya se contaron todos. **Inmueble:** tres bloques de lectura (inmueble, servicios y seguridad) con «N de M capturados» y «Editar»; los sí/no van como etiqueta suave (sí `green`, no `amber`) y las frecuencias de menos a más grave (nunca `green`, a veces `amber`, seguido `red`); sus ventanas se agrupan por secciones con ícono. **Tablero:** cuatro cifras, «Lo que falta» (tarjetas con «Completar» que lleva a la vista donde se captura), los hallazgos con su marca (`Findings`, el mismo bloque que Beneficiarios), el estado de todos los espacios y de todo el equipo, el estado por tipo y lo que más falla |
| **Beneficiarios (ADR-029)** | Tres vistas, que son las pestañas de la carpeta (con conteos): **Personas** (misma lista que Personal: nombre y «N cosas por revisar» en ámbar, grupo como etiqueta de color, edad, situación con punto verde o etiqueta suave, barra de «ficha completa» con su porcentaje, y la casilla «Agregar persona» al final), **Tablero** y **Lista de espera**. La ficha es el mismo formulario fijo y ancho de Personal en cinco pasos con `StepNav` (avance por paso), encabezado con la persona y su avance total, una línea de qué se pide en cada paso e íconos por sección; «No sé la fecha exacta» es un interruptor que cambia la fecha por la edad aproximada; la CURP va tapada (`MaskedField`) con «Mostrar» y «Cambiar»; las opciones de varias respuestas (motivos de ingreso, discapacidad, salud, programas) son etiquetas que se activan; los campos propios de asilo o casa hogar aparecen según el tipo de institución. **Tablero:** cuatro cifras (`Figures`), «Lo que dicen sus datos» en dos columnas de tarjetas, cada hallazgo con ícono y marca (violeta «La ayuda automática también lo sabe» o neutra con candado «Solo para usted»), y debajo la pirámide por edad y sexo (mujeres a la izquierda en `rose`, hombres a la derecha en `sky`, con el conteo junto a cada barra) y barras ordenadas por categoría, cada una de un color propio (apoyo `violet`, movilidad `sky`, discapacidad `teal`, salud `rose`, motivos `amber`, programas `green`, situación legal `cyan`), con cantidad y porcentaje; dos columnas que se llenan alternadas para no dejar huecos. **Lista de espera:** tabla con solicitud (avatar o ícono si no hay nombre, teléfono), fecha y «Lleva N días esperando» (ámbar desde 90 días), datos, motivo y acciones («Darle ingreso», editar; «Quitar de la lista» solo para el administrador y pide confirmación en la fila) |
| **Entrada y cuentas (ADR-028)** | Las pantallas de entrada son una bandeja centrada sobre el fondo: el nombre del programa arriba, un ícono en cuadro `ink`, el título, una línea de ayuda, el formulario y, abajo, «Sus datos se quedan en esta computadora». Crear la cuenta de administración usa una bandeja más ancha con los campos en dos columnas y, si la computadora tenía PIN, un interior para pedirlo. El código de recuperación se muestra grande y fácil de copiar, con una etiqueta ámbar «No se vuelve a mostrar». La pantalla bloqueada muestra a la persona (avatar, nombre, rol y «Bloqueado»). Arriba a la derecha, un **menú de la persona** (avatar, nombre, rol y «Cambiar mi contraseña»); en el riel quedan «Bloquear» y «Cerrar sesión»; «Administración» y «Ayuda automática» solo aparecen con el permiso. **Administración**: pestañas Personal y cuentas (conteos, cuentas con rol y estado, «Dar acceso», contraseña temporal, activar y desactivar), Solicitudes (pendientes como interiores con «Devolverlo» y «Borrar por completo» en rojo sólido, que pide confirmación; historial), Bitácora (filtro, fecha y hora, quién, etiqueta de color por tipo de evento) y Recuperación (renovar el código) |
| **Ventanas** | Igual que hoy, con el estilo de bandeja |

## 11. Cambios frente al sistema actual

| Hoy | Propuesta |
|---|---|
| Lienzo gris + una hoja blanca + barra superior de texto | Riel de íconos e isotipo arriba, con bandejas directamente sobre el fondo de la ventana (sin hoja ni marco) |
| Casi sin color (tinta + un azul) | Ocho acentos sólidos y vivos, iguales en claro y oscuro, con significado fijo |
| Inter | Plus Jakarta Sans |
| Pestañas segmentadas | Pestañas con efecto de carpeta: la activa se une a su bandeja (sin superficie gris detrás) |
| Etiquetas apagadas | Etiquetas sólidas con el texto de 3.2 |
| Radios 10 / 20 | 14 / 20 / 28 / 32 |

## 12. Riesgos y decisiones abiertas

- **Pestañas recortadas** (esquinas cóncavas) necesitan CSS propio y `drop-shadow`; se resuelven con un componente `Tray` que concentra el truco. Riesgo bajo.
- **Más color puede cansar.** La regla de «un color grande por pantalla» es lo que lo evita; hay que revisarla con las usuarias.
- **Barra superior con ocho secciones:** en la ventana mínima (900 px) las secciones pasan a una segunda fila que se desplaza; comprobar en la prueba con las monjas que encuentran Finanzas e Instalaciones sin buscar.
- **Avatares de color para personas:** vuelven a aparecer nombres de personal en pantalla; es lo mismo que hoy (datos locales) pero hay que mantener que no salgan en documentos.
- **Fuente:** `@fontsource-variable/plus-jakarta-sans` se empaqueta con la app (sin red); confirmar el peso extra (~100 KB).
- **Ventana pequeña:** por debajo de 1100 px las secciones de la barra pasan a una segunda fila y por debajo de 900 px las bandejas pasan a una columna; verificar en la ventana real de Tauri.

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
9. **Pantallas modelo.** Inicio, «Mi institución» y una página de módulo (Personal) son la referencia. Una forma nueva de tarjeta, de ícono o de color se escribe aquí, se prueba primero en esas tres pantallas y solo entonces se extiende.
10. **Antes de agregar algo, busca lo que ya existe** (§7.1): si un `Metric`, un `Tile` o un `Status` resuelve el caso, se usa; si no, se discute el componente nuevo, no un estilo suelto.

## 15. Nombres que se conservan por compatibilidad

- **Color de proyecto.** La base y `src-tauri` guardan `blue · violet · teal · green · amber · orange · pink · red` (`PROJECT_COLORS`). La interfaz los muestra con los acentos de §3.2 mediante una sola tabla (`src/lib/palette.ts`): `blue→sky`, `pink→rose`, `orange→cyan`, el resto con su mismo nombre. En el selector de color no se ofrecen `amber` ni `red` (están reservados para estado, §3.4), pero un proyecto que ya los tenga los sigue mostrando.
- **`orange`.** En la paleta del prototipo se llamó así al amarillo; en el código se llama `amber`.

## 16. Auditoría de coherencia (2026-10-08)

Se revisó la interfaz contra §3, §6 y §14 después de agregar isotipos, tintes y tarjetas nuevas. Lo que se encontró, lo que ya se corrigió y lo que queda por decidir:

| # | Hallazgo | Regla | Estado |
|---|---|---|---|
| 1 | Los módulos tenían isotipos de color propios (6 colores nuevos) en el riel, la pestaña, las tarjetas y «Siguientes pasos» | §3.4 «el color nombra», §14.3 | **Corregido**: íconos de línea y un acento por página (§3.6, §3.7) |
| 2 | El logotipo iba en azul de marca, que §3.2.1 reservaba al primer inicio | §3.2.1 | **Corregido**: la S de la barra va en una sola tinta (`LogoMark mono`) |
| 3 | Los módulos usaban `green` (Finanzas) y `sky` (Instalaciones) sin estar en la tabla de §3.2, y `green` también significa «Listo» | §3.4 | **Corregido**: los módulos ya no toman colores de §3.2; Finanzas usa menta turquesa, a más de 20° del verde de «Listo» (§3.7) |
| 4 | Dos tarjetas de cifras distintas (`Metric` en «Mi institución» y tarjeta propia de módulo) y las mismas cifras en dos sitios | §1 «cada dato aparece una sola vez», §14.3 | **Corregido**: un solo `ModuleCard` sobre `Metric`; las cifras de un módulo viven en su página y las que cruzan módulos, en Inicio |
| 5 | Estilos escritos para una sola pantalla (`module-tile`, `module-card`, `tile--bare`, `icon-btn--iso`, `mod-glyph`, tonos `mod-*`) | §14.3, §14.9 | **Corregido**: se borraron |
| 6 | Cada página abría distinto: unas con título sobre el fondo, los módulos sin título y con el nombre en la pestaña; Documentos con pestañas negras; las vistas, a veces un control gris y a veces pestañas | §2, §6 | **Corregido (2026-10-09)**: anatomía universal en §6.1. Cabecera en todas; carpeta con el acento y las vistas como pestañas (`Folder tabs`). Falta Administración (aún usa `Dock`) |
| 7 | La bienvenida del primer inicio dibuja burbujas con los isotipos de color (`WelcomeArt.tsx`, `public/SVG/isotipo_*.svg`): es lo único que sigue usándolos | §3.6 | **Por decidir**: dejarlas (es identidad de marca en un panel propio) o pasarlas a los íconos de línea |
| 8 | Dentro de una página de módulo conviven la pestaña del color del módulo y etiquetas de varios colores (puestos, tipos de ingreso) | §3.4 «un solo color grande por carpeta» | **Por decidir**: las categorías son datos y llevan colores de la serie; confirmar con la prueba de pantalla |
| 9 | Valores sueltos (hexadecimales, escalas de Tailwind, estilos con color) en las pantallas | §14.4 | **Sin hallazgos**: `src/design.test.ts` los impide y pasa |
| 10 | Íconos: 104 `Icon`, 21 `Tile`, 15 `IconButton`, todos de línea; la única imagen suelta es el dibujo de la bienvenida | §3.6 | **Sin hallazgos** fuera del punto 7 |
| 11 | El riel de solo íconos no nombraba las secciones y, con 8 entradas, no cabía en ventanas bajas | §12 (riesgo anotado) | **Corregido**: barra superior con los nombres a la vista (§6) |
| 12 | Varios colores a la vez en una misma página (módulo de un color, puestos de otros, tarjetas de otros) | §3.4 «un solo color grande», §3.7 | **En prueba** en Inicio, Mi institución y Personal; falta aplicarlo a Proyectos, Beneficiarios, Instalaciones y Finanzas cuando se confirme |
