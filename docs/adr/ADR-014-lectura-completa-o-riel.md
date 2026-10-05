# ADR-014 — Decisión estratégica: riel de convocatorias o lectura completa estructurada

**Estado:** REEMPLAZADA por ADR-015 (schema canónico, 2026-10-02). El riel v2 se abandona: lo medido con él es resultado afinado sobre los mismos documentos, no resultado ciego. Resumen de lo hecho y por qué no funcionó: `docs/huella-riel-de-convocatorias.md`.

## Contexto
Al cerrar el riel (ADR-013) se vio que: (1) **no ahorra dinero**: una lectura completa de una convocatoria de 30 páginas son unos 11 mil tokens (≈ $0.24 MXN con Flash-Lite, ≈ $0.32 con Preview) y el riel con Flash-Lite cuesta $0.17 a $0.24; lo que compró fue reproducibilidad y verificación; (2) pierde entre el 25 y el 40 % de las conclusiones de una lectura completa (ADR-013); (3) todo lo medido es de un solo convocante (Monte de Piedad), con reglas y segmentador afinados leyendo esos PDFs: la primera vez que el extractor vio NMP 2024 dio 0 criterios y 0 documentos, y cada convocatoria nueva pidió arreglos. No se sabe qué hace con otra institución, y mantener una regla por formato es el costo real.

## Lo que se conserva pase lo que pase
Limpieza de texto, lectores de fechas y montos, verificación contra el texto, contratos de etiqueta, temperatura 0 solo en los modelos medidos (Flash-Lite: 100 % consigo mismo), y el banco de medición (`call_review_agreement_live`, `call_review_consistency_live`, `call_full_reading_vs_rail_live`).

## Opciones
1. **Riel** (ADR-013): el código corta y encuentra, el modelo etiqueta; reglas como fuente principal.
2. **Lectura completa libre**: un modelo lee todo y escribe conclusiones. Hereda el problema que motivó el riel: con el prompt abierto de la primera revisión, tres modelos dieron tres perfiles distintos del mismo PDF.
3. **Lectura completa estructurada** (propuesta a medir): una sola llamada lee el documento entero y llena una ficha de campos fijos (partes, fechas por etapa, topes, quién puede participar, documentos, evaluación, restricciones, población por tipo de institución, requisitos del proyecto, condiciones, dudas); cada dato lleva su cita literal y su página y el código la verifica; las reglas pasan de fuente principal a validador (un valor que encuentran y el modelo no se lista como desacuerdo); lo que solo sirve para el formato del Monte de Piedad se queda como verificación opcional o se retira.

## Sobre los embeddings
No repiten el trabajo de la primera lectura (esa es proactiva: lo que la IA debe saber para la primera pregunta; la búsqueda es reactiva), pero para **una** convocatoria de ~11 mil tokens lo más simple es meter el documento completo en el contexto de cada turno, con caché del prompt (la parte repetida cuesta una décima parte): unos pocos pesos por diagnóstico. Los embeddings se justifican con muchos documentos grandes (guías, anexos, historial de la institución) que no caben en el contexto, no para una sola convocatoria. Se aplazan hasta medir tamaños y costos reales.

## El duelo que decide
Tres arquitecturas × dos modelos (Flash-Lite con temperatura 0, Preview). Por cada una: costo, tiempo, reproducibilidad en tres corridas, cobertura contra la referencia (las 100 conclusiones de las lecturas completas y los 36 hechos de NMP 2026) y exactitud de los datos duros (fechas, topes, figuras legales, documentos).

**Condición:** mientras todo sea del Monte de Piedad no mide viabilidad general. Hacen falta 2 o 3 convocatorias de otras instituciones, en PDF con texto y de formatos muy distintos (fundación empresarial, convocatoria gubernamental, cooperación internacional), usadas solo para medir, sin afinar contra ellas.

## Lo que mostraron los documentos de otros convocantes (primer vistazo, sin gastar)
Se consiguieron dos convocantes distintos del Monte de Piedad: **Fundación Alsea** (fundación corporativa: PDF de 13 páginas con la convocatoria 2027 más un formulario en Word) y el **Programa para el Bienestar de las IAP del Estado de México** (programa de gobierno: cartel de una página, Reglas de Operación de la Gaceta del Gobierno de 11 páginas, carta compromiso en Word y formato único en Excel). La convocatoria de la JAP de la Ciudad de México exige iniciar sesión y solo la directora puede abrirla, así que queda fuera.

