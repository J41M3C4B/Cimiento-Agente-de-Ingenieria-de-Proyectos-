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

## 3. La capa con IA: llenar con los documentos (cuando el cajón la usa)

Las reglas leen los formatos fijos al subir un archivo y no cuestan nada. Lo que está escrito de mil maneras (la misión, a quién atiende, el personal…) lo lee la capa con IA, solo cuando la persona la pide («Llenar con mis documentos»). Referencia: `core/reader/fill.rs` (C2, ADR-036).

1. **Qué se pide:** solo los campos **vacíos** que la IA puede ver. Lo lleno no (ADR-038) y lo protegido tampoco: solo lo leen las reglas.
2. **Qué se le da:** por campo, los 3 pasajes del cajón que mejor hablan de él. Se buscan por palabras (las del campo en una tabla, como `SEARCH`, más su título del manual) y por significado si hay vectores. Nunca documentos completos.
3. **Una llamada por ventana** con el modelo más barato. Lleva los campos con cada opción y su nombre de la pantalla (`option_label`), y los pasajes numerados con su documento y página puestos por el código.
4. **El código revisa:**
   - el campo pedido;
   - el pasaje mostrado;
   - la cita dentro de ese pasaje (`core::reader::quote::find`, la misma para todo agente que cita; ver abajo);
   - la cifra dentro de la cita (la IA no calcula). «Un», «una» y «primer» solos no son cifra: «voluntarios de una parroquia» no dice 1 (`fill::says_number`);
   - el valor que cabe.

   Lo que pasa es sugerencia con documento y página, bajo la misma política (sección 2). Sin documentos, sin campos vacíos o sin pasajes, no se llama.
5. **Su batería** (como `fixtures/expediente/lector.json` y `reader_live`):
   - por campo, el valor esperado y sus páginas, o «nada» si ningún documento lo dice;
   - primero en seco (la página esperada está entre los pasajes; la verificación descarta lo inventado);
   - después, una corrida real con el modelo más barato: valores, páginas, inventados, llamadas, tokens, costo y tiempo, anotados en el roadmap y en `07-ia-y-costos.md`.
   - Si falla, primero se mejora lo que se le da (palabras, pasajes, oficio, nombres de las opciones); solo después se mide un modelo más fuerte.

## 3b. El Lector contesta preguntas: el agente del cajón como herramienta del Asistente

Referencia: `core/reader/ask.rs` y la herramienta `record_reader` (C3, ADR-036 §6).

1. **El Asistente no lee documentos:** le pregunta al Lector del cajón. Recibe la respuesta ya revisada, nunca pasajes crudos, y solo cita los pasajes que el Lector le entregó. Un solo nivel de delegación: el Lector no pregunta a nadie.
2. **El Lector busca en su cajón** los 4 pasajes que mejor responden, o solo en un documento cuando la persona lo nombró (con sus primeras páginas si ninguna palabra coincide). Hace una llamada con el modelo más barato.
3. **El código revisa** que cada cita esté en un pasaje mostrado y que cada cifra esté en lo citado. Sin cita que pase, la respuesta es «no lo dice».
4. **La IA ve el tipo del documento, nunca el nombre de su archivo** (decisión de la institución, 2026-10-13). El nombre es solo la referencia de la cita en la pantalla. Si la persona nombra un archivo, el código lo reconoce y le dice al agente qué documento es.
5. **El mapa del cajón va en el contexto del Asistente:** qué documentos hay, por su tipo y su estado (con texto, de imagen, sin archivo, falta, vigencia), nunca qué dicen.
6. **Su batería** (como `fixtures/expediente/asistente.json` y `assistant_battery_live`):
   - preguntas desde varias pantallas, con la herramienta esperada, el documento y la página que se deben citar, y al menos una que ningún documento contesta;
   - primero en seco (la página esperada está entre los pasajes);
   - después, rondas reales con el modelo más barato, porque un modelo barato no decide igual cada vez.

**Cómo se revisa una cita** (`core::reader::quote::find`, C4). La usan el Lector de C2 y C3 y el Capturista:
- sin mayúsculas, acentos ni signos;
- se perdona un desliz de copia del modelo: «Reg1menes» por «Regímenes», o letras escritas como HTML (`&eacute;`). Una cita de 12 letras o más puede diferir en una letra de cada 20, a lo más 3; una más corta debe ser exacta;
- **nunca una cifra:** los números de la cita deben ser los mismos que los de las palabras halladas;
- lo que se guarda y se muestra son las palabras del documento (o de la persona), nunca la copia del modelo.

Personal y Beneficiarios no tienen esta capa: nada de sus fichas llega a la IA (principio 5).

## 4. Lo que entrega cada módulo para su cajón

1. **Sus tipos de documento:** código, clase y las frases que los reconocen, sin acentos y en minúsculas.
2. **Sus datos propios por documento:** las columnas de su ficha y una tabla de origen por campo como `core_record_field`, con `origin` y `source_ref`, que se queda aunque se borre el archivo.
3. **Reglas por tipo, en Rust y sin IA.**
   - Cada valor lleva su página y su línea; un dato protegido no guarda la línea.
   - Tolerancia a lo que lee mal una imagen en datos con formato: dígitos confundidos con letras (O→0, I/l→1, S→5, B→8), siglas («1.A.P.»).
4. **Su tabla de autoridad por campo:** qué documento manda cuando dos dicen cosas distintas.
5. **Fixtures ficticios** en `fixtures/`: uno por tipo, en texto, y al menos un PDF escaneado o una foto. Nunca datos reales.
6. **La batería por tabla:** una línea por fixture con el tipo, los datos propios esperados y las sugerencias esperadas. Plantilla: `what_each_document_says_of_itself` en `core/reader/rules.rs`.
6b. **La prueba completa del cajón:** sus archivos como los sube una institución (escaneos, fotos, Word, una hoja girada, una copia desteñida y archivos con fallas), uno por documento, con lo que debe pasar en un `esperado.json` y una prueba en seco que los sube por el mismo camino que la pantalla. Lo que todavía no sale bien va en «conocidos»: se reporta aparte y la prueba pide quitarlo cuando se corrige. Plantilla: `fixtures/expediente/prueba-completa` y `core/records/full_tests.rs`; la guía para la persona, `docs/16-institucion-de-prueba.md`.
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

## 5. Medir antes de cambiar

Un cambio de regla, de motor o de umbral se justifica con una medición, primero en seco:

| Herramienta | Qué mide |
|---|---|
| `cargo test pdf_report -- --ignored --nocapture` | Qué saca Pdfium de cada PDF de una carpeta (letras, palabras partidas, signos raros, páginas como imagen, párrafos, filas de tabla), sin imprimir el texto |
| `cargo test reader_on_a_file -- --ignored --nocapture` | Qué entiende el Lector de un archivo real, sin guardar ni imprimir su texto |
| `cargo test core::reader` | La batería y las pruebas de ciclo de vida, y la capa con IA en seco |
| `cargo test reader_live -- --ignored --nocapture` | La capa con IA con el modelo más barato sobre el expediente ficticio (4 llamadas) |

Las salidas con documentos reales van a un archivo en el equipo, nunca al repositorio.
