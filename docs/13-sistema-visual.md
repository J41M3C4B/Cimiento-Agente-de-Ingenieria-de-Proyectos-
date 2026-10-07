# 13 · Sistema visual

**Estado:** propuesta para validar con un prototipo (ver «Validación»). Reemplaza al sistema descrito en el comentario de `src/index.css` si se aprueba.

Este documento recoge la dirección visual tomada de las referencias que la institución eligió (un tablero de tareas en bandejas blancas sobre un marco gris, con color sólido y tranquilo) y la traduce a reglas que se puedan aplicar a Cimiento. Es una especificación de diseño: no cambia flujos, datos ni textos (`docs/08-estilo-redaccion.md` sigue mandando).

## 1. Idea

Cimiento acompaña a personas que dan asistencia social, con poco tiempo y poca experiencia con programas. La interfaz debe sentirse **amable, ordenada y viva**, no como una herramienta de oficina.

Tres ideas sostienen todo lo demás:

1. **Color con significado.** Cada cosa que importa tiene su color sólido y lo conserva en toda la app. El color ayuda a reconocer dónde se está y qué sigue; las monjas que probaron las referencias lo señalaron como lo que más les facilita el uso. Nunca es el único dato: siempre va con palabras.
2. **Bandejas sobre un marco.** Todo vive dentro de un marco redondeado claro; dentro, cada tema es una bandeja blanca, grande, con mucho aire. Una pantalla es un tablero de bandejas, no una página larga.
3. **Una cosa a la vez.** Cada bandeja responde una pregunta. Lo que se puede hacer ahora es lo más grande y lo más oscuro de la pantalla.

## 2. Superficies

Tres niveles, de atrás hacia adelante. La profundidad la da el cambio de tono y una sombra muy suave, no los bordes.

| Nivel | Qué es | Claro | Oscuro | Radio |
|---|---|---|---|---|
| 0 · Lienzo | El fondo de la ventana | `#E4E5E8` | `#0E0F12` | — |
| 1 · Marco | La hoja grande que contiene todo | `#F2F2F4` | `#17181C` | 32 px |
| 2 · Bandeja | Cada módulo | `#FFFFFF` | `#1F2025` | 28 px |
| 3 · Interior | Elementos dentro de una bandeja (burbujas, calendario, campos) | `#F6F6F8` | `#26272D` | 18–20 px |

Sombras:

- Bandeja: `0 1px 1px rgb(20 20 30 / .03), 0 10px 30px -12px rgb(20 20 30 / .12)`.
- Marco: `0 30px 80px -30px rgb(20 20 30 / .25)`.
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

Ocho colores, saturados pero tranquilos (ni neón ni pastel). Cada uno trae el color del texto que se pone encima.

| Token | Claro | Sobre él | Significa en Cimiento |
|---|---|---|---|
| `sun` | `#F7D046` | `ink` | Lo que está en curso; encabezado de la bandeja principal |
| `sky` | `#4B7BEC` | blanco | Convocatoria; enlaces y selección |
| `leaf` | `#34B26B` | blanco | Listo, confirmado, correcto |
| `amber` | `#F3994B` | `ink` | Falta algo, atención |
| `rose` | `#EB5F9C` | blanco | Revisión |
| `violet` | `#7F6DF2` | blanco | Personas que atendemos |
| `teal` | `#2DB4A6` | blanco | Personal |
| `red` | `#EE5753` | blanco | Error, borrar, bloquear |

En modo oscuro los mismos tonos suben un 6 % de luminosidad y se mantienen los textos de encima.

### 3.3 Etiquetas sólidas y tintes

Las etiquetas de estado y de categoría son **sólidas**, del mismo color vivo que el resto de la interfaz; un fondo pastel con texto oscuro se ve apagado junto a los avatares, íconos y bandejas de color. El texto de encima sigue la tabla de 3.2: `ink` sobre `leaf`, `teal`, `amber`, `sun`, `rose` y `red` (todas ≥ 4.8:1), y blanco sobre `sky` y `violet`, que para eso se oscurecen un 14 % en la etiqueta (4.8:1). En modo oscuro los acentos son más claros y llevan `ink` oscuro.

El **tinte** (acento al 18 % sobre la bandeja, texto en el acento mezclado al 58 % con tinta) queda solo para datos informativos de bajo énfasis, como las etiquetas de contexto «Convocatoria NMP 2026» o «Gastos de inversión». Si algo es estado, es sólido.

