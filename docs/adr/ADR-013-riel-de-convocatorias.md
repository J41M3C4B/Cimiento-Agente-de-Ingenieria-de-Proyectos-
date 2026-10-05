# ADR-013 — Riel de convocatorias: el código encuentra, el modelo solo etiqueta

**Estado:** REEMPLAZADA por ADR-015 (2026-10-02). Se conserva como historia; ver `docs/huella-riel-de-convocatorias.md`. El código del riel se archivó el 2026-10-03 en `docs/codigo-archivado/riel-*.rs.txt` y sus prompts en `docs/prompts-archivados/`.

## Nombre
**Riel de convocatorias** (módulo `call_rail`). Es el paso de IA de la lectura de una convocatoria. No es:
- **Ingesta en silencio** (ADR-011): todo el flujo al subir el archivo (reglas + modelo + resumen).
- **Extractor**: las reglas deterministas de `call_extract` (sin IA).
- **Revisión v2**: el prompt abierto anterior (`call_review.v2`), que el riel ya reemplazó.

Partes del riel: **candidatos** (segmentador), **etiquetado** (el modelo) y **campos fijos** (preguntas que siempre se responden).

## Problema
Con el prompt v2 («compara y completa») cada modelo decide qué buscar, cuántos elementos listar, con qué palabras escribirlos y cómo redactar `applies_to`. En NMP 2024 dieron perfiles distintos 3.6-flash, Flash-Lite y 3-flash-preview (criterios añadidos 0 vs 2, 1 vs 4 paráfrasis). Una convocatoria no debe dar un resultado con un modelo y otro con otro.

## Decisión
1. El código segmenta las páginas escogidas en **candidatos con id** (oraciones y viñetas con señal de regla; fechas y cifras con su contexto).
2. El modelo **etiqueta cada candidato** con un vocabulario cerrado (criterio, exclusión, prioridad, documento, evaluación, objetivo, convocante, categoría, no_aplica) y una aplicabilidad cerrada (todas, organizaciones_nuevas, por_categoría, por_etapa, otra + nota corta).
3. **El texto lo pone el código**: el modelo devuelve ids y etiquetas, nunca texto. No hay paráfrasis ni citas inventadas.
4. **Todo candidato tiene respuesta**; lo que falte se repregunta una vez y, si no, queda «sin etiquetar» para una persona.
5. **Campos fijos** que siempre se responden, con «no aparece en el texto» como salida válida: fecha de cierre, vencimiento, quiénes pueden participar, duración máxima, tope administrativo, monto, documentos.
6. Definiciones y ejemplos fijos en el prompt, tomados de las convocatorias de desarrollo (NMP 2026, 2025 y JAP), no de 2024.
7. Texto libre del modelo solo en `doubts`, marcado como opinión del modelo.

## Cómo se mide
Prueba de acuerdo entre modelos: los mismos candidatos a Flash-Lite, 3-flash-preview y 2.5-flash sobre NMP 2024, 2025 y 2026; se reporta el porcentaje de candidatos con la misma etiqueta y cuáles difieren. Antes, en seco con un modelo simulado que contesta distinto.

## Consecuencias
- Cambiar de modelo o de proveedor cambia la precisión de las etiquetas, no la estructura ni las palabras.
- Salida corta: cabe en un modelo ligero y baja el razonamiento.
- Deja de depender de verificar citas; esa verificación queda solo para `doubts`.

## Avance: el segmentador de candidatos (`documents/call_rail.rs`)
Sin IA y sin gasto. Garantías que comprueban los tests: cada candidato es un trozo literal de la página limpia, las mismas páginas dan los mismos candidatos con los mismos ids, y nada de lo que encontraron las reglas queda fuera.

Cada candidato trae: `id` (`p8-3`), página, `section` (el título vigente), `lead` (la línea que abrió su lista: «Para las organizaciones que nunca han recibido donativo…:», que dice a quién aplica el ítem), el texto literal y las señales por las que es candidato (`bullet`, `obligation`, `exclusion`, `priority`, `participation`, `limit`, `document`, `date`, `figure`, `table`, `rule_section`).

Medido sobre los PDFs reales (`cargo test survey_candidates -- --ignored --nocapture` con `CIMIENTO_PDF_DIR`):

| Convocatoria | Candidatos | Tamaño | Lo que las reglas encontraron y está entre ellos |
|---|---|---|---|
| NMP 2026 | 166 | ≈ 4,970 tokens | 27/27 |
| NMP 2025 | 132 | ≈ 5,300 tokens | 19/20 |
| NMP 2024 | 97 | ≈ 3,630 tokens | 17/17 |
| JAP (aviso, no convocatoria) | 27 | ≈ 1,420 tokens | 13/13 |

