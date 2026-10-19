---
adr: 35
estado: aceptada
fecha: 2026-10-10
tema: [documentos]
reemplaza_en_parte: ["[[ADR-004-office-rust-python-plan-b]]", "[[ADR-011-ingesta-en-silencio-y-oficina-solo-lectura]]"]
reemplazado_en_parte_por: ["[[ADR-037-pdfium-unico-motor-de-pdf]]"]
tags: [adr]
---
# ADR-035 · Leer páginas escaneadas y fotos (OCR) con Tesseract, dentro del programa

**Estado:** aceptada (2026-10-10). Reemplaza lo que el ADR-004 («No se intenta OCR en v1») y el ADR-011 («OCR diferido») decían del OCR. Lo demás de esos ADR sigue igual.

## Contexto

Hasta ahora el programa solo leía PDF con texto. Un PDF escaneado o una foto se detectaba, pero no se leía:

- en el **expediente** (N4) el archivo quedaba como «Lo tenemos», sin texto;
- en **Proyectos** una convocatoria escaneada se rechazaba;
- en **Documentos** la ventana «Subir documento» solo tomaba texto pegado.

Las actas, constancias, poderes y muchas convocatorias que tienen las instituciones están escaneados o fotografiados. Sin su texto, ni la persona ni la IA pueden usarlos (Lector IA6, expediente hacia Proyectos N5).

## Decisión

Decisiones de la institución (2026-10-10): Tesseract dentro del instalador, precisión antes que rapidez, y las páginas leídas con dudas se guardan con aviso.

1. **Tesseract 5.5.3 viaja con el programa** (`src-tauri/resources/ocr`, en `bundle.resources`).
   - Se llama como programa aparte (`tesseract.exe`), sin compilarlo dentro: la compilación en Windows pide vcpkg y C++ y es frágil, para un resultado del mismo tamaño.
   - Solo se copian el ejecutable y las 33 bibliotecas que necesita; las herramientas de entrenamiento quedan fuera.
   - `scripts/fetch-ocr.ps1` lo descarga con huella SHA-256 y nunca se sube al repositorio: lo corre una vez quien clona el proyecto y antes de armar el instalador.
2. **Pdfium dibuja cada página escaneada** para Tesseract, que no abre PDF (`pdfium-render` 0.9.4 y `pdfium.dll` 7881 de bblanchon, la versión que lee esa biblioteca).
   - La página se dibuja a 300 ppp en gris y llega a Tesseract por su entrada estándar: **ninguna imagen se escribe en disco** (principio 6).
   - Un archivo que llega de la pantalla (Documentos, convocatorias) vive en una carpeta temporal solo mientras se lee.
