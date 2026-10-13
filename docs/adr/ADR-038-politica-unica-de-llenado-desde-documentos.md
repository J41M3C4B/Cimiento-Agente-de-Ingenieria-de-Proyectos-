---
adr: 38
estado: aceptada
fecha: 2026-10-13
tema: [documentos, ia, nucleo]
reemplaza_en_parte: ["[[ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento]]", "[[ADR-036-cajones-de-documentos-y-un-agente-por-cajon]]"]
tags: [adr]
---
# ADR-038 · Una sola política para llenar desde documentos, y la lectura de archivos sin huecos

**Estado:** aceptada (2026-10-13, decisiones de la institución). Aplicada en el Expediente; es la metodología de los Lectores de todos los módulos ([metodología](../agents/metodologia-lectores.md)).

## Contexto

Llevábamos varias iteraciones atacando síntomas de la lectura de archivos (el OCR, el rechazo como «lista de personas», el motor de PDF). Las causas de raíz seguían ahí:

1. **El Lector nunca llenaba la ventana del propio documento.** Solo proponía datos para las cuatro ventanas de «Mi institución». Un poder trae su vigencia y su número de escritura, y la ventana del poder quedaba vacía mientras se sugerían datos en otras ventanas.
2. **No había una política de llenado; había excepciones sueltas:**
   - se sugería sobre campos ya llenos;
   - lo rechazado volvía;
   - al guardar un campo, sus sugerencias seguían ahí;
   - si dos documentos proponían el mismo campo, ganaba el último subido;
   - un archivo puesto en el lugar equivocado se leía con las reglas equivocadas.
3. **La lectura tenía huecos:**
   - las tablas seguían con otro motor;
   - una página con encabezado impreso y cuerpo escaneado no se leía;
   - las hojas giradas salían como basura;
   - una CURP leída con «O» por «0» escapaba al escáner;
   - un acta larga congelaba el equipo y obligaba a esperar con la ventana abierta.

## Decisión

### 1. Datos propios solos; datos de «Mi institución», como sugerencia

- **Los datos del propio documento se escriben solos al subirlo** si el campo está vacío: fecha de emisión, vigencia y folio o número. Llevan su origen y su página.
  - El origen por campo vive en `core_record_field` (migración 0033): `document`, `computed` o `user`.
  - Al reemplazar el archivo se leen del nuevo, **nunca sobre lo que escribió la persona**.
  - Quitar el archivo deja lo leído.
- **Los datos de las ventanas de «Mi institución» siguen siendo sugerencia con «Usar».**

### 2. Un campo lleno no recibe nada

Ni sugerencia ni aviso.

- La regla vive en un solo punto, `proposals::propose`, por donde pasan todas las sugerencias:
  - **campo lleno:** se rechaza;
  - **rechazado antes:** se rechaza, por la huella SHA-256 del valor, que sobrevive al valor olvidado y al archivo borrado;
  - **autoridad por campo:** una sugerencia pendiente de un documento con más autoridad no la reemplaza una de menos (`reader::authority`; ejemplo: el nombre legal, la constancia del SAT antes que el acta).
- Al guardar, las sugerencias de documentos para los campos que quedaron llenos se descartan solas.
- Lo que propone la IA no sigue estas reglas: la persona le pidió el cambio en el chat.

### 3. Un archivo en el lugar equivocado no llena nada

Si no menciona su propio tipo y una regla reconoce otro, se guarda y se avisa: «Este archivo parece «Constancia de situación fiscal», no este documento».

### 4. Reglas de los datos propios por tipo

| Tipo | De dónde sale cada dato propio |
|---|---|
| Escrituras (acta, reformas, poder) | La fecha en palabras de sus primeras páginas y el número de escritura, también en palabras («cuarenta y dos mil ciento diez» → 42110) |
| Oficio de donataria | Su fecha y su número de oficio |
| Opinión de cumplimiento (32-D) | Su fecha de emisión; vigencia de 30 días naturales, calculada (origen `computed`), salvo que diga otra |
| El resto | La fecha después de «fecha de emisión, expedición, presentación, inscripción o registro», «vigente hasta…», el folio y la clave CLUNI |

Lo que lee mal una imagen se corrige por regla: el RFC con letras en lugar de dígitos y «1.A.P.».

Batería de 13 documentos ficticios.

### 5. Lectura de archivos sin huecos

- **Pdfium lee también las tablas** (ADR-037, bloque 3); salen `pdf-extract` y `lopdf`.
- **Página mixta:** imágenes en la mitad o más y menos de 200 letras → se lee como imagen.
- **OCR:**
  - Tesseract con prioridad baja;
  - una hoja girada se endereza con el modelo de orientación (OSD) y se lee otra vez;
  - una página casi sin tinta, medida contra su propio papel, no se relee;
  - en texto leído de imagen, el escáner tapa una CURP o un RFC con letras por dígitos si, corregidos, siguen siéndolo.
- **El Expediente lee en segundo plano:**
  - una lectura por documento a la vez;
  - la ventana se puede cerrar;
  - el evento `record-read` dice cómo terminó, también si hubo error;
  - la fila dice «Leyendo: 3 de 12 páginas»;
  - si el programa se cierra, la lectura no se reanuda.
- Un archivo con contraseña lo dice (ADR-037, bloque 2).
- **Las imágenes dentro de un Word o un Excel** (una hoja escaneada pegada) se leen con el mismo OCR, como páginas al final del texto; una imagen chica (un logo, menos de 600 píxeles por lado) no.

## Consecuencias

- **Para los módulos:** la [metodología de los Lectores](../agents/metodologia-lectores.md) dice qué entrega cada uno: tipos, datos propios con origen por campo, reglas, autoridad, fixtures, batería y pruebas de ciclo de vida.
- **Reemplaza en parte:**
  - al ADR-034 (IA6): el Lector ya no solo sugiere; los datos propios se escriben solos;
  - al ADR-036 §4: la política del Lector del Expediente.
- **Límites conocidos:**
  - una lectura en segundo plano no sobrevive a cerrar el programa;
  - la letra a mano no se lee;
  - el selector de archivos nativo no se pudo manejar en la prueba del WebView real: el flujo de la ventana se probó con Vitest, y el comando, el evento y el aviso, en la app real.