Lo que la medición descubrió y se corrigió:
- En 2025 los criterios no llevan viñeta y están bajo un título de tres líneas en mayúsculas; «Ser una organización privada sin fines de lucro…» no tiene ninguna palabra de obligación. Ahora un título sobre requisitos o criterios convierte en candidatos los párrafos que tiene debajo (señal `rule_section`, vigente en su página y la siguiente) y los títulos que se parten en varias líneas son uno solo.
- Las líneas en mayúsculas con fecha («19 MAYO-6 JUNIO») se tiraban como títulos.
- Los calendarios en tabla salen del PDF como líneas sueltas (las etapas y luego sus fechas, en el mismo orden): una fecha sola no dice de qué etapa es. Una racha de líneas cortas con varias fechas se emite como un solo candidato `table`, incluido el esquema numerado de 2026 («1. Postulación / ABRIL / 2. Registro… / A PARTIR DE JULIO»).

Falta de 80: un criterio de 2025 p. 15 que la propia regla extrae mal (pega el título «Etapas por financiar de los Proyectos» con el párrafo que sigue); el segmentador lo parte bien en oraciones.

## Avance: el etiquetado con una sola llamada (`documents/call_review.rs`, prompt `call_review.v3`)
Decisión tomada: **una sola llamada** con todos los candidatos (≈ 3,600 a 5,300 tokens en las convocatorias medidas), y repreguntar solo lo que el modelo se salte. Partir en tandas por tema queda como plan B si la prueba de acuerdo muestra que un modelo barato se pierde con listas largas.

Qué manda el código y qué decide el modelo:
- **El código** corta, numera, escoge qué candidatos se envían (los renglones de las listas de conceptos y las tablas de indicadores son de las reglas y no se mandan; si el documento excede 30,000 caracteres se descartan primero los de señal más débil), arma el texto de cada elemento y decide dónde va.
- **El modelo** devuelve ids y palabras de listas cerradas: 12 etiquetas (`criterion`, `exclusion`, `priority`, `evaluation`, `document`, `objective`, `funder`, `category`, `date`, `limit`, `procedure`, `not_relevant`), 5 valores de a quién aplica (`all`, `new_organizations`, `per_category`, `per_stage`, `other`) y 9 preguntas fijas (`application_window`, `registration`, `info_session`, `disbursement`, `max_duration_months`, `max_admin_percent`, `min_operating_years`, `max_amount`, `min_amount`) con `found` + ids o `not_in_text`. El esquema JSON solo admite esos valores y un test mantiene en paso el prompt, el esquema y las constantes.
- **Preguntas fijas:** el modelo señala los candidatos y el código lee el valor con los mismos lectores de fechas y límites del extractor. Si el modelo dice que algo no aparece y las reglas lo encontraron, las reglas se quedan y el desacuerdo se lista. Si señala un candidato donde no hay valor, queda «no se pudo leer», nunca un valor inventado.
- **Lo que agrega el modelo** es supuesto de la IA, no bloquea y dice a quién aplica con frases fijas («organizaciones nuevas», «según la categoría»…).
- **Quitar:** el modelo ya no borra nada. Una fecha suelta que las reglas leyeron dentro de una línea que el modelo llama documento («DOF publicado en 18 enero 2024») se quita con su razón; un criterio de las reglas que el modelo llama `procedure` o `not_relevant` queda en la lista de «por revisar» de una persona.
- **Falta de respuestas:** un id sin etiqueta se repregunta una vez, solo ese; si sigue sin etiqueta queda en `unlabeled` para una persona. Un id inventado o una etiqueta fuera de la lista se rechazan y se listan. La primera etiqueta válida de un id gana y el orden en que el modelo las enumere no cambia el resultado.

Descubrimiento colateral: el extractor no leía «a las 11:00 hrs» (24 horas) ni una fecha con hora y sin año («el 10 abril a las 11:00 hrs», la sesión de NMP 2024). Ya las lee; NMP 2026 sigue en 36 de 36.

Los prompts v1 y v2 se archivaron en `docs/prompts-archivados/`.

## Primera corrida real (NMP 2024, `gemini-3-flash-preview`)
Una llamada, 22 s, $0.3439 MXN (7,284 tokens de entrada, 4,983 de salida), 97 de 97 fragmentos etiquetados, 0 respuestas rechazadas, sin repregunta. Funcionó en lo estructural y mostró tres defectos de diseño nuestro, ya corregidos:

