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
   - El modelo de español no tiene el signo «@»: un correo se leía «contacto(Dcasa…» y el **escáner no lo encontraba**. Con `eng+spa` se leen bien los correos en Arial, Times, Calibri y Verdana, se conservan los acentos y la confianza sube (página limpia: 87.9 → 91.7).
   - Cuesta 15 MB más y un 25 % más de tiempo por página.
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
8. **Topes.**
   - 100 páginas escaneadas por archivo; con más, se pide subirlo en partes o pedir el original.
   - 120 segundos por página.
   - 40 MB por archivo, como ya estaba.

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
  - leer las imágenes dentro de Word y Excel;
  - enderezar fotos giradas (`osd`);
  - una vista del texto leído en Documentos, para revisar las páginas con dudas.