El extractor de reglas, sin tocarlo, sobre los dos:
- **Bienestar, Reglas de Operación:** clasificó el documento como «aviso de gobierno que no es una convocatoria de donativos» y no sacó ningún requisito; con esa clasificación el riel no llama al modelo.
- **Bienestar, cartel:** tipo convocatoria, pero 1 criterio, 0 documentos y 2 fechas; los criterios de priorización (a–e) y los requisitos quedaron en una maqueta de dos columnas que las reglas no leen.
- **Alsea:** tipo «desconocido», **0 criterios, 0 documentos, 0 fechas** y un solo límite (12 meses).

Hallazgos que no estaban en el diseño:
1. **Una convocatoria es un paquete de documentos que se remiten entre sí**, no un PDF: el cartel de Bienestar dice «los requisitos del numeral 8.1 de las Reglas de Operación» y los formatos están en un Excel y un Word. Leer cada archivo por separado pierde esas remisiones.
2. **Los documentos de un mismo paquete pueden contradecirse:** el PDF de Alsea 2027 pide una contrapartida mínima del 30 % y el formulario en Word (lineamientos de «2026», con fecha límite del 30 de septiembre de 2026) dice 20 %.
3. **La clasificación previa («¿es una convocatoria?») es una puerta que falla con convocantes nuevos** y deja todo en cero sin avisar. Una lectura completa no necesita esa puerta.

## El duelo (`call_duel_live`)
Mide, sobre un paquete de documentos, las dos arquitecturas (el riel tal como corre en el producto y la lectura completa estructurada, `call_sheet`) con dos modelos (Flash-Lite con temperatura 0 y Preview). Se puntúa contra una referencia de hechos verificados a mano (`referencias/alsea.json`, 42 hechos; `referencias/bienestar.json`, 22 hechos; fuera del repositorio porque vienen de documentos reales). Un hecho cuenta como encontrado si todos sus patrones aparecen en lo que la arquitectura produjo. Reporta costo y tiempo reales, y la lista de hechos que faltan. Se usa solo para medir: no se afina nada contra estos documentos.

## Resultado del primer duelo (hechos verificados encontrados; Alsea 42, Bienestar 22)
| Arquitectura | Alsea | Bienestar | Costo MXN | Tiempo |
|---|---|---|---|---|
| Riel (como corre hoy), Flash-Lite o Preview | 25 y 25 | 3 y 3 | 0.18–0.24 | 7 s / 55 s |
| Lectura completa estructurada, Preview | **34** | **11** | 0.34–0.41 | 18–19 s |
| Lectura completa estructurada, Flash-Lite | 8 (15 con el PDF y el Word juntos) | 11 | 0.17–0.20 | 6–10 s |

- El riel sale mal parado con convocantes nuevos, pero **parte del techo era sobreajuste propio**: los contratos de etiqueta (escritos con las palabras del Monte de Piedad) descartaron 50 de las 104 etiquetas del modelo en Alsea (48 %): criterios de evaluación numerados, lista de prioridades y requisitos del proyecto en viñetas sin verbo de obligación pasaron a «no relevante»; y la puerta «¿es una convocatoria?» dejó el riel en 3/22 en Bienestar sin llamar al modelo. La comparación no fue justa con el concepto del riel.
- El riel dio 25 con los dos modelos en Alsea: **la cobertura la fija el código y no depende del modelo** (su virtud real), a un techo bajo por el vocabulario y los contratos.
- La lectura completa con Preview es la mejor y más rápida que el riel con Preview, pero **enumera menos al final de las listas y depende de un modelo preview** (20 llamadas al día, 503 frecuentes). Con Flash-Lite es **inestable** (8/42 con 14 elementos en Alsea, 11/22 en Bienestar): cuánto enumera varía con el día, el mismo problema que motivó el riel.
- **Capacidad que solo tiene la lectura completa:** con el PDF y el Word de Alsea juntos, Flash-Lite señaló la contradicción de la contrapartida («las páginas 4 y 5 indican 30 % mientras que la página 14 indica 20 %») y leyó bien los topes. El riel lee cada documento por separado y no la puede dar.
- **Faltan hechos que ninguna de las dos arquitecturas captura:** cómo y dónde entregar (físico y USB, lugar, horario), «por única ocasión», «no se reciben solicitudes incompletas». El vocabulario del riel y las secciones de la ficha no tienen dónde ponerlos; también faltan en el riel el calendario por etapas, las causas de descalificación y los rubros no financiables.