1. **El calendario como un solo bloque.** Las seis etapas salían en un solo candidato; el modelo lo señaló para cuatro preguntas fijas y el código aplicó el tipo de cada pregunta a todas las fechas del bloque, que se pisaron entre sí (8 desacuerdos falsos y dos «dudas» de contradicción inexistentes). Ahora un calendario de etapas numeradas es un candidato por etapa («Etapa 5: Postulación del proyecto Mayo 16 al 23»), y una pregunta fija solo cambia el tipo de una fecha cuando el candidato señalado tiene exactamente una; con varias, deja el tipo de las reglas y lo lista.
2. **`criterion` demasiado amplio.** Pasó de 5 a 16 porque el modelo incluyó requisitos del proyecto («deberán incluir perspectiva de derechos humanos») y condiciones al recibir el donativo («no podrán transferir los recursos a cuentas de inversión»). Vocabulario ahora de 14 etiquetas: `criterion` es solo de la organización; `project_requirement` es lo que el proyecto debe incluir; `funding_condition` es lo que debe o no hacer si resulta financiada. Las dos nuevas van al perfil (`project_requirements`, `funding_conditions`), no a la elegibilidad.
3. **`priority` y `category` estirados** a descripciones. El prompt ahora dice qué no son, con ejemplos de la propia corrida.

Observaciones que quedan abiertas: el modelo metió una *sesión de capacitación* (23 mayo 11:00) en `info_session` (la definición ya excluye otras sesiones); el texto de dos columnas deja guiones de corte de línea («de- sarrollo»), que ensucian la cita literal; el candidato de convocante es un párrafo largo de presentación del Monte de Piedad.

## Segunda corrida real y ajustes
Con las tres correcciones anteriores (NMP 2024, `gemini-3-flash-preview`, 22 s): los desacuerdos de fechas pasaron de 8 a 1, `criterion` quedó en 6 (los 5 de la p. 8 más una nota), `application_window` apunta a la Etapa 5 (16–23 mayo), `info_session` ya no mezcla la sesión de capacitación y el modelo señaló una contradicción real del PDF (la p. 24 dice que Evaluación es 3–20 de mayo y la p. 25 que es 1–22 de abril). Quedaban tres cosas, ya atendidas:

- **`disbursement` no se pudo leer** («Etapa 6: … entrega de donativos 15 Julio»): el extractor no lee un día y un mes sueltos (en una página entera sería demasiado laxo). Ahora hay una lectura laxa que solo se usa en un fragmento que el modelo ya señaló como fecha para una pregunta fija.
- **Contrato por etiqueta.** `priority` y `evaluation` solo valen si el propio fragmento habla de prioridad o evaluación; si no, el código lo deja como no relevante y lo lista. El modelo etiquetó como prioridad «se ha visto que los programas con tres estrategias son más efectivos» incluso con el contraejemplo escrito en el prompt: una regla que debe cumplirse va en el código, no en la redacción.
- **`funding_condition` recogió listas de gastos financiables.** El prompt ahora dice que eso (y «los apoyos dependen de los recursos disponibles») es `not_relevant`.

## La prueba de acuerdo entre modelos
`cargo test call_review_agreement_live -- --ignored --nocapture` con `CIMIENTO_CALL_PDF` y, opcional, `CIMIENTO_AGREE_MODELS` (por defecto `gemini-3-flash-preview,gemini-3.5-flash-lite`). Etiqueta la misma convocatoria con cada modelo, fijando ese modelo (sin respaldo a otro), y compara etiqueta por etiqueta: porcentaje de candidatos con la misma etiqueta, misma aplicabilidad, preguntas fijas iguales y la lista de etiquetas distintas con el texto del fragmento. Cada modelo cuesta una llamada. El informe de cada revisión guarda la etiqueta final de cada candidato (`labels_by_id`) y `agreement()` es una función pura probada con modelos simulados.

## Primera prueba de acuerdo (NMP 2024: `gemini-3-flash-preview` contra `gemini-3.5-flash-lite`)
Los dos terminaron completos y con el mismo formato (102/102 etiquetados, sin repreguntas ni rechazos): Preview 17 s y $0.3147 MXN, Flash-Lite 10 s y $0.2436. **Misma etiqueta en 78 de 102 fragmentos (76.5 %)**, misma aplicabilidad en 73 de esas 78, preguntas fijas iguales 8 de 9. Debajo del 90 % que nos habíamos puesto como referencia.

Casi todas las 24 diferencias eran Flash-Lite más generoso que Preview, y casi todas eran etiquetas sin palabras que las justifiquen: `funding_condition` en listas de gastos financiables, `project_requirement` en encabezados («Población objetivo») y descripciones sin deber, `category` en la descripción larga de una estrategia, `criterion` en descripciones de grupos de organizaciones. Había también fronteras genuinas (`procedure` contra `not_relevant`) y hasta casos donde Flash-Lite acertó más (`limit` en «el monto máximo a solicitar será…»).

Respuesta: **contratos por etiqueta en código** (`CONTRACTS` en `call_review.rs`), la generalización de lo hecho con `priority`. Una etiqueta solo vale si el fragmento trae al menos una de las señales que la justifican (`funding_condition`: deber, prohibición, tope o cifra; `project_requirement`: deber; `criterion`: deber, exclusión, participación, tope, documento o título de requisitos; `limit`: tope o cifra; `date`: fecha; `exclusion`, `priority` y `evaluation`: sus palabras) y una categoría no pasa de 160 caracteres. Si no, queda como `not_relevant` y se lista el desacuerdo. `labels_by_id` guarda la etiqueta ya con el contrato aplicado, que es lo que usa el producto, y por eso es lo que se compara entre modelos. Además, una fecha suelta que es el último día de una ventana del mismo tipo («Tienes hasta el 23 de mayo» ante «16 al 23 de mayo») ya no se agrega como otra fecha.

