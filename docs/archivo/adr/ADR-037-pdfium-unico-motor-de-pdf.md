---
adr: 37
estado: aceptada
fecha: 2026-10-13
tema: [documentos]
reemplaza_en_parte: ["[[ADR-004-office-rust-python-plan-b]]", "[[ADR-035-leer-paginas-escaneadas-y-fotos-ocr]]"]
tags: [adr]
---
# ADR-037 · Pdfium como único motor de PDF

**Estado:** aceptada (2026-10-13, decisión de la institución), por bloques. **Los tres bloques hechos** (medición, texto y tablas).

## Contexto

Hoy un PDF se lee con tres piezas:

| Pieza | Qué hace | Con qué |
|---|---|---|
| `documents/pdf.rs` | El texto de cada página | `pdf-extract` 0.12.1 |
| `documents/pdf_rows.rs` | Filas de tablas sin bordes, por la posición de cada letra | `pdf-extract` |
| `documents/pdf_grid.rs` | Tablas con bordes, por las líneas que dibuja la página | `lopdf` 0.42.0 |

Desde el ADR-035, **Pdfium** (`pdfium-render` 0.9.4 y el `pdfium.dll` de Chrome) ya viaja en el instalador para dibujar las páginas escaneadas. Pdfium también da el texto, la caja de cada letra y los trazos de la página: puede hacer las tres cosas.

## Decisión

**Pdfium lee el texto de todo PDF, con texto o escaneado.** `pdf-extract` y `lopdf` salen del programa cuando las tablas también se lean con Pdfium.

## Medición (bloque 1, 2026-10-13, en seco)

`cargo test compare_pdf_engines` lee cada PDF de una carpeta con los dos motores y compara, sin imprimir el texto. Corrido sobre 24 PDF públicos de muestra (`D:\Agente Proyectos\pdf_pruebas`, fuera del repositorio) y los fixtures de `fixtures/office`.

- **Las mismas letras.** En todos los archivos, los dos motores sacan exactamente el mismo número de letras (sin contar espacios), con los mismos acentos. La diferencia está en los espacios.
- **`pdf-extract` parte las palabras.** Con algunas fuentes mete espacios dentro de la palabra: «Thi s report outli nes the launch strategy», «devi ce», «i s», «sample-files. com». Pdfium las saca enteras.

  | Archivo | Palabras de una letra, `pdf-extract` | Pdfium | Palabras que coinciden (E→P) |
  |---|---|---|---|
  | sample-10-page | 34 | 1 | 76 % |
  | sample-5-page | 20 | 5 | 61 % |
  | image-doc | 14 | 4 | 87 % |
  | dev-example | 11 | 1 | 92 % |

  En los demás coinciden del 96 al 100 %. Una palabra partida estorba a la búsqueda, al Lector y **al escáner**: una CURP o un correo con un espacio en medio puede escapar a sus patrones (principio 5).
- **Pdfium es más rápido.** De 5 a 20 veces en casi todos. Un documento de 100 páginas escaneadas: `pdf-extract` tardó 24.7 s en descubrir que no tenía texto, Pdfium 29 ms. La primera llamada a Pdfium tarda ~1.3 s (carga la biblioteca); después es inmediata. Solo un archivo de 20 páginas fue más lento con Pdfium (569 ms contra 267 ms).
- **PDF con contraseña:** ninguno de los dos los abre (Pdfium dice `PasswordError`). Igual que hoy.
- **Escaneados:** los dos dicen que no hay texto en las mismas páginas.

## Lo que el bloque 2 debe cuidar

- **Guion de corte.** Pdfium marca con el signo U+0002 un guion al final del renglón (apareció en un archivo de 50 páginas). Se convierte en «-», como en la página.
- **Párrafos.** Pdfium separa renglones con un salto, pero no deja renglón en blanco entre párrafos; `pdf-extract` sí. El corte de fragmentos para la búsqueda (`documents/chunking.rs`) separa por renglón en blanco: con Pdfium, cada página sería un solo párrafo cortado por palabras. Los párrafos se rehacen con la distancia entre renglones (la caja de cada letra la da Pdfium), y se mide que la búsqueda (`cargo test core::archive::search`) y las convocatorias (`cargo test canonical`) no empeoren.
- **Sin `pdfium.dll` no se lee ningún PDF.** Antes un PDF con texto se leía aunque faltara. El instalador siempre lo trae y en desarrollo lo baja `scripts/fetch-ocr.ps1`; si faltara, el PDF sale como «no se pudo abrir».

## Bloques

1. **Medición** (hecho): la comparación de arriba.
2. **El texto de cada página con Pdfium** (`pdf.rs`), con el guion de corte y los párrafos. Las tablas siguen como hoy. Pruebas de convocatorias, Lector, búsqueda y expediente. *Hecho, ver abajo.*
3. **Las tablas con Pdfium:** `pdf_rows` sobre las cajas de letra de Pdfium y `pdf_grid` sobre sus trazos. Al final se quitan `pdf-extract` y `lopdf`. *Hecho, ver abajo.*

