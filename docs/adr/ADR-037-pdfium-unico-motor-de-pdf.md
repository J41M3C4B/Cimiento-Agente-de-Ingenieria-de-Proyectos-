---
adr: 37
estado: aceptada
fecha: 2026-10-13
tema: [documentos]
reemplaza_en_parte: ["[[ADR-004-office-rust-python-plan-b]]", "[[ADR-035-leer-paginas-escaneadas-y-fotos-ocr]]"]
tags: [adr]
---
# ADR-037 · Pdfium como único motor de PDF

**Estado:** aceptada (2026-10-13, decisión de la institución), por bloques. **Bloque 1 (medición) hecho;** el 2 y el 3 pendientes.

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
- **Sin `pdfium.dll` no se lee ningún PDF.** Hoy un PDF con texto se lee aunque falte. El instalador siempre lo trae y en desarrollo lo baja `scripts/fetch-ocr.ps1`; el error se dice con las palabras de siempre («lo que lee no está instalado»).

## Bloques

1. **Medición** (hecho): la comparación de arriba.
2. **El texto de cada página con Pdfium** (`pdf.rs`), con el guion de corte y los párrafos. Las tablas siguen como hoy. Pruebas de convocatorias, Lector, búsqueda y expediente.
3. **Las tablas con Pdfium:** `pdf_rows` sobre las cajas de letra de Pdfium y `pdf_grid` sobre sus trazos. Al final se quitan `pdf-extract` y `lopdf`.

## Consecuencias

- Un solo motor para leer y dibujar PDF, el de Chrome, mantenido y rápido.
- Dos dependencias menos al terminar el bloque 3.
- Lo que hoy dice el ADR-004 sobre la lectura de PDF y la nota del ADR-035 «Pdfium dibuja cada página escaneada» quedan reemplazados en esa parte: Pdfium también lee el texto.