Se midió sobre 2024, que ya no es un documento reservado: la validación limpia del ajuste es repetir la prueba en NMP 2025 y 2026.

## Prueba de acuerdo sobre 2025 y 2026 (con los contratos de etiqueta)
- **NMP 2025:** misma etiqueta en 111 de 132 (84.1 %), misma aplicabilidad en 85 de esas 111, preguntas fijas 8/9. Preview 41 s y $0.4546 MXN, Flash-Lite 20 s y $0.3910.
- **NMP 2026:** 141 de 162 (87.0 %), misma aplicabilidad 80 de 141 (57 %), preguntas fijas 8/9. Preview 23 s y $0.4553, Flash-Lite 19 s y $0.3529.
- **NMP 2024:** Preview devolvió 503 a los 80 s y, como la prueba fija el modelo, no hubo comparación (Flash-Lite solo: 19 s, 6 criterios, 10 documentos, 10 fechas, 4 límites, igual que Preview en la corrida anterior).

Qué había detrás de las diferencias, y la respuesta (la regla de siempre: lo que el código puede decidir no lo decide el modelo):
1. **`applies_to` era lo que más variaba** (57 % de acuerdo en 2026) y el código lo puede leer: «nunca han recibido donativo», «organizaciones nuevas», «Categoría N», «en esta etapa»… ahora lo deduce `derive_applies` del texto del fragmento y de la línea que abrió su lista, y el modelo ya no lo contesta (el esquema no lo admite). Precedencia: organizaciones nuevas, categoría, etapa, todas.
2. **El índice y los títulos se tomaban como reglas.** Flash-Lite etiquetó líneas del índice de la p. 2 de 2025 («Requisitos de participación», «Criterios obligatorios que cumplir…») como `criterion`, y eso metía criterios falsos en la elegibilidad. Ahora el segmentador descarta las rachas de ≥ 5 líneas-título en las tres primeras páginas y trata un título corto solo entre líneas en blanco y seguido de un párrafo («Población objetivo») como encabezado de sección.
3. **Una oración cortada en cinco candidatos** (2026 p. 4, líneas separadas por líneas en blanco). Ahora un párrafo que termina sin puntuación y es seguido por otro que empieza en minúscula se une.
4. **El esquema numerado de 2026 era un solo bloque:** ahora cada etapa numerada («1. Postulación», «2. Registro…», «3. Entrega de recursos») es un candidato, como ya pasaba con «Etapa N».

Cobertura del segmentador tras los cambios: 27/27 (2026), 19/20 (2025), 19/19 (2024), 13/13 (JAP); los candidatos de 2025 bajan de 132 a 101.

Lo que queda de las diferencias son sobre todo fronteras (`procedure` contra `not_relevant`, `objective` contra `funder`) y alguna etiqueta que el modelo caro no vio y el barato sí (en 2026, «Solo puedes participar en una sola agenda de derechos»: Preview la dejó `not_relevant`). Esto último ya lo cubre el paso que lista para una persona un criterio de las reglas al que el modelo no da etiqueta de regla.

## Tercera prueba de acuerdo (con `applies_to` por código, índice y títulos fuera, oraciones unidas)
| | Misma etiqueta | Misma aplicabilidad | Preguntas fijas | Criterios / documentos / fechas / límites |
|---|---|---|---|---|
| NMP 2024 | 84/95 (88.4 %) | 84/84 | 8/9 | (5, 9, 10, 4) en los dos |
| NMP 2025 | 84/101 (83.2 %) | 84/84 | 7/9 | (13, 4, 5, 3) en los dos |
| NMP 2026 | 135/156 (86.5 %) | 135/135 | 9/9 | (14, 32, 5, 3) contra (13, 36, 5, 3) |

Tiempos y costos: Flash-Lite 8 a 54 s y $0.19 a $0.41 MXN; Preview 16 a 52 s y $0.28 a $0.35. Sin repreguntas ni rechazos en ninguna corrida. 2024 pasó de 76.5 % a 88.4 %, la aplicabilidad llegó al 100 % y el resultado de producto es idéntico en 2024 y 2025.

Lo que queda por fragmento es sobre todo ruido que no llega al producto: `procedure` contra `not_relevant` (la mitad de las diferencias de 2026), `objective` contra `funder` contra `not_relevant` sobre párrafos de presentación, `project_requirement` contra `not_relevant`. Y dos errores reales, uno de cada modelo: Preview llamó `procedure` a «No serán recibidos proyectos fuera de las fechas…» (es una exclusión) y Flash-Lite llamó `criterion` a una invitación («invitamos a todas las organizaciones…»).

