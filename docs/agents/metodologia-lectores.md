# Metodología de los Lectores: leer un archivo y llenar desde él

Cómo cualquier cajón de documentos (Expediente, Personal, Beneficiarios, Instalaciones, Finanzas, Donantes) lee un archivo y llena sus formularios con lo que dice. El Expediente del núcleo es la referencia: un módulo nuevo copia esta forma y no inventa otra.

Decisiones de origen: [ADR-038](../adr/ADR-038-politica-unica-de-llenado-desde-documentos.md) (la política), [ADR-036](../adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) (cajones), [ADR-037](../adr/ADR-037-pdfium-unico-motor-de-pdf.md) (PDF) y [ADR-035](../adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) (OCR).

## 1. El camino de un archivo, siempre el mismo

1. **Leer** (`documents::reading`), en segundo plano. La persona sigue trabajando y un aviso dice cómo terminó, también si hubo error.
   - **PDF:** Pdfium saca el texto, los párrafos, las filas de las tablas y el guion de corte.
   - **Páginas como imagen:**
     - una página sin texto (menos de 20 letras), o mixta (imágenes en la mitad o más y menos de 200 letras), se lee con OCR;
     - un archivo con contraseña lo dice, nunca sale como «dañado».
   - **Word y Excel:** por sus partes; si tienen contraseña, lo dicen.
   - **Fotos:** con OCR.
2. **Reconocer el tipo** (`reader::kinds`): por sus palabras, sin IA.
   - Si el archivo se puso en el lugar de otro tipo y no menciona el suyo, se guarda, pero **no llena nada** y se avisa qué parece (`kinds::looks_like_other`).
3. **Reglas por tipo** (`reader::rules`): cada tipo sabe dónde escribe cada dato.
   - Cada dato encontrado (`Found`) lleva su campo, su valor, la página, la línea de donde salió y si se calculó (origen `computed`).
   - Lo que la regla no encuentra, no lo adivina.
4. **La política de llenado** (abajo). Es una sola y vive en un solo lugar.
5. **El escáner**:
   - tapa lo que se guarda del texto;
   - el Lector lee **antes** de tapar, por eso un dato propio protegido (el RFC de la institución, un folio) llega a su campo sin que su valor quede en el texto guardado.

## 2. La política de llenado

| Situación | Datos propios del documento (su ventana: emisión, vigencia, número) | Datos de otras ventanas («Mi institución») |
|---|---|---|
| El campo está vacío | Se escriben solos, con su origen (`document`, o `computed` si se calcularon) y la página | Llegan como **sugerencia** con «Usar»; nada se guarda sin el clic |
| El campo ya tiene un valor que escribió la persona | Nunca se tocan | Nada: ni sugerencia ni aviso |
| El campo se llenó con el archivo anterior y se reemplaza el archivo | Se leen del archivo nuevo | Lleno: nada |
| La persona rechazó antes ese valor para ese campo | — | No vuelve a sugerirse, ni con otro archivo (se recuerda por su huella SHA-256, sin guardar el valor) |
| Ya espera una sugerencia de otro documento | — | Gana el documento con más autoridad sobre ese campo; a igual autoridad, el más nuevo |
| El archivo parece otro tipo de documento | Nada | Nada; se avisa qué parece |
| La persona guarda el campo | — | Las sugerencias de documentos que esperaban para él se descartan solas |
| Se quita o borra el archivo | Lo leído se queda, con su origen | Las sugerencias pendientes se van; las rechazadas quedan solo como huella |

Lo que propone la IA (el Capturista) **no** sigue las reglas de «campo lleno» ni de «rechazado»: la persona le pidió ese cambio en el chat.

**Dónde vive en el código** (el Expediente):

| Pieza | Qué hace |
|---|---|
| `core::agents::proposals::propose` (`from_a_document`) | El único punto por donde pasa toda sugerencia: campo lleno, rechazado antes, autoridad |
| `proposals::discard_filled` | Descarta, al guardar, las sugerencias de los campos que quedaron llenos |
| `core::records::write_own` y `core_record_field` | Escriben los datos propios y su origen por campo; nunca sobre lo de la persona |
| `core::reader::authority` | La tabla de quién manda en cada campo |
| `core::reader::kinds::looks_like_other` | El archivo en el lugar equivocado |
| `core::records::background` | Una lectura por documento a la vez; el evento `record-read` dice cómo terminó |

## 3. Lo que entrega cada módulo para su cajón

1. **Sus tipos de documento:** código, clase y las frases que los reconocen, sin acentos y en minúsculas.
2. **Sus datos propios por documento:** las columnas de su ficha y una tabla de origen por campo como `core_record_field`, con `origin` y `source_ref`, que se queda aunque se borre el archivo.
3. **Reglas por tipo, en Rust y sin IA.**
   - Cada valor lleva su página y su línea; un dato protegido no guarda la línea.
   - Tolerancia a lo que lee mal una imagen en datos con formato: dígitos confundidos con letras (O→0, I/l→1, S→5, B→8), siglas («1.A.P.»).
4. **Su tabla de autoridad por campo:** qué documento manda cuando dos dicen cosas distintas.
5. **Fixtures ficticios** en `fixtures/`: uno por tipo, en texto, y al menos un PDF escaneado o una foto. Nunca datos reales.
6. **La batería por tabla:** una línea por fixture con el tipo, los datos propios esperados y las sugerencias esperadas. Plantilla: `what_each_document_says_of_itself` en `core/reader/rules.rs`.
7. **Las pruebas de ciclo de vida**. Plantilla en `core/reader/tests.rs`:
   - un campo lleno no recibe sugerencia y guardar descarta las que esperaban;
   - lo rechazado no vuelve, ni al volver a subir el mismo archivo;
   - el documento con más autoridad conserva su sugerencia;
   - los datos propios se escriben solos y el reemplazo respeta lo escrito a mano;
   - el archivo en el lugar equivocado no llena nada;
   - quitar el archivo deja lo leído.
8. **La lectura en segundo plano con aviso:** se reusa la del Expediente (clave por documento, avance con su clave, evento al terminar). Nada termina en silencio.
9. **Qué tapa el escáner en su cajón.**
   - El Expediente tapa todo y no rechaza (sus papeles traen por fuerza datos de personas).
   - Personal y Beneficiarios guardan fichas de personas: su Lector es solo de reglas y nada de ellas va a la IA (principio 5).

## 4. Medir antes de cambiar

Un cambio de regla, de motor o de umbral se justifica con una medición, primero en seco:

| Herramienta | Qué mide |
|---|---|
| `cargo test pdf_report -- --ignored --nocapture` | Qué saca Pdfium de cada PDF de una carpeta (letras, palabras partidas, signos raros, páginas como imagen, párrafos, filas de tabla), sin imprimir el texto |
| `cargo test reader_on_a_file -- --ignored --nocapture` | Qué entiende el Lector de un archivo real, sin guardar ni imprimir su texto |
| `cargo test core::reader` | La batería y las pruebas de ciclo de vida |

Las salidas con documentos reales van a un archivo en el equipo, nunca al repositorio.