Hipótesis a medir antes de decidir: **enumeración por código + las secciones de la ficha como vocabulario + sin contratos por palabras ni puerta de clasificación**. Para medir el concepto del riel sin lo hecho a la medida de un convocante hay dos interruptores solo para pruebas en vivo (`CIMIENTO_RAIL_CONTRACTS=off`, `CIMIENTO_RAIL_GATE=off`). Si el riel libre sube con Flash-Lite a ~30/42 en Alsea y ~10/22 en Bienestar, el concepto es sano y se rehace con el vocabulario de la ficha; si se queda cerca de 25 y 3, la base pasa a ser la lectura completa con Preview y respaldo.

## Segundo duelo: el riel sin lo hecho a la medida del Monte de Piedad
Con los contratos de etiqueta y la puerta de clasificación apagados (solo en la prueba en vivo), Flash-Lite:

| | Alsea (42) | Bienestar (22) | Costo MXN | Tiempo |
|---|---|---|---|---|
| Riel como estaba | 25 | 3 | 0.18–0.24 | 7–55 s |
| **Riel libre, Flash-Lite** | **33** | **12** | 0.18 / 0.21 | 8 s / 54 s |
| Lectura completa estructurada, Preview | 34 | 11 | 0.34 / 0.41 | 19 s / 18 s |
| Lectura completa estructurada, Flash-Lite | 8 | 11 | 0.17–0.20 | 6–10 s |

El riel liberado con el modelo barato igualó a la lectura completa con el modelo fuerte por la mitad del costo: pasó de 25 a 33 y de 3 a 12 solo quitando lo hecho a la medida de un convocante. Lo que seguía faltando eran huecos de vocabulario: en Alsea, el calendario por etapas, los rubros no financiables y un criterio de evaluación; en Bienestar, cómo y dónde entregar (lugar, USB, solicitudes incompletas, formatos), «por única ocasión», el objetivo y algunas obligaciones.

**Decisión: riel v2** (prompt `call_review.v4`): (1) se quitan los contratos por palabras y la puerta de «¿es una convocatoria?» (código archivado en `docs/codigo-archivado/call_review.v3-con-contratos.rs.txt`); (2) el vocabulario pasa de 11 a 15 etiquetas con las secciones que otro convocante necesita: `population` (a quién atiende y a quién no), `funded` (qué se puede financiar), `calendar` (etapas con sus fechas) y `submission` (cómo, dónde, cuándo y en qué formato se entrega); `exclusion` pasa a cubrir también los gastos no financiables y las causas de descalificación, y va solo al perfil (ya no se agrega como criterio de elegibilidad); (3) el prompt dice que un elemento de una lista bajo un título de criterios de evaluación o priorización es de esa clase aunque no repita la palabra.

Lo que se conserva del resto de la comparación: la lectura completa estructurada (`call_sheet`) queda como lector opcional para lo que el riel no puede dar (contradicciones entre documentos del mismo paquete: con el PDF y el Word de Alsea juntos detectó 30 % contra 20 %), sin conectar a nada. Los embeddings siguen aplazados.

Limitación nueva anotada: el segmentador solo emite frases con alguna señal (viñeta, obligación, exclusión, prioridad, participación, tope, documento, fecha, cifra o título de requisitos). Una convocatoria escrita sin ninguna de esas palabras no genera candidatos.

Alsea y Bienestar ya no son documentos reservados: se miraron sus fallos para decidir. Para medir la v2 sin sesgo hacen falta otras convocatorias de otras instituciones.