Respuestas:
- **`procedure` sale del vocabulario** (13 etiquetas): ningún producto lo usaba y los modelos se partían en dos entre él y `not_relevant`. El prompt dice ahora que los pasos y avisos del trámite son `not_relevant`.
- **Las páginas del anexo de documentos no se envían** (las reglas lo leen entero): su columna de descripciones volvía de Flash-Lite como 5 documentos de más (36 contra 32). Lo que trae fecha, tope, exclusión o prioridad en esas páginas sí se envía.
- **Medida de acuerdo sobre lo que alimenta el producto** (`critical_rate`): de los fragmentos que al menos un modelo manda a criterios, documentos, fechas o límites, cuántos coinciden en esa clase. Discrepar en si un párrafo es el convocante o el objetivo no cambia nada de lo que un proyecto necesita; esta es la cifra que dice si el producto cambia al cambiar de modelo. La imprime la prueba en vivo.

## Cuarta prueba de acuerdo (sin `procedure`, sin el anexo de documentos en el envío)
| | Misma etiqueta | Alimentan el producto (criterios, documentos, fechas, límites) | Preguntas fijas | Producto final |
|---|---|---|---|---|
| NMP 2024 | 87/95 (91.6 %) | 30/37 (81.1 %) | 8/9 | (5, 9, 10, 4) contra (6, 10, 10, 4) |
| NMP 2025 | 82/101 (81.2 %) | 15/20 (75.0 %) | 9/9 | (11, 3, 5, 3) contra (12, 4, 5, 3) |
| NMP 2026 | 127/135 (94.1 %) | 29/33 (87.9 %) | 9/9 | (12, 35, 5, 3) contra (13, 35, 5, 3) |

Flash-Lite 9 a 15 s y $0.17 a $0.29 MXN; Preview 8 a 28 s y $0.18 a $0.51. Sin repreguntas ni rechazos. Por fragmento 2024 y 2026 pasan el 90 %; 2025 no. En lo que alimenta el producto el denominador es pequeño (20 a 37 fragmentos), así que cada discrepancia pesa 3 a 5 puntos: en cifras absolutas son 7, 5 y 4 discrepancias y el producto final difiere en como máximo 1 criterio y 1 documento por convocatoria.

Más de la mitad de las discrepancias que alimentan el producto eran fragmentos con una fecha o un tope («Asistir a la sesión informativa… el 10 abril a las 11:00 hrs», «Tienes hasta el 23 de mayo…», «El monto máximo… será visible en la Plataforma…») repartidos entre `date`, `limit` y `not_relevant`. Esas fechas y topes no dependen de la etiqueta (los leen las reglas sobre toda la página y las preguntas fijas con ids), así que **`date` y `limit` salen del vocabulario** (11 etiquetas) y las fechas y los topes se comparan por las preguntas fijas. La medida «alimentan el producto» pasa a ser solo criterios y documentos. Y el último día de una ventana («hasta el 23 de mayo» ante «16 al 23 de mayo») ya no se lista como algo leído aparte, que hacía diferir las preguntas fijas de 2024.

## Quinta prueba de acuerdo: el ruido de un solo modelo
| | Misma etiqueta | Criterios y documentos | Preguntas fijas | Producto final |
|---|---|---|---|---|
| NMP 2024 | 83/95 (87.4 %) | 14/19 (73.7 %) | 9/9 | (9, 10, 10, 4) contra (5, 9, 10, 4) |
| NMP 2025 | 81/101 (80.2 %) | 10/13 (76.9 %) | 8/9 | (12, 4, 5, 3) contra (13, 4, 5, 3) |
| NMP 2026 | 124/135 (91.9 %) | 18/26 (69.2 %) | 8/9 | (12, 33, 5, 3) contra (13, 38, 5, 3) |

No mejoró respecto a la anterior, y en 2024 empeoró (9 contra 5 criterios). La causa no es de un modelo contra otro: **Preview dio 5 criterios en una corrida y 9 en la siguiente, sobre el mismo PDF y con el mismo código** (p23-1, p23-2, p9-1 y p17-1 pasaron a `criterion`). Lo que veníamos midiendo como «diferencia entre modelos» mezcla la diferencia entre modelos con la diferencia entre dos corridas del mismo modelo, y ningún ajuste de prompt o de contratos puede bajar el desacuerdo entre modelos por debajo del ruido de uno solo.