### 3.4 Reglas de uso

- **El color nombra.** Un color pertenece a un significado y no se reutiliza para otro. La tabla de arriba es la lista completa.
- **Los pasos del proyecto tienen su color** (se ven en el avance, en las etiquetas y en la bandeja principal):

  | Paso | Color |
  |---|---|
  | 1 · Convocatoria | `sky` |
  | 2 · Diagnóstico | `violet` |
  | 3 · Objetivo | `teal` |
  | 4 · Redacción | `sun` |
  | 5 · Revisión | `rose` |
  | 6 · Listo | `leaf` |

- **Cada proyecto conserva el color que su dueña le puso** (ya existe en la base, `project.color`); aparece en su tarjeta y en su punto del menú.
- **Las personas** (personal, beneficiarias) se muestran con iniciales sobre un color sólido de la serie `sky, violet, teal, sun, rose, leaf, amber`, siempre el mismo para el mismo nombre. Solo en este equipo; no sale en documentos ni a la IA (`docs/03-gobernanza-datos.md`).
- **Estados siempre con texto:** «Listo», «Por confirmar», «Mal estado». Etiqueta con tinte, no solo punto.
- **Un solo color grande por pantalla.** La bandeja principal lleva el color de su paso; el resto es blanco y neutro. Más de un bloque de color sólido a la vez cansa.
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
- Radios: marco 32 · bandeja 28 · interior 18–20 · campo 14 · etiqueta y botón de ícono: círculo completo (999).
- Altura de controles: 40 (botón), 44 (botón circular de ícono), 36 (etiqueta y campo compacto). Botón principal de una bandeja: 44.
- Íconos: trazo de 1.8 px (los actuales de `icons.tsx`), 18–20 px.

## 6. Estructura de pantalla