3. **Modelo preciso, inglés primero y luego español** (`tessdata_best`, `-l eng+spa`).
   - El modelo de español no tiene el signo «@»: un correo se leía «contacto(Dcasa…» y el **escáner no lo encontraba**. Con `eng+spa` se leen bien los correos en Arial, Times, Calibri y Verdana y la confianza sube (página limpia: 87.9 → 91.7).
   - Cuesta 15 MB más y un 25 % más de tiempo por página.
   - *Corrección (2026-10-11):* no siempre conserva los acentos. Cuando elige el modelo de inglés para un renglón, lo lee sin acentos («situacion», «numero»). Medido en nueve muestras:
     - con `spa+eng`, la «@» se pierde en Arial, Calibri y Verdana, y en la factura;
     - con `eng+spa`, se pierden los acentos en un texto de dos columnas.

     Se queda `eng+spa`: proteger un correo (principio 5) pesa más que un acento, y la IA entiende «situacion».
4. **Qué se lee como imagen:**
   - las páginas de un PDF con menos de 20 caracteres de texto;
   - un PDF cuyo texto no se puede sacar (se intenta dibujarlo);
   - las fotos JPG y PNG.

   Word y Excel se leen como antes; sus imágenes, no.
5. **Las dudas se dicen.**
   - Tesseract da su confianza por palabra. La de la página es el promedio pesado por letras, y por debajo de 70 la página queda «con dudas».
   - Una página con dudas se lee otra vez con el umbral adaptativo de Leptonica (`thresholding_method=1`) y se queda la lectura más segura. Ese umbral rescata copias desteñidas (de nada a 65.6), pero en páginas limpias confunde la «@»; por eso es solo un segundo intento.
   - La página con dudas **se guarda** y se avisa cuáles son: en la ventana del expediente, en la línea del documento y en Documentos.
6. **Qué se guarda.**
   - Solo el texto limpio, tras el escáner como cualquier texto.
   - En `document`, qué páginas se leyeron de imagen y cuáles con dudas (`picture_pages`, `doubtful_pages`, migración 0029).
7. **Sin congelar el programa.**
   - El archivo se lee **sin tomar la base de datos**: el comando lee primero y guarda después.
   - Las páginas se leen de dos a cuatro a la vez, un hilo cada Tesseract.
   - La pantalla oye el avance por el evento `reading-progress`: «Leyendo las páginas escaneadas: 3 de 12».
   - El último archivo leído como imagen se recuerda **en memoria**, por la huella de sus bytes: al volver de la cuarentena con «Tapar y seguir» no se lee otra vez.
8. **Tablas y correos mal leídos** (2026-10-11, tras probar una factura real):
   - Tesseract lee la página renglón por renglón (`--psm 4`). El modo automático (3) partía la tabla de la factura en un bloque por columna. Un texto a dos columnas salió igual con los dos modos.
   - Dos palabras de un renglón separadas por más de 1.5 veces su altura son celdas distintas y se unen con « | », como las filas de un PDF con texto (`pdf_rows`). Ejemplo: «Arroz | 10 | $28.00 | $280.00».
   - Aun con `eng+spa`, la factura real traía «infoGorlandojuan.es» (la «@» leída como «G») y el correo no se tapó. En un texto leído de imagen, el escáner toma como posible correo una dirección con un dominio conocido pegada a otra palabra, si su renglón habla de correo o un signo ocupa el lugar de la «@» (`RegexScanner::reading_pictures`). Es un aviso: la persona decide.
9. **Topes.**
   - 100 páginas escaneadas por archivo; con más, se pide subirlo en partes o pedir el original.
   - 120 segundos por página.
   - 40 MB por archivo, como ya estaba.

### Precisiones con documentos reales (2026-10-13) — **las tres hechas** (la 3, en el ADR-038)

Al probar la aplicación con documentos reales salieron tres problemas. Se midieron en el equipo de desarrollo, con los mismos parámetros de la aplicación, sin guardar ni imprimir el texto (`cargo test reader_on_a_file`, que ahora también dice, por página, qué datos protegidos encuentra el escáner, sin sus valores). Nada de esos documentos entró al repositorio.

**1. La letra chica se lee mal: hay que agrandar la imagen, no entrenar el modelo.**

- Una captura de pantalla de la ficha de un registro público (854×764 a 96 ppp, letra de unos 7 píxeles) perdía un número completo y un título, juntaba «rn» en «m» («Hemández», «Intemado») y cambiaba «t» por «l» («alención»).
- La misma imagen agrandada con Lanczos:

  | Escala | Confianza | Número y título | Palabras de prueba bien / mal |
  |---|---|---|---|
  | ×1 (como hoy) | 83.0 | se pierden | 9 / 13 |
  | ×2 | 94.1 | se leen | 18 / 3 |
  | ×3 | 93.3 | se leen | 19 / 2 |
  | ×4 | 93.6 | se leen | 20 / 1 |

- **Entrenar Tesseract no se hace por ahora.** Ya usamos el modelo más preciso (`tessdata_best`). Los conjuntos de datos públicos (inglés, letra a mano, recibos) no se parecen a un acta mexicana, y ajustar con ellos puede empeorar lo que ya se lee bien. Si algún día hace falta, será con texto ficticio generado por nosotros y después de agotar lo demás.
- Lo que no arregla ni agrandar ni entrenar: «I.A.P.» se lee «1.A.P.» en todas las escalas, porque en Arial la «I» y la «l» son el mismo dibujo. Eso lo arreglan reglas en el código para siglas y campos con formato (CURP, RFC, montos).
- Veinte fotos de recibos en inglés, guardadas fuera del repositorio, sirven solo como prueba difícil de fotos (arrugadas, chuecas, con fondo). Agrandarlas ayuda menos (un total leyó «194,02» por 144.02): las fotos necesitan además recortar, enderezar y contraste. No sirven para entrenar.
- **Propuesta:** Tesseract ya da la altura de cada palabra. Si la altura típica queda por debajo de unos 20 píxeles, la imagen se agranda para que la letra mida unos 30 y se lee otra vez. La resolución de una foto no se adivina: se mide en la primera lectura, y una página bien leída no se lee dos veces. Va con pruebas: una captura chica y una foto chica ficticias, con su texto esperado.
- **Hecho (2026-10-13, `documents/ocr.rs`):**
  - **Cómo se mide la letra:** el cuartil inferior de las alturas de las palabras de dos o más letras, con al menos 5 palabras. No la mediana: Tesseract da a algunas palabras la altura de todo su renglón (32 píxeles en una foto cuya letra mide 9), nunca menos que la suya. En los fixtures normales queda en 22 a 25 píxeles y no se relee nada.
  - **Cuánto se agranda:** hasta que la letra mida 30 píxeles, a lo más ×4, y ningún lado pasa de 5000 píxeles. Menos de ×1.25 no se hace. Se queda la lectura más segura; si sigue «con dudas», el segundo intento con umbral adaptativo se hace sobre la imagen ganadora.
  - **Una foto chica donde no se halla ni una palabra** no deja medir nada: Tesseract no encontró ninguna en la foto ficticia de un aviso tomado de lejos. Si la imagen mide menos de 1200 píxeles por su lado mayor, se agranda ×3 de todos modos. Una página de PDF (2550×3300 a 300 ppp) o una foto de celular son más grandes: una página en blanco no se lee dos veces.
  - **El agrandado es nuestro (Lanczos de tres lóbulos, en gris).** El crate `image` (0.25.10, solo JPEG, PNG y PGM, sin hilos) solo abre la imagen. Su agrandado guarda cuatro números por pixel a medio camino: una página de PDF ocuparía unos 200 MB por cada página leída a la vez. Lo transparente de un PNG se ve blanco, como lo ve Tesseract.
  - **Fixtures** (`fixtures/office/generar.py chicas`):
    - `captura-chica.png`: ficha de un registro, Arial de 10 píxeles. Tal como llega pierde el título, lee «8213» por «48213», «Intemado», «Vale» y el correo sin «@». Agrandada ×4 lo lee todo, correo incluido.
    - `foto-chica.jpg`: aviso borroso, un poco girado, en JPEG. Tal como llega, cero palabras; agrandada ×3 se lee completa.
    - Sin el agrandado, sus dos pruebas fallan.

**2. Las actas constitutivas se rechazan como «lista de personas».**

| Acta real | Páginas (escaneadas) | Lo que encuentra el escáner | Regla que la rechaza |
|---|---|---|---|
| A | 37 (37) | 3 CURP y 4 RFC de persona, en 3 páginas | Más de 5 datos bloqueantes en todo el archivo (`ROSTER_BLOCK_LIMIT`) |
| B, testada | 69 (67) | 1 RFC y 1 NSS | Un renglón con «NOMBRE» y «EDAD», en una sola página |

- La primera regla cuenta **datos**, no **personas**: la CURP y el RFC de un mismo socio suman dos, así que un acta con 3 o 4 socios ya rebasa el tope.
- La segunda busca «EDAD» **dentro de palabras**: «soci**edad**», «propi**edad**», «activid**ad**» la activan. El renglón de B es una frase de 119 letras con dos comas, no el encabezado de una tabla.
- **Propuesta** (`RegexScanner::looks_like_roster`):
  - palabras completas («EDAD» sí, «SOCIEDAD» no);
  - un encabezado de lista es un renglón de celdas cortas, no una frase;
  - contar **personas distintas**: la CURP y el RFC de una persona empiezan con las mismas 10 letras;
  - en la duda, **cuarentena** («Tapar y seguir») en lugar de rechazo. Un acta debe poder guardarse con los datos de los socios tapados; el rechazo completo queda para lo que de verdad es una lista.
- Pruebas: un acta ficticia con varios socios y la palabra «sociedad» entra a cuarentena; una lista ficticia con encabezado y una con 6 o más personas distintas siguen rechazadas.
- **Decidido de otra forma (2026-10-13):** la institución descartó afinar la regla. Hasta una constancia de situación fiscal salía rechazada, y ningún tope de datos personales le atina a papeles que por fuerza los traen. **En el Expediente el escáner tapa todo y guarda: sin rechazo y sin cuarentena.** Ver ADR-036, «Precisión del 2026-10-13». La regla queda solo para las convocatorias de Proyectos, sin cambios.

**3. Leer un acta larga ocupa demasiado el equipo.**

- Medido en un Ryzen 5 3400G (4 núcleos, 8 hilos, 16 GB), con 4 páginas a la vez:
  - acta A: 37 páginas en 171 s;
  - acta B: 67 páginas en 216 s. 20 de ellas quedaron «con dudas» y se leyeron dos veces; casi todas son pares (reversos con sello o franjas del testado, por confirmar).
- En un equipo típico de una institución (2 núcleos, 4 a 8 GB) se calculan **8 a 10 minutos por acta** con el procesador muy ocupado.
- **La nube no es opción para el OCR:** el escáner debe revisar el texto antes de que algo salga del equipo (principio 5), y una imagen no se puede revisar sin leerla primero.
- **Propuestas, de la más barata a la más grande:**
  1. Tesseract con **prioridad baja** (`BELOW_NORMAL_PRIORITY_CLASS`): Windows le da el procesador solo cuando nadie más lo usa. La computadora no se congela; la lectura tarda más.
  2. **Leer en segundo plano y guardar el avance por página**: la persona sigue trabajando y puede cerrar el programa sin perder lo leído.
  3. **Medir `tessdata_fast`** contra `tessdata_best`, ya con la imagen agrandada. Hoy cada renglón pasa por dos modelos precisos (`eng+spa`).
  4. **No releer una página casi vacía** aunque salga «con dudas»: una segunda lectura no le gana nada.

- **Hecho (2026-10-13, ADR-038):**
  - Tesseract con prioridad baja;
  - el Expediente lee en segundo plano (sin reanudar al cerrar el programa);
  - una página casi sin tinta, medida contra su propio papel (la desteñida del fixture tiene 0.46 %, la limpia 1.4 %), no se relee.
  - `tessdata_fast` no se adoptó: la institución eligió precisión antes que rapidez (2026-10-10), y la prioridad baja y el segundo plano quitan el congelamiento.

**Orden propuesto:** primero la regla de «lista de personas» (hoy impide guardar cualquier acta real); después prioridad baja y agrandado de letra chica; al final la medición de `tessdata_fast` y la lectura en segundo plano. *La institución pidió primero el agrandado (2026-10-13), y ya está hecho.*

## Mediciones (fixtures ficticios, 2026-10-10, un hilo)

| Archivo | Lectura | Confianza | Tiempo |
|---|---|---|---|
| `acta-escaneada.pdf` página 1 (limpia, 200 ppp) | completa, con correo | 91.7 | ~2.3 s |
| `acta-escaneada.pdf` página 2 (desteñida, borrosa y sucia) | con errores, «con dudas» | 65.6 | se lee dos veces |
| `constancia-foto.jpg` (foto girada 1.5°) | completa | 96 | 0.8 s |

El instalador crece unos 70 MB sin comprimir: 49 MB de Tesseract y sus bibliotecas (26 de ellos son `libstdc++`), 29 MB de los dos modelos y 6 MB de Pdfium. Comprimido, unos 30 MB.

## Consecuencias

- Expediente, Documentos y convocatorias leen escaneos y fotos. Documentos acepta PDF, Word, Excel y fotos sin pedir que se pegue el texto.
- **Límites que la persona debe saber:**
  - no se lee letra a mano;
  - las fotos chuecas o con sombra se leen mal;
  - sellos y firmas no salen o salen como basura.
- **Riesgo que queda para el escáner:** un dato mal leído (una CURP con «O» en lugar de «0», un correo borroso) puede escapar a sus patrones. El modelo `eng+spa` corrige el caso más común (la «@»), y las páginas con dudas se avisan; no hay más protección por ahora.
- **Licencias:**
  - Tesseract y los modelos: Apache-2.0.
  - Leptonica: BSD.
  - Pdfium y sus binarios: BSD y MIT.
  - Bibliotecas LGPL (iconv, intl, unistring, idn2): van como DLL aparte, reemplazables.

  Las licencias viajan en `resources/ocr/licenses`.
- Tauri da la carpeta de recursos con el prefijo `\\?\` de Windows, y Tesseract no abre sus modelos así: `ocr::set_home` lo quita. Las pruebas no lo ven porque usan la carpeta del código; se encontró al revisar en el WebView real.
- Solo Windows: el script baja los binarios de Windows. Para otro sistema hará falta su propio Tesseract y Pdfium.
- Queda para después:
  - ~~leer las imágenes dentro de Word y Excel~~: hecho el 2026-10-13 (ADR-038). Las imágenes de `word/media` y `xl/media` de 600 píxeles o más por su lado mayor (una hoja pegada; un logo no) se leen con el mismo OCR, como páginas al final del texto (`word-con-imagen.docx`);
  - ~~enderezar fotos giradas (`osd`)~~: hecho el 2026-10-13. Una página con dudas y con tinta se revisa con el modelo de orientación (`osd.traineddata` de `tessdata_best`, en `fetch-ocr.ps1` con su huella) y, girada, se endereza y se lee otra vez (`acta-girada.pdf`);
  - una vista del texto leído en Documentos, para revisar las páginas con dudas.