Hasta ahora no se había fijado nada del muestreo (ADR-007: sin parámetros, los valores por defecto son los probados). Para una tarea de clasificación la aleatoriedad solo agrega ruido, así que se añadió una opción **opt-in**: `GeminiProvider::with_sampling(temperatura, semilla)`, apagada por defecto, que el servicio simulado acepta (y sigue rechazando todo lo no documentado) y que solo usan las pruebas en vivo con `CIMIENTO_REVIEW_TEMPERATURE` y `CIMIENTO_REVIEW_SEED`. Y se escribió `call_review_consistency_live`: el mismo modelo N veces sobre el mismo PDF, comparado consigo mismo, para conocer ese piso de ruido. Pendiente de medir: el ruido de Flash-Lite y de Preview con el muestreo por defecto, y si temperatura 0 con semilla lo baja sin degradar el resultado (la guía de Google para la serie 3 es no tocar la temperatura, así que se prueba primero con Flash-Lite).

## El ruido de Flash-Lite contra sí mismo (NMP 2024, 3 corridas)
| | Misma etiqueta | Criterios y documentos | Preguntas fijas | Producto final |
|---|---|---|---|---|
| Muestreo por defecto | 96.8 % | 93.3 % | 9/9 | (5, 9, 10, 4), (6, 9, 10, 4), (5, 10, 10, 4) |
| Temperatura 0 y semilla 7 | **100 %** | **100 %** | 9/9 | (6, 9, 10, 4) en las tres |

Con temperatura 0 y semilla las tres corridas salieron idénticas etiqueta por etiqueta y las tres tardaron 9 s (la de muestreo por defecto llegó a 56 s en una). Con el muestreo por defecto, Flash-Lite ya era poco ruidoso: las diferencias eran 1 a 4 fragmentos de frontera (`p27-2`, `p22-2`, `p22-3`, `p17-1`). **El ruido grande de la prueba anterior era de Preview** (5 y luego 9 criterios), que razona más y varía más; lo que se leía como diferencia entre modelos era en buena parte la variación de Preview consigo mismo.

Decisión: **la tarea `call.review` fija temperatura 0 y semilla 7** (`AiTask::sampling`), solo para esa tarea; las tareas que escriben para una persona conservan los valores del proveedor (ADR-007). Un proveedor puede ignorarlo (hoy solo Gemini lo manda) y las pruebas en vivo lo pueden cambiar con `CIMIENTO_REVIEW_TEMPERATURE` (`none` para no mandar nada, o un número). Pendiente: comprobar que Preview, un modelo de la serie 3 para el que Google recomienda no tocar la temperatura, no se degrada con ella, y repetir la prueba de acuerdo entre modelos con el muestreo fijado en los dos.

## Preview no tolera la temperatura 0
Tres corridas de `gemini-3-flash-preview` con temperatura 0 y semilla 7 sobre NMP 2024 terminaron las tres en `truncated` (42, 49 y 45 s): el modelo razonó hasta llenar todo el espacio de salida (9,000 de respuesta más 4,000 de razonamiento). Sin muestreo fijado, la misma tarea usa unos 5,000 tokens de salida en 17 a 28 s. Es la repetición en bucle que Google advierte para la serie 3 cuando se baja la temperatura. Fijar el muestreo para toda la tarea fue un error: sirve con Flash-Lite (96.8 % a 100 % de acuerdo consigo mismo) y rompe a Preview, que es el modelo fuerte.

Respuesta: `AiSettings::sampling_models` (hoy solo `gemini-3.5-flash-lite`) lista los modelos donde una temperatura fija se midió y no daña; para cualquier otro se usan los valores del proveedor, aunque la tarea pida muestreo. Un modelo se agrega a la lista solo después de medirlo con `call_review_consistency_live`. Además, `Truncated` pasa a ser un error por el que la cadena pasa al siguiente modelo (un modelo que entra en bucle no implica que el siguiente lo haga), y con eso el último recurso de la revisión, Flash-Lite, también cubre este caso. Las corridas truncadas se cobran: unas tres veces 13,000 tokens de salida.

## Preview con su muestreo natural (3 corridas, NMP 2024)
Las tres `used` (25, 27 y 25 s), **producto final idéntico en las tres: 5 criterios, 10 documentos, 10 fechas, 4 límites**; misma etiqueta 95.8 %, criterios y documentos 100 %, preguntas fijas 9/9. Las pocas etiquetas que cambian son `funding_condition` o `project_requirement` contra `not_relevant`, que solo alimentan el perfil. El salto de 5 a 9 criterios que se vio una vez en Preview fue antes de quitar `procedure`, `date` y `limit` y de excluir el anexo del envío, y con el riel actual no se reproduce.

## Sexta prueba de acuerdo (Preview con su muestreo natural contra Flash-Lite con temperatura 0)
| | Misma etiqueta | Criterios y documentos | Preguntas fijas | Producto final |
|---|---|---|---|---|
| NMP 2024 | 88/95 (92.6 %) | 14/17 (82.4 %) | 9/9 | (7, 10, 10, 4) contra (5, 9, 10, 4) |
| NMP 2025 | 85/101 (84.2 %) | 10/12 (83.3 %) | 8/9 | (11, 5, 5, 3) contra (12, 4, 5, 3) |
| NMP 2026 | Preview devolvió 503 | | | Flash-Lite solo: (14, 38, 5, 3) |