## Tercer duelo: riel v2 con Flash-Lite (vocabulario de 15 etiquetas, sin contratos ni puerta)
| | Hechos encontrados | Costo MXN | Tiempo |
|---|---|---|---|
| Alsea (42) | 37 | 0.18 | 8 s |
| Bienestar (22) | 18 | 0.21 | 9 s |
| Alsea + Word (42) | 37 | 0.25 | 10 s |

Los 5 hechos que seguían faltando en Alsea (teoría de cambio y los 4 rubros no financiables) no eran errores del modelo. Se revisó sin modelo, solo con el segmentador:
1. **Los elementos de una lista numerada cuya primera línea no termina en punto se tomaban por títulos de sección y se tiraban.** En la p. 7 de Alsea desaparecían 6 de los 8 criterios de evaluación (sobrevivían el 5 y el 6, de una línea con punto).
2. **Una regla escrita para el Monte de Piedad** («las viñetas de una página con "Rubros Financiables" ya las leen las reglas») **descartaba toda la lista de lo que se financia y lo que no** y nunca llegaba al modelo.
Ambos se corrigieron y se verificaron sin modelo (294 tests; cobertura del segmentador sin cambios: 27/27 en 2026, 19/19 en 2024, 1/1 en Alsea). No se ha medido con el modelo.

## Autocrítica (Jaime, 2026-10-02): «el riel no está funcionando, solo ajustamos parámetros para que cuadre con lo que tenemos; no es lo suficientemente inteligente para cubrir lo que sea»
Es correcta, y los dos defectos recién encontrados son la prueba: cada documento nuevo destapa otra suposición del código sobre cómo se escribe una convocatoria (cómo es un título, qué es una lista, qué es una tabla, qué páginas «ya son de las reglas»). Qué es relevante lo decide el código en demasiados sitios, y eso no converge para «lo que sea».

Qué es evidencia a ciegas y qué no:
- **Medido sin afinar contra esos documentos:** lectura completa estructurada con Preview 34/42 (Alsea) y 11/22 (Bienestar); riel original 25/42 y 3/22; riel sin contratos ni puerta 33/42 y 12/22 (solo se quitó lo hecho para el Monte de Piedad, aunque ya se habían visto los fallos).
- **Afinado después de ver los fallos de esos documentos (no es resultado ciego):** riel v2, 37/42 y 18/22.
- La lectura completa con Flash-Lite fue inestable (8/42 en Alsea, 11/22 en Bienestar) y con Preview depende de un modelo preview (20 llamadas al día, 503 frecuentes).

## Qué se conserva con cualquier arquitectura
Extraer el texto de PDF y Word; verificar cada cita contra su página; leer fechas y montos de la cita; temperatura 0 solo en Flash-Lite (100 % reproducible); el banco de medición (`call_duel_live`, referencias en `D:\Agente Proyectoseferencias`, `call_review_agreement_live`, `call_review_consistency_live`, `call_full_reading_vs_rail_live`); el vocabulario de 15 secciones; la detección de contradicciones entre documentos de la lectura completa.

## Para decidir mañana (sin más ejecuciones hasta decidir)
Principio propuesto: el código solo hace lo que no depende de cómo esté escrita una convocatoria (extraer texto, verificar citas, leer fechas y montos de las citas, controlar costo y tiempo, medir); **qué es relevante lo decide el modelo**, no una regla.
1. Congelar la arquitectura **antes** de medir con documentos nuevos. Conseguir 2 o 3 convocatorias de otras instituciones (internacional, banco, otra de gobierno), sin afinar nada contra ellas.
2. Dos candidatos sin suposiciones de formato:
   - **Riel mínimo:** el segmentador corta solo por líneas y párrafos, sin filtrar por palabras ni detectar títulos, listas ni tablas; el modelo etiqueta todo con las 15 etiquetas (cuesta más salida pero el documento completo son 4 a 14 mil tokens).
   - **Lectura completa por secciones:** varias pasadas enfocadas sobre el mismo texto («enumera TODOS los requisitos», «TODAS las fechas», etc.), cada una con citas verificadas, para atacar que el modelo se detiene antes al enumerar. Son varias llamadas del mismo tipo, no código por formato.
3. Medirlos a ciegas contra la misma referencia con Flash-Lite y Preview y quedarse con el que cubra más con menos suposiciones. Los embeddings siguen aplazados.
