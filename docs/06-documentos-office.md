# 06 · Documentos de Office (Excel y Word)

## Regla clave

**La IA propone, el código escribe.** La IA nunca recibe ni devuelve un archivo. Recibe una lista estructurada y devuelve JSON. El código valida y escribe en una **copia** del archivo original.

## Excel: cuestionarios

### Paso 1 · Lectura y mapeo (código)

Se recorre el libro y se genera una lista de campos:

```json
{
  "sheet": "Datos generales",
  "cell": "C14",
  "question": "Número de beneficiarios directos",
  "field_type": "number",
  "required": true,
  "allowed_values": null,
  "current_value": null
}
```

Heurísticas para encontrar preguntas y celdas de respuesta:

- Celda con texto + celda contigua vacía (a la derecha o abajo) con borde o color de relleno distinto.
- Celdas con validación de datos → `field_type = list` y `allowed_values` desde la validación.
- Celdas con fórmula → `field_type = formula`; **no se escriben nunca**.
- Celdas combinadas → se escribe solo en la celda superior izquierda.
- Hojas ocultas → se leen pero no se muestran salvo que tengan campos requeridos.

El mapeo se guarda en `questionnaire_field`. Si la convocatoria se repite cada año, el mapeo queda en su `call_template` y se reutiliza (costo de IA cero).

Si la heurística duda, la UI muestra la hoja y el usuario marca la celda con un clic. La IA puede ayudar a etiquetar preguntas ambiguas, pero solo como sugerencia.

### Paso 2 · Respuestas

Orden de llenado, de más barato a más caro:

1. **Mapeo directo al perfil** (`profile_mapping`): "Número de beneficiarios" ← suma de `population_group.count`. Sin IA.
2. **Cálculo**: totales, porcentajes, promedios → código.
3. **Respuesta corta conocida**: datos de la institución, fechas, montos del proyecto → código.
4. **Texto abierto** ("Describa el impacto esperado") → IA, con el diagnóstico y la sección del proyecto como contexto. Origen `ai_assumption` hasta confirmar.

### Paso 3 · Validación (código)

- Requeridos vacíos.
- Tipo correcto (número, fecha, valor dentro de la lista permitida).
- Longitud máxima si la hay.
- Totales del cuestionario vs presupuesto del proyecto: deben cuadrar.

### Paso 4 · Escritura

- Se abre una copia del archivo original y se escribe **solo** en las celdas de respuesta.
- Se conservan estilos, celdas combinadas, validaciones y fórmulas.
- Se marca el libro para recalcular fórmulas al abrir.
- Nombre de salida: `{nombre original} - llenado {fecha}.xlsx`.
- Se escanea el resultado antes de entregarlo.

### Límites de la versión 1

| Caso | Comportamiento |
|---|---|
| `.xlsm` (macros) | No se procesa. Mensaje: "Este archivo trae macros y por seguridad no lo modificamos. Te mostramos las respuestas para que las copies." |
| Hoja protegida con contraseña | Igual que arriba |
| `.xls` antiguo | Se pide guardarlo como `.xlsx` (Fase 5.5: conversión automática si la librería lo permite) |
| Imágenes/logos | Deben conservarse; se prueba en Fase 0 |

## Word: la guía del proyecto (ADR-018)

Cimiento **no llena el formato del donante** (ADR-011): ni el Excel ni el Word de la convocatoria. El Word que genera es una **guía con las conclusiones** del proyecto para que la persona la pase a esos formatos.

### Cómo se construye

El .docx se escribe desde bloques (`documents/docx.rs::build`): título, encabezados, párrafos, viñetas, listas de verificación «☐», tablas con fila de encabezado y notas. No hay plantilla binaria: el número de secciones cambia con cada convocatoria, así que la «plantilla propia» vive en el código. Letra Calibri, tamaño carta, estilos mínimos; la única metadata es el título, el autor (la institución) y la fecha.

```
datos confirmados (convocatoria, resumen, secciones, presupuesto, cronograma, perfil)
  → el código arma los bloques (guide_service)
  → escaneo de todo el texto: solo lo que identifica a una persona cuenta; cero hallazgos o no se genera
  → docx::build
  → se guarda una copia nueva en Descargas (nunca sobrescribe)
```

Reglas:

- La IA no recibe ni devuelve archivos: solo redactó textos que la persona confirmó.
- Todas las cifras de la guía salen de código (presupuesto, cronograma) o de lo confirmado.
- Solo se genera con la revisión limpia y en la última etapa; lo que cambie después obliga a volver y regenerar.
- `fill_template` (marcadores `{{…}}` sobre un .docx existente) se conserva para cuando haga falta, pero hoy no se usa en el producto.

### Vista previa

En la app se muestra el contenido por secciones (no un render del .docx). La guía final se abre en Word para revisión.

## Pruebas técnicas obligatorias (Fase 0)

Con archivos de prueba creados por nosotros que imiten formatos reales:

- [ ] Excel con celdas combinadas, validación de lista, fórmulas, logo y colores → leer, escribir 10 celdas, abrir en Excel: formato intacto y fórmulas recalculadas.
- [ ] Word con encabezado, pie, logo, estilos y tabla → reemplazar 5 marcadores y agregar una tabla: formato intacto.
- [ ] Si alguna falla con las librerías de Rust, probar el plan B (Python sidecar) y documentar en ADR-004.