Tiempos y costos: Preview 24 a 25 s y $0.34 a $0.44 MXN; Flash-Lite 8 a 9 s y $0.17 a $0.19. La corrida de 2026 se detuvo por un 503 de Preview (la prueba fija el modelo y no cae a otro).

**Corrección de una lectura previa:** Preview dio 5 criterios en las tres corridas seguidas de 2024 (con pausas) y 7 en esta sesión: `p23-1` y `p23-2` («Organizaciones que no recibieron donativos… / que nunca han recibido donativo…») volvieron a ser `criterion`. Tres corridas seguidas estables no demuestran estabilidad entre sesiones. Esas dos frases describen grupos de organizaciones y pasaban el contrato por estar bajo un título de requisitos (`rule_section`). Ahora, si lo único que justifica un `criterion` es estar bajo ese título, el texto tiene que estar escrito como requisito (empezar con un infinitivo: «Ser…», «Tener…», «Contar…»).

**Estado del riel.** Cada modelo es reproducible por sí solo en lo que alimenta el producto (Flash-Lite con temperatura 0, 100 %; Preview, 100 % en criterios y documentos en una sesión), y entre los dos modelos el producto difiere en 1 a 2 criterios y 1 documento por convocatoria. Lo que queda son fronteras de juicio (descripciones de grupos, encabezados de tema, frases que son a la vez requisito y trámite). Con un solo modelo en producción la persona obtiene el mismo resultado al subir la misma convocatoria, que es lo que el riel debía garantizar; cambiar de modelo puede mover un par de elementos, que quedan marcados como supuestos de la IA por confirmar. Se da por cumplido; no se persigue el 90 % por fragmento entre modelos distintos.

## Cuánto importa la variación entre modelos (criterio con el que se cierra el riel)
Lo que se puede lograr es **reproducibilidad** (mismo modelo, misma convocatoria, mismo resultado: Flash-Lite con temperatura 0 llegó a 100 %) y no **equivalencia** entre modelos o entre días, porque el razonamiento cambia y hay frases genuinamente ambiguas. La pregunta útil no es cuánto coinciden sino qué se pierde cuando difieren. Las diferencias medidas caen en cuatro clases:

1. **Sin efecto:** etiquetas del perfil narrativo (`funding_condition` contra `not_relevant`, `funder` contra `objective`). La mayoría de las discrepancias.
2. **Elemento de más:** un criterio que no lo es (`p23-1`, `p23-2`). Entra marcado como supuesto de la IA, sin bloquear; la persona lo descarta al confirmar.
3. **Elemento de menos de poca importancia:** un documento condicional que un modelo vio y el otro no; sigue en la lista de dudas y en el anexo.
4. **Omisión de un requisito real** (una vez Preview dejó sin regla «Solo puedes participar en una sola agenda de derechos»). Es la única grave.

Por diseño la clase 4 es la más difícil de alcanzar: el modelo solo agrega y nunca borra (salvo una fecha leída dentro de un documento), de modo que todo lo que encuentran las reglas (36 de 36 hechos verificados en NMP 2026) está en el resultado pase lo que pase; lo que sí depende del modelo es lo que las reglas no leen (en NMP 2024, los 10 documentos); los contratos y la lista de desacuerdos señalan lo dudoso; y la confirmación final con la persona («esto es lo que entiendo de la convocatoria») es la red de seguridad.

El acuerdo entre modelos no mide corrección: dos modelos pueden coincidir en algo equivocado y el que discrepa puede ser el que acierta (Flash-Lite acertó con «No serán recibidos proyectos fuera de las fechas…» donde Preview se equivocó). Pendiente, la medida que sí dice si la variación es grave: **omisiones y sobrantes contra una referencia verificada** por convocatoria (ya hay 36 hechos para NMP 2026; faltan 2024 y 2025).

## Qué modelo hace el riel
**`gemini-3.5-flash-lite` primero, con temperatura 0 y semilla 7; `gemini-3-flash-preview` de respaldo.** Medido sobre las mismas convocatorias: Flash-Lite 8 a 10 s y $0.17 a $0.24 MXN contra 24 a 25 s y $0.34 a $0.54 de Preview; reproducible (100 % consigo mismo con temperatura 0); 500 llamadas al día contra 20; y mismos criterios, documentos, fechas y límites salvo uno o dos elementos de frontera. Preview no tolera la temperatura 0 (bucle de razonamiento), así que de respaldo trabaja con su muestreo natural. En el código: `AiTask::light_model_first` (la tarea sigue pidiéndose con la forma de petición con que se midió, la de tarea fuerte; solo cambia el orden de los modelos). Las tareas que escriben para una persona (preguntas, resumen, necesidades) conservan el modelo fuerte primero.