```
┌ Lienzo ─────────────────────────────────────────────────────────┐
│ ┌ Marco ──────────────────────────────────────────────────────┐ │
│ │ [logo] [cápsula: dónde estoy · avance]        (🔔)(🔍)(☰)(C)│ │
│ │ (⌂)  ┌ bandeja ┐ ┌ bandeja ┐ ┌ bandeja con pestaña ───────┐ │ │
│ │ (▣)  │         │ │         │ │                            │ │ │
│ │ (♜)  └─────────┘ └─────────┘ └────────────────────────────┘ │ │
│ │ (▤)  ┌ bandeja ──────┐ ┌ chat ────────────┐ ┌ cifra ──────┐ │ │
│ │ ...  └───────────────┘ └──────────────────┘ └─────────────┘ │ │
│ │ (⏻)                                                          │ │
│ └──────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

- **Marco:** ocupa la ventana con 16 px de margen; contenido a 1440 px como máximo.
- **Barra superior:** a la izquierda el logo y una **cápsula** (píldora blanca) que dice dónde se está: proyecto, paso y su avance en seis puntos con el color de cada paso. A la derecha, botones circulares con borde de 1 px: avisos, buscar, menú y la institución.
- **Riel izquierdo:** botones circulares de 44 px, en columna: Inicio, Mis proyectos, Mi institución, Documentos; separador; Ayuda automática, Seguridad; al fondo Ayuda y **Bloquear en rojo**. El activo va relleno de `ink`. Con etiqueta al pasar el cursor (tooltip a la derecha), no desplegable.
- **Cuadrícula:** 12 columnas, separación de 16. Las bandejas ocupan 3, 4, 5, 6 o 8 columnas; en pantallas angostas (< 900 px) pasan a una columna y el riel se vuelve una fila horizontal.
- **Bandeja con pestaña:** el título vive en una pestaña recortada en la esquina superior izquierda (esquinas cóncavas); sirve para bandejas con un tema («Mi institución», «Resumen del proyecto»). La pestaña activa de una barra de pestañas inferior se levanta en el color de la bandeja.
- **Bandeja principal con encabezado de color:** una banda `sun` (o el color del paso) con título y acción, y el cuerpo blanco encima con esquinas superiores de 24.

## 7. Componentes

- **Botón principal:** fondo `ink`, texto blanco, 44 px, radio 999. Uno por bandeja como máximo. Secundario: blanco con borde de 1 px; terciario: texto.
- **Botón circular de ícono:** 44 px, blanco, borde 1 px `line`; activo en `ink`; peligro en `red` sólido.
- **Cápsula:** píldora blanca de 52 px de alto con estado, fecha y avance.
- **Etiqueta (tag):** fondo sólido del acento, texto según 3.3, radio 999, 28 px de alto, 13 px, peso 700. Variantes: `soft` (tinte) para contexto y `line` (borde de 1.5 px, sin relleno) para datos neutros como «Sin redactar». Siempre con palabras.
- **Casilla:** 22 px, radio 7; marcada = `ink` con palomita blanca; el texto hecho se tacha y baja a `ink-3`.
- **Avatar de iniciales:** círculo de 36 px (28 en filas densas) de color sólido; pila de avatares solapados 10 px con anillo de 2 px del color de la bandeja.
- **Tarjeta de persona:** interior (nivel 3) con avatar, nombre, dato secundario y estado a la derecha.
- **Casilla para agregar:** rectángulo con borde discontinuo de 1.5 px, radio 20, con «+»; sustituye a los botones «Agregar» sueltos.
- **Cifra de bandeja:** número a 72 px peso 300 sobre un interior, con su etiqueta arriba y su explicación abajo.
- **Calendario:** interior con día de la semana, mes y la fecha en cifra grande. Se usa para el cierre de la convocatoria.
- **Chat:** mensajes de la persona en burbuja de interior alineada a la derecha con su avatar; los del asistente en burbuja de interior a la izquierda con avatar `ink` «AI». Caja de escribir: interior con campo, botón de micrófono (si existe) y envío circular `ink`.
- **Barra de pasos:** seis puntos o píldoras con el color de su paso; lo hecho lleno, lo actual con anillo, lo que sigue vacío con el color al 25 %.
- **Barra de composición:** segmentos continuos de 8 px de alto con los colores de las fuentes (ingresos, avance).
- **Tabla:** sin cabecera rellena; filas de 52 px con línea fina; la primera columna en peso 700; etiquetas de color para la categoría.
- **Formulario (dentro de una ventana):**
  - Secciones con etiqueta en mayúsculas pequeñas y una línea fina; los campos van en dos columnas (una en pantallas angostas) y los largos ocupan las dos.
  - Campo: 48 px de alto, radio 14, borde de 1.5 px `line`; al enfocarlo, borde `sky` y aro de 4 px al 45 %. Etiqueta arriba en peso 700; el asterisco rojo marca lo obligatorio.
  - Selector: mismo campo con flecha a la derecha y «Seleccionar» en `ink-3` mientras está vacío. Dinero: «$» fijo a la izquierda y la unidad («al año») a la derecha; solo acepta dígitos.
  - Interruptor: 48 × 28, verde cuando está activo.
  - Opciones cortas (estado, sí/no): **píldoras seleccionables** con el color del estado; la elegida se rellena de sólido. Opciones con explicación (tipo de institución): **tarjetas** con círculo de selección; la elegida lleva borde `ink` y fondo de interior.
  - Error: el campo se marca en `red`, el mensaje va debajo con ícono, en palabras de la persona («Escriba el nombre de la persona.»), y el foco pasa al primer campo con error. El mensaje se quita al escribir.
  - Pie con «Cancelar», «Guardar y agregar otra» (solo al agregar) y «Guardar» o «Guardar cambios» como botón principal. `Esc` o tocar fuera cierran; el foco vuelve al botón que abrió la ventana.
  - Al guardar, un aviso breve abajo («Persona agregada») y las cifras de la ficha se actualizan al momento.
- **Ventana (modal):** bandeja de 28 con título de 18, secciones con etiqueta en mayúsculas pequeñas y botones al pie; fondo atenuado.

## 8. Movimiento

- Transiciones de 160–220 ms con `ease-out` en color, sombra y posición. Nada rebota.
- Entrada de mensajes: aparecen 8 px más abajo y suben (ya existe `anim-rise`).
- Pestañas y cambios de pantalla: fundido de 150 ms; sin desplazamientos largos.
- Se respeta `prefers-reduced-motion`.

## 9. Accesibilidad y uso real

- Contraste: texto normal ≥ 4.5:1, texto grande y componentes ≥ 3:1. El texto sobre `sun` y `amber` es `ink`; sobre los demás, blanco.
- Nunca solo color: cada estado lleva palabra.
- Objetivo de toque ≥ 40 px; foco visible con aro de 3 px `sky` al 40 %.
- Densidad: se prefiere menos bandejas con más aire. Si una bandeja necesita más de siete filas, se pagina o se abre en ventana.
- Modo claro y oscuro con los mismos tokens; el claro es el principal.

## 10. Aplicación a las pantallas de Cimiento

| Pantalla | Estructura |
|---|---|
| **Inicio** (nueva) | Bandeja principal del proyecto en curso (encabezado `sun`, lo que falta confirmar como lista con casillas), recordatorio con el cierre de la convocatoria, bandeja «Mi institución» con pestaña y personal, y abajo resumen del proyecto + asistente + cifra de pendientes |
| **Mis proyectos** | Cada proyecto es una bandeja con su color, avance en puntos y botón principal «Continuar»; «Nuevo proyecto» como casilla discontinua |
| **Mi institución** | Una sola bandeja de cabecera en dos mitades: a la izquierda la identidad, sin ícono de perfil, en tres alturas: «Mi institución» como etiqueta pequeña arriba, el nombre (32 px) y la misión centrados verticalmente con mucho aire entre ellos, y la etiqueta de estado con «Editar» pegadas al fondo; a la derecha un grupo de 2×2 con las cifras en interiores (personas `violet` con barra de ocupación, personal `teal`, nómina `sun`, cuotas `leaf`), cada una con su ícono en cuadro de color, etiqueta, cifra de 30 px y detalle. Debajo, pestañas recortadas; datos en bandeja con filas; ingresos con barra de composición; tablas con avatares y etiquetas. Por debajo de 1000 px la bandeja pasa a una columna y por debajo de 520 px el grupo también |
| **Espacio de trabajo** | Chat grande al centro en su bandeja; panel derecho en otra bandeja con partes del proyecto y su estado en etiquetas; «Pasar a la revisión» como botón principal |
| **Ventanas** | Igual que hoy, con el estilo de bandeja |

## 11. Cambios frente al sistema actual

| Hoy | Propuesta |
|---|---|
| Lienzo gris + una hoja blanca + barra superior de texto | Marco claro redondeado con riel de íconos y cápsula superior |
| Casi sin color (tinta + un azul) | Ocho acentos sólidos con significado fijo |
| Inter | Plus Jakarta Sans |
| Pestañas segmentadas | Pestañas recortadas en la bandeja y barra inferior con pestaña levantada |
| Etiquetas apagadas | Etiquetas con tinte al 16 % y texto oscuro del acento |
| Radios 10 / 20 | 14 / 20 / 28 / 32 |

## 12. Riesgos y decisiones abiertas

- **Pestañas recortadas** (esquinas cóncavas) necesitan CSS propio y `drop-shadow`; se resuelven con un componente `Tray` que concentra el truco. Riesgo bajo.
- **Más color puede cansar.** La regla de «un color grande por pantalla» es lo que lo evita; hay que revisarla con las usuarias.
- **Riel de íconos:** sin etiqueta visible cuesta reconocer las secciones. Mitigación: tooltip inmediato y etiqueta bajo el ícono activo. Decidir en la prueba con las monjas.
- **Avatares de color para personas:** vuelven a aparecer nombres de personal en pantalla; es lo mismo que hoy (datos locales) pero hay que mantener que no salgan en documentos.
- **Fuente:** `@fontsource-variable/plus-jakarta-sans` se empaqueta con la app (sin red); confirmar el peso extra (~100 KB).
- **Lienzo y marco en ventana pequeña:** por debajo de 900 px el riel pasa a fila y las bandejas a una columna; verificar en la ventana real de Tauri.

## 13. Validación

1. Prototipo navegable de tres pantallas (Inicio, Mi institución, Asistente) para ver el sistema completo antes de tocar la app.
2. Si se aprueba, se aplica por capas: tokens y fuente → marco, riel y cápsula → bandejas y componentes → pantalla por pantalla, con las 95 pruebas verdes en cada paso.
3. Prueba corta con dos o tres personas de la institución: ¿encuentran «Continuar»?, ¿entienden qué significa cada color?
4. Al aprobarse se registra como ADR (`adr/ADR-025-identidad-visual.md`) y este documento deja de ser propuesta.
