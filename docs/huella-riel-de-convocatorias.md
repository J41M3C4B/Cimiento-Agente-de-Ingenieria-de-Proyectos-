# Huella: el riel de convocatorias (2026-10-01 al 02)

Documento breve de lo que se hizo, qué se midió y por qué se abandonó. Reemplaza a las salidas de prueba de esos dos días (borradas). Detalle histórico completo en ADR-013 y ADR-014.

## Qué se hizo
Un **riel**: el código cortaba el PDF en candidatos con id (segmentador), el modelo solo los etiquetaba con un vocabulario cerrado (de 12 a 15 etiquetas) y el código armaba el resultado con contratos por etiqueta, lectores de fechas y montos y la puerta «¿es una convocatoria?». Se midió con 3 convocatorias del Monte de Piedad (2024, 2025, 2026), Fundación Alsea y el Programa para el Bienestar (Edomex), con Flash-Lite (temperatura 0, semilla 7) y Preview.

## Qué se midió (hechos verificados a mano encontrados; Alsea 42, Bienestar 22)
| Arquitectura | Alsea | Bienestar |
|---|---|---|
| Riel original | 25 | 3 |
| Riel sin contratos ni puerta (Flash-Lite) | 33 | 12 |
| Lectura completa estructurada, Preview | 34 | 11 |
| Lectura completa estructurada, Flash-Lite | 8 | 11 (inestable) |
| Riel v2 afinado (**no es resultado ciego**) | 37 | 18 |

Costo y tiempo no decidían: 0.17 a 0.54 MXN y 8 a 55 s por convocatoria en cualquier variante.

## Por qué no funcionó
1. **El código decidía qué era relevante** según cómo estaba escrito el documento (qué es un título, una lista, una tabla, qué páginas «ya son de las reglas», qué palabras indican obligación). Cada convocatoria nueva destapó otra suposición: NMP 2024 dio 0 criterios; Alsea 0 criterios, 0 documentos y 0 fechas; Bienestar quedó en cero por la puerta de clasificación.
2. **Se afinaba contra los mismos documentos.** Los contratos de etiqueta, escritos con las palabras del Monte de Piedad, descartaron el 48 % de las etiquetas del modelo en Alsea. Los resultados «buenos» (37 y 18) eran afinados, no a ciegas.
3. **Compilaba archivos, no significado:** una convocatoria es un paquete de documentos que se remiten entre sí y a veces se contradicen (30 % contra 20 % de contrapartida en Alsea), y el riel leía cada pieza por separado.
4. **No era agnóstico**: lo que no tuviera una señal esperada (viñeta, verbo de obligación, fecha) ni llegaba a ser candidato y se perdía en silencio.

## Qué se aprendió y se conserva
- El código solo debe hacer lo independiente del formato: extraer texto y tablas, verificar que cada cita exista en su página, leer fechas y montos de la cita, medir costo y tiempo.
- Flash-Lite con temperatura 0 y semilla es 100 % reproducible; Preview no tolera temperatura 0 (bucle de razonamiento), tiene 20 llamadas al día y 503 frecuentes.
- La lectura completa pesa lo mismo que el riel en costo (~11 mil tokens por convocatoria de 30 páginas).
- Hace falta un lugar para lo que ningún vocabulario previsto captura (cómo y dónde entregar, «por única ocasión», definiciones de población).
- Medir siempre con hechos verificados a mano (`D:\Agente Proyectos\referencias\alsea.json` y `bienestar.json`) y fijar el criterio de éxito antes de correr.

## Qué sigue
Contrato en lugar de riel: un **schema canónico** (`schemas/canonical_call.schema.json`, ADR-015) que el modelo rellena sección por sección con citas verificadas por código.