## ¿El riel oculta la convocatoria? (duda abierta al cerrar)
El riel ahorra dinero y da reproducibilidad, pero puede ahorrar también conclusiones: las que nacen de juntar varias partes del documento (una contradicción entre páginas, «esto no aplica a la Categoría 2», «piden evidencia y cotizaciones, así que el plan de pruebas debe ir por ahí»). El riel etiqueta fragmentos de una lista cerrada y entre el 30 y el 60 % queda como `not_relevant`; si algo importante cayó ahí, o ni llegó a ser fragmento, se perdió sin aviso. No es un problema de que el riel reemplace al texto (cada elemento lleva su página y su fragmento literal, y el PDF sigue disponible) sino de que hoy no hay manera de volver al texto original cuando una conclusión necesite matiz: eso lo cubre la búsqueda por significado (embeddings y RAG), que no es una mejora aparte sino la mitad que falta para que el riel sea un índice y no un sustituto.

Medida (`call_full_reading_vs_rail_live`): un modelo fuerte lee la convocatoria completa, sin riel, y escribe de 30 a 50 conclusiones; por cada una se comprueba si quedó en el producto del riel, solo en un fragmento que el riel no usó (lo perdió el etiquetado) o en ningún fragmento (lo perdió el segmentador, o nace de juntar partes). Cada lugar pide un arreglo distinto. Pendiente de correr.

## Resultado: ¿el riel oculta la convocatoria? (`call_full_reading_vs_rail_live`, NMP 2026 y 2024)
Un modelo fuerte (`gemini-3-flash-preview`) leyó la convocatoria completa y escribió 50 conclusiones por documento (19 s, 10,687 y 7,032 tokens de entrada). La primera medición salió mal: «en ningún fragmento» 23 y 22 de 50, porque comparaba solo contra una parte del producto y con palabras idénticas. Recalculada contra todo el producto (indicadores, rubros, enlaces, fechas, límites, figuras legales) y por raíces de palabra: **37 de 50 (74 %) en NMP 2026 y 31 de 50 (62 %) en NMP 2024 están en el producto**. De las que no, cerca de la mitad sí están pero como dato estructurado que la comparación de texto no ve (el 10 % de gastos administrativos es `max_admin_percent`, el DOF es un documento, el 23 de mayo es una fecha, el equipo de cómputo es un rubro, «Tips Anónimos» es un enlace).

Pérdidas reales:
- **Requisitos del proyecto que el modelo dejó como `not_relevant` en esa corrida** («los gastos de inversión deben ajustarse a los precios de mercado», «la propuesta debe incluir Teoría de Cambio, estrategias e indicadores», «la formación técnica debe estar certificada»). Pierde el etiquetado; en otras corridas esas frases salieron como `project_requirement`.
- **Definiciones de población por tipo de institución** («la atención en asilos se limita exclusivamente a personas adultas mayores de 60 años o más»): no tienen dónde caer ni en las reglas ni en el vocabulario. La que más importa para el mercado de Cimiento.
- **Descripciones de los documentos del anexo** («últimas tres actas firmadas, aunque no estén protocolizadas», «comprobante de domicilio no mayor a tres meses»): el anexo no se envía y las reglas guardan el nombre, no la descripción.
- **Descripciones de estrategias y procedimientos** («una propuesta por categoría», «crear un usuario y cargar la documentación desde cero»).

No se pierde: figuras legales, años de operación, autorización, topes, duración, ventanas de postulación ni la lista de documentos: todo lo que decide si una organización puede participar y qué debe entregar. Lo que se pierde es sobre todo cómo armar un buen proyecto y los matices de cada documento. La lectura completa no es la verdad (un modelo, una corrida, 50 conclusiones pedidas y 50 devueltas, un subconjunto que escogió): es una lupa, no un examen.

Decisiones: (1) las pérdidas son justo lo que se preguntaría en el chat y se contesta mejor volviendo al texto, así que la búsqueda por significado es la parte que el riel no puede ser; (2) las 100 conclusiones de estas dos lecturas, cada una con su página, sirven de preguntas de referencia para medir esa búsqueda; (3) pendientes pequeños del riel: etiqueta `population` («a quién atiende cada programa») y llevar la descripción de cada documento del anexo junto a su nombre.

## Actualización (ADR-014): el riel v2
Los contratos por palabras y la puerta de clasificación se quitaron: escritos leyendo el Monte de Piedad, descartaron el 48 % de las etiquetas del modelo en Alsea y dejaron a Bienestar en cero. El vocabulario pasa a 15 etiquetas (`population`, `funded`, `calendar`, `submission` nuevas) y el prompt a `call_review.v4`. Las secciones sobre contratos, `procedure` y la puerta de este documento describen la v1 del riel y quedan como historia. El código con contratos está archivado en `docs/codigo-archivado/`.
