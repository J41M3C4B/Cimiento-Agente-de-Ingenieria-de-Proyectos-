# ADR-004 · Office en Rust, Python como plan B

**Estado:** Aceptada (Fase 0). Plan B con Python **no necesario** por ahora; falta confirmación visual en Office real (ver abajo)

## Contexto
Hay que editar Excel y Word **existentes** conservando formato, fórmulas, celdas combinadas, validaciones e imágenes. Las librerías de Rust para edición son menos maduras que las de Python.

## Decisión provisional
- Excel: `calamine` para lectura, `umya-spreadsheet` para edición.
- Word: manipulación directa del ZIP/XML de la plantilla (`zip` + `quick-xml`); `docx-rs` para documentos nuevos.
- Si las pruebas de `06-documentos-office.md` fallan, ese módulo se implementa como sidecar de Tauri en Python (openpyxl / python-docx), empaquetado como ejecutable.

## Resultado de la prueba técnica (Fase 0)

Archivos de prueba ficticios en `fixtures/office/` (se regeneran con `generar.py`). Código en `src-tauri/src/documents/`.

### Excel: `umya-spreadsheet 3.1` (edición) + `calamine 0.36` (lectura)
Se escribieron 10 celdas (texto, números, fecha, lista, celda combinada, texto largo, en 2 hojas) y se comparó el XML interno antes/después.
- **Se conservó:** celdas combinadas, validación de lista, fórmulas, imágenes/logos (2 de 2), hoja oculta, rellenos, formatos numéricos y estilo de las celdas de respuesta.
- **Se perdió:** `fullCalcOnLoad` (la librería lo escribe fijo sin esa marca) y las fórmulas quedan sin valor guardado. **Solución:** parche sobre `xl/workbook.xml` al guardar (`mark_full_calc_on_load`), así Excel recalcula al abrir.
- La librería reescribe las cadenas en `sharedStrings.xml`; es equivalente.
- Las pruebas son `documents::xlsx::tests`.

### Word: ZIP + XML directo (`zip` + `regex`), sin `docx-rs`
- Reemplazo de marcadores `{{x}}` en cuerpo, encabezado y pie, **incluido un marcador partido en dos runs** (se conserva el formato del run donde empieza). Párrafos múltiples se clonan. Tabla nueva insertada en lugar de `{{table:x}}`. Autor del documento reemplazado.
- Se conservaron todas las partes del paquete (estilos, encabezado con logo, pie, tabla existente, viñetas).
- Limitación conocida: la tabla nueva usa el estilo `TableGrid`; si la plantilla del donante no lo define, Word usa el predeterminado. Pendiente (Fase 4): copiar el estilo de la tabla existente en la plantilla.
- Las pruebas son `documents::docx::tests`.

### Pendiente de confirmar a mano
Esta máquina no tiene Excel, Word ni LibreOffice, así que la verificación fue estructural (XML), no visual. Abrir en Office real `src-tauri/target/office-out/cuestionario-prueba-llenado.xlsx` y `plantilla-prueba-llenada.docx` (se regeneran con `cargo test`) y confirmar que abren sin aviso de reparación y con el formato intacto. Si fallan, activar el plan B (sidecar Python) solo para el módulo afectado.

### PDF: `pdf-extract 0.12` (solo texto, sin OCR)
- PDF con tabla: el texto sale completo; las filas de la tabla salen como una línea (`Obra y equipamiento $300,000 12`), suficiente para fragmentar y buscar, no para reconstruir columnas.
- PDF escaneado (solo imagen): se detecta por páginas con menos de 20 caracteres visibles y se reporta (`PdfText::is_fully_scanned`, `pages_without_text`) para avisar al usuario. No se intenta OCR en v1.
- Archivo dañado: el extractor puede entrar en pánico; se captura y se convierte en error.
- Las pruebas son `documents::pdf::tests`. No se probó `pdfium-render`; no hizo falta.