## Bloque 2 hecho (2026-10-13): el texto con Pdfium

- **`pdf::read_pdf` lee con Pdfium** letra por letra, con la caja de cada una.
  - Los renglones son los que da Pdfium.
  - El signo U+0002 se escribe «-».
  - Un renglón en blanco separa dos párrafos cuando la distancia entre dos renglones pasa de media altura del renglón más bajo, o cuando el siguiente sube en la página (otra columna). Se toma el más bajo porque bajo un título grande la distancia es chica comparada con el título, pero no con el texto; con el más alto, el título se pegaba al primer párrafo.
- **Medido otra vez sobre los 24 PDF de muestra:**
  - mismas letras que antes;
  - sin signos raros;
  - palabras de una letra (texto partido): de 34 a 1 en el peor archivo;
  - párrafos parecidos a los de antes (53 contra 64, 35 contra 51; `pdf-extract` partía de más las listas);
  - de 5 a 20 veces más rápido.
- **Un archivo con contraseña lo dice** (pedido de la institución: que nada termine en silencio).
  - Un PDF: Pdfium responde `PasswordError` → `PdfError::Protected` → `ReadError::Protected`.
  - Un Word o Excel guardado con contraseña no es un ZIP: Office lo guarda en su contenedor viejo (empieza con D0 CF 11 E0) con una parte llamada «EncryptionInfo». Ese par se reconoce antes de intentar leerlo.
  - En el Expediente (`Unreadable::Protected`) y en las convocatorias, la persona lee: «Ese archivo tiene contraseña y no lo podemos abrir. Guarde una copia sin contraseña (o pida una a quien se lo dio) y súbala otra vez.» Antes salía «dañado».
  - Un PDF que solo restringe imprimir o copiar (contraseña de propietario) se abre y se lee como cualquier otro.
- **Ya no hay segundo intento** cuando el texto no sale: antes, si `pdf-extract` fallaba, Pdfium contaba las páginas para leerlas como imagen. Ahora es el mismo motor, así que un archivo que Pdfium no abre está dañado.
- **Fixtures y pruebas:**
  - `fixtures/office/protegido.pdf` (`generar.py protegidos`);
  - `paragraphs_come_from_how_far_apart_the_lines_are`, `a_pdf_with_a_password_says_so`, `the_text_of_a_page_keeps_its_paragraphs`, `a_file_with_a_password_says_so` y el Expediente en `what_cannot_be_taken_says_why…`.
- **Lo que no mide este bloque:** la búsqueda y el Lector se prueban con textos ficticios (`.txt`), no con PDF. Su prueba real es subir los PDF de la institución.

## Bloque 3 hecho (2026-10-13): las tablas con Pdfium

- **Una pasada por página** (`pdf::read`), que saca tres cosas:
  - el texto;
  - las letras, con `pdf_rows::glyphs_of`: el origen, la caja, el tamaño y el ángulo de cada una (`Dir`); un espacio que Pdfium infiere va como letra sin ancho, justo después de la anterior;
  - las líneas, con `pdf_grid::rules_of_page`: los trazos pintados, siguiendo los formularios anidados con sus matrices.
- La lógica de tablas sobre `Glyph` y `Rule` no cambió.
- **Medido antes de quitar el motor viejo:** 605 filas de tabla en los 24 PDF de muestra y los fixtures. 601 iguales y 4 mejores: los espacios que el viejo se comía («Item 2 Item 3» en vez de «Item 2Item 3»).
- **Un hueco encontrado con un fixture nuevo** (`tabla-con-lineas.pdf`: una tabla dentro de un formulario anidado, sobre un fondo de página):
  - Pdfium cierra un rectángulo con su propio primer punto, y el rectángulo delgado relleno salía con seis puntos y no daba línea;
  - así dibuja Word los bordes de sus tablas;
  - las muestras no lo mostraban porque sus tablas usan líneas trazadas;
  - corregido.
- **Página mixta:** imágenes en la mitad o más de la página y menos de 200 letras (un encabezado impreso sobre un cuerpo escaneado) → se lee como imagen (`pagina-mixta.pdf`).
- **Fuera del programa:** `pdf-extract`, `lopdf`, el recolector de letras y el intérprete de contenido.
  - Las herramientas de medición quedan con Pdfium: `pdf_report`, `print_lines_of_a_page` y `count_tables_by_kind`.

## Consecuencias

- Un solo motor para leer y dibujar PDF, el de Chrome, mantenido y rápido.
- Dos dependencias menos al terminar el bloque 3.
- Lo que hoy dice el ADR-004 sobre la lectura de PDF y la nota del ADR-035 «Pdfium dibuja cada página escaneada» quedan reemplazados en esa parte: Pdfium también lee el texto.
