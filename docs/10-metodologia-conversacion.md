# 10 · Metodología de la conversación y bitácora

**Estado:** parcialmente adoptada. El 2026-10-03 se aprobó la **convocatoria primero** y la **conversación progresiva guiada por la IA** (apertura + cinco porqués, solo con IA, interfaz de chat) en el `adr/ADR-017-conversacion-guiada-con-ia.md`, que ya está construido. Siguen **pendientes** de este documento: la bitácora completa con solidez por hecho (§4–5), el formulario tipado (§13), el plan de pruebas generado y la priorización como «elegir el objetivo». Donde este documento y el ADR-017 difieran, vale el ADR-017.

## 1. Por qué

La primera corrida real del caso dorado mostró lo que ya se sospechaba del diseño actual:

- **La IA no recuerda.** `next_question` recibe el perfil, el pedido inicial y solo las respuestas de *su* dimensión (`diagnosis_service.rs`). No sabe qué se dijo antes ni qué se va a preguntar después: en la dimensión 2 preguntó lo que cubre la 3.
- **No hay memoria estructurada.** Entre llamadas solo viaja texto crudo de preguntas y respuestas. Nada dice qué está establecido, qué es hipótesis, qué falta ni qué decisiones están abiertas. El resumen final se vuelve a deducir de cero.
- **No hay método.** Las 7 dimensiones son una lista de campos; la causa de fondo es una pregunta. La IA pregunta libremente dentro de una dimensión y el código solo cuenta repreguntas.
- **Medida real:** 11 llamadas, 5 de ellas repreguntas, y un resumen que omitió alternativas que el propio caso pedía.

## 2. Principios

1. **La convocatoria va primero.** Los proyectos nacen de un donativo posible. La convocatoria define el marco (qué financia, topes, población, criterios) y desde ahí se baja a lo particular.
2. **Una llamada, dos productos.** Cada respuesta de la IA es un solo JSON con el *mensaje* para la persona y el *registro* de lo que esa vuelta aporta. El código guarda el registro; no hay llamada de extracción aparte.
3. **La IA lleva la conversación; el código garantiza que termine.** De cara a la persona, la IA toma la iniciativa (propone hipótesis, ofrece opciones, avisa el avance, contesta dudas y vuelve al hilo). Por debajo, el código decide en qué fase y hueco se está, cuánto presupuesto de vueltas queda y cuándo se cierra. La IA redacta la pregunta de *ese* hueco y etiqueta lo que se dijo. Si se desvía, el código la corrige o descarta. Ver §6.1.
4. **Nada sin evidencia.** Todo hecho del registro lleva una cita textual de lo que dijo la persona; el código comprueba que la cita exista en su texto. Lo que no la tenga baja a hipótesis (`ai_assumption`). Los números siguen la revisión de `domain/figures.rs`.
5. **La persona confirma.** Su texto literal se guarda siempre aparte (origen `user`). Lo que la IA entendió se muestra («Lo que entendí hasta ahora») y la persona lo confirma o corrige; hasta entonces es `ai_assumption`.
6. **Agnóstica del tema.** Las fases y etiquetas no dependen del proyecto (baño, comedor, medicinas). El tema entra por la convocatoria y por lo que la persona cuenta.
7. **Cuántos, nunca quiénes.** Todo texto del registro pasa por el escáner antes de guardarse, como cualquier otro.

## 3. Flujo de etapas

Hoy: `PROFILE → DIAGNOSIS → PRIORITIZATION → CALL_SELECTION → DRAFTING → REVIEW → READY`.

Propuesto: `PROFILE → CALL_SELECTION → DIAGNOSIS → PRIORITIZATION → DRAFTING → REVIEW → READY`.

- `CALL_SELECTION` pasa antes: se elige la convocatoria y se confirman sus requisitos (`call_template`). Sus requisitos tipificados entran al registro como **restricciones**.
- La verificación de encaje con el checklist (monto, beneficiarios, documentos) se hace en dos momentos: al inicio, como restricciones que orientan las preguntas, y antes de redactar, cuando ya existen los números del proyecto.
- `PRIORITIZATION` deja de ser «qué necesidad elegir entre varias» y pasa a ser **elegir el objetivo**: la problemática raíz que el diagnóstico encontró, con los problemas secundarios que resolvería (§6). El puntaje por criterios del código se conserva.
- Cambia `domain/stage.rs` (orden y condiciones, con sus pruebas), la interfaz y `02-flujo-funcional.md`.

## 4. El turno: un JSON, dos productos

El prompt del sistema (fijo, versionado, cacheable) define para siempre la estructura de cada respuesta. El código le dice en cada llamada **en qué hueco está** y le pasa el resumen de la bitácora (§5). La salida validada:

```json
{
  "message": "Lo que ve la persona: la siguiente pregunta, o el cierre.",
  "record": {
    "answered":       [{ "topic": "causa", "text": "Lo que dijo, en una frase fiel", "quote": "fragmento literal" }],
    "key_points":     [{ "text": "Dato o idea que importa para el proyecto", "quote": "fragmento literal" }],
    "objective":      { "text": "Objetivo del proyecto como se entiende hoy", "status": "draft" },
    "to_review":      [{ "text": "Dato dudoso o contradicción con lo dicho antes", "quote": "fragmento literal" }],
    "open_decisions": [{ "text": "Decisión que la persona todavía debe tomar", "options": ["A", "B"] }],
    "closes":         ["causa_raiz"],
    "targets":        "indicador_exito"
  }
}
```

Las etiquetas son las de la conversación de trabajo: **respondió**, **punto clave**, **objetivo**, **punto a revisar**, **decisiones abiertas**. `closes` y `targets` existen para que el código sepa qué hueco se cerró y a cuál apunta el mensaje.

Validaciones en código (si fallan, un reintento con el error, como hoy; si falla otra vez, se sigue sin registrar y se avisa):

- `targets` debe ser exactamente el hueco que el código asignó.
- Cada `quote` debe aparecer (normalizada) en el texto de la persona de esa vuelta; si no, esa entrada pasa a hipótesis.
- Los números de `record` y de `message` deben aparecer en lo que dijo la persona o en la convocatoria.
- `closes` solo acepta huecos con al menos una entrada con cita válida.
- Escáner sobre todo el texto antes de guardar.

## 5. La bitácora

La IA no «tiene memoria»: la memoria es una estructura del proyecto, del código, que se le muestra cada vez.

**Entradas** (tabla `ledger_entry`, bosquejo): `id`, `project_id`, `turn`, `kind` (`answered`, `key_point`, `objective`, `to_review`, `open_decision`, `constraint`), `topic` (hueco), `text`, `quote`, `answer_id` (la respuesta literal de donde sale), `status` (`open`, `resolved`, `discarded`), y los campos de siempre de origen y confirmación (`origin`, `source_ref`, `confirmed_at`, `confirmed_by`). Aparte, el registro de turnos (`conversation_turn`: hueco asignado, mensaje, JSON recibido, modelo) para auditar y medir.

**Resumen para la llamada** (lo arma el código, unas pocas centenas de palabras, no el historial): objetivo actual y su estado; por cada hueco cerrado, su punto clave en una línea; puntos a revisar abiertos; decisiones abiertas; restricciones de la convocatoria; las dos últimas vueltas literales; el hueco asignado y qué se espera. Es corto a propósito: cabe entero siempre, por eso no se necesita recuperación vectorial para esto.

**RAG vectorial solo para texto grande:** convocatorias y documentos de proyectos pasados (FTS5 y `sqlite-vec` ya están en la pila; Fase 3). La bitácora estructurada no se recupera, se incluye completa.

## 6. Las fases (de lo general a lo particular)

El código recorre las fases en orden; cada una tiene huecos y un criterio de salida que **el código evalúa sobre la bitácora**, no la IA.

| Fase | Huecos | Sale cuando |
|---|---|---|
| 0 · Marco | Restricciones de la convocatoria (monto, población, uso permitido, criterios) | Convocatoria confirmada y restricciones cargadas |
| 1 · Situación | `problema`, `afectados` (grupo y cifra), `consecuencia_hoy` | Los tres con cita válida |
| 2 · Raíz | `cadena_causal`, `causa_raiz` | Hay una causa que está en manos de la institución y se puede medir, o la persona no sabe más, o se llegó al tope de niveles |
| 3 · Objetivo | `objetivo`, `indicador_exito` | Objetivo en estado firme (confirmado por la persona) con indicador |
| 4 · Alcance | `intentos_y_alternativas`, `efectos_secundarios`, `encaje_con_restricciones` | Alternativas distintas entre sí registradas y sin chocar con restricciones bloqueantes |
| 5 · Viabilidad | `relacion_con_mision`, `sostenibilidad` | Ambos con cita válida |

- **Suficiente** = todos los huecos obligatorios cerrados. Un tope de vueltas (a decidir, §11) corta la conversación; lo no cubierto pasa a `open_questions`.
- **Elección del hueco:** el código toma el primer hueco abierto de la fase, ajustado por las restricciones (por ejemplo, si la convocatoria exige un mínimo de beneficiarios, `afectados` sube de prioridad). La IA no elige.
- **Una respuesta puede cerrar varios huecos** (`closes`), con evidencia. Así no se repregunta lo ya dicho.
- **Problemas secundarios:** al terminar, el código revisa qué entradas de `consecuencia_hoy` y `efectos_secundarios` dependen de la raíz y las lista como lo que se resolvería en cascada.
- Los 5 porqués son una técnica que la IA puede usar *dentro* de la fase 2, no la columna vertebral.

**Correspondencia con el resumen actual** (la salida de `02` no cambia): `problem_statement` ← problema; `affected` ← afectados; `current_consequences` ← consecuencia_hoy; `root_causes` ← causa_raiz y cadena; `reframed_need` ← objetivo; `alternatives` ← alternativas; `suggested_indicators` ← indicador_exito; `open_questions` ← huecos sin cerrar, puntos a revisar y decisiones abiertas. El resumen se arma desde la bitácora, con menos texto de entrada y sin volver a leer toda la conversación.

### 6.1 Convergencia: la IA lleva la conversación y el código garantiza el final

**Premisa:** nunca se sabe qué va a contestar la persona. El caso dorado, con respuestas escritas de antemano, es el camino feliz: sirve de comprobación mínima, no de prueba de que el método converge. Por eso la garantía de que la conversación termine **no depende de la buena voluntad del modelo ni de un guion**: la dan un presupuesto y una medida de progreso que calcula el código.

**Cómo lleva la IA la conversación (lo que ve la persona):**
- Prefiere **proponer para confirmar** antes que preguntar en abierto cuando ya tiene base: «Por lo que cuenta, la raíz parece ser X. ¿Es así?» cuesta menos responder que «¿por qué cree que pasa?».
- Cuando la persona está atorada, ofrece **opciones cerradas** (A, B, «no lo sé»).
- Avisa el avance («vamos en el paso 3 de 6») para que la persona sepa que hay un final.
- Si la persona pregunta algo, contesta breve **solo con lo que está en la convocatoria y el perfil** (sin inventar) y regresa al hilo.

**Lo que garantiza el código (independiente de lo que conteste la persona o la IA):**
1. **Tope global de vueltas (T)** y **máximo 2 intentos por hueco**. Cada vuelta gasta presupuesto, sin excepción.
2. **Medida de progreso:** huecos cerrados o avanzados. Una vuelta que no cierra ni avanza ningún hueco suma a un contador de estancamiento. Con **2 seguidas**, el código impone a la IA una **táctica más fácil** en la siguiente vuelta: pregunta abierta → hipótesis para confirmar → opciones cerradas → dejar el hueco pendiente.
3. **Nada bloquea.** Un hueco que no se cierra tras sus intentos pasa a `open_decisions` / `open_questions` con lo poco que sí se sabe, y la conversación sigue al siguiente.
4. **Cierre garantizado:** al cerrar lo obligatorio o agotar T, el código termina y arma el resumen con lo que hay.
5. **Peor caso acotado:** T vueltas (no mil), y además corre el tope mensual de gasto que ya existe.

**Respuestas imprevisibles y qué hace el sistema:**

| La persona… | Qué pasa |
|---|---|
| Contesta vago o «no sé» | Cuenta como intento del hueco; baja la táctica (hipótesis, opciones); si persiste, el hueco queda pendiente |
| Contesta fuera de tema | La IA lo reconoce, lo registra si sirve para otro hueco y repite el hueco de otra forma una vez |
| Contesta varios huecos de golpe | `closes` con evidencia; esos huecos no se vuelven a preguntar |
| Se contradice con algo anterior | `to_review`; la IA pide aclararlo (cuenta contra el tope) |
| Pregunta de vuelta | Respuesta breve con lo que consta; el hueco sigue asignado |
| Cambia el objetivo a medio camino | Se registra, se vuelve a la fase de raíz u objetivo y consume vueltas del mismo tope |
| Escribe datos de personas | Cuarentena, como hoy |
| Deja la conversación a medias | Todo queda guardado; se retoma en el mismo hueco |

**Cómo se prueba sin conocer la respuesta:** además del caso dorado, una **batería de personas simuladas** contra el servicio falso (después, unas pocas corridas reales): parca, divagante, evasiva, contradictoria, que pregunta de vuelta, que contesta varios huecos a la vez, que mete datos personales y que cambia de objetivo. Para **todas** debe cumplirse: termina en T vueltas o menos; ningún hueco queda abierto sin pasar a pendiente; cero cifras inventadas; todas las citas verificadas; ningún mensaje repite un hueco cerrado. Límite honesto: esas personas las escribimos nosotros, no sustituyen una **prueba piloto con una directiva real**, que es el único juez de verdad. Se planea una al terminar la rebanada 2.

## 7. Memoria para otros proyectos

La bitácora se exporta como JSON (`institution_memory.v1`): por proyecto, la convocatoria, el objetivo, la causa raíz, los hechos confirmados, las decisiones y las lecciones; y aparte, los hechos de la *institución* (antigüedad del edificio, capacidad).

- Solo entra lo **confirmado** por la persona, ya revisado por el escáner, con fecha y versión.
- Al usarlo en otro proyecto entra como **propuesta a reconfirmar** (origen `prior_project`), porque los datos envejecen.
- Los hechos del proyecto no se mezclan con los de la institución salvo que la persona los promueva.

## 8. Costo

Estimaciones, a medir (partiendo de las mediciones reales del ADR-009: una pregunta ligera ≈ 750 de entrada y 30 de salida ≈ $0.006 MXN):

- El registro agrega ≈ 150–250 de salida y el resumen de bitácora ≈ 400–800 de entrada por vuelta: unos **$0.01 MXN extra por vuelta** en el nivel ligero. No hay llamada de extracción, así que el costo no se duplica.
- Se compensa con **menos vueltas** y con un resumen final más barato (parte de la bitácora, no de toda la conversación).
- El prompt fijo con la metodología se cachea.

## 9. Métricas nuevas

Vueltas hasta suficiencia (la medida de «menos iteraciones»), huecos cerrados por vuelta, proporción de entradas con cita rechazada, tamaño del resumen de bitácora enviado, y vueltas que repiten un hueco ya cerrado. Se suman al reporte de `ai::metrics`.

## 10. Plan por rebanadas

1. **Contrato, bitácora y método sobre el caso dorado**, con una convocatoria ficticia en `fixtures/` y su resumen de restricciones como entrada. Primero con la simulación gratuita (el servicio falso hace de modelo y se mide cuántas vueltas hacen falta); después una corrida real. Es la rebanada que contesta si el método funciona.
2. **Reordenar etapas y pantalla**: convocatoria primero y panel «Lo que entendí».
3. **Lectura de convocatorias** (Fase 3): PDF → fragmentos → requisitos → confirmación humana, que sustituye la entrada ficticia de la rebanada 1.
4. **Resumen desde la bitácora** y elección del objetivo en lugar de la priorización actual.
5. **Exportar e importar la memoria**.

Criterios de aceptación de la rebanada 1: **(a)** sobre el caso dorado (camino feliz): suficiencia en menos vueltas que las 11 llamadas / 5 repreguntas de hoy (meta a fijar, §11), los criterios del caso dorado igual o mejores que los del ADR-009 y ninguna pregunta repetida de un hueco ya cerrado; **(b)** sobre la batería de personas simuladas (§6.1): todas terminan en T vueltas o menos, sin huecos abiertos sin pasar a pendiente; **(c)** en todos los casos: cero cifras inventadas y todas las entradas de hechos con cita verificada. Una prueba piloto con una persona real cierra la rebanada 2.

## 11. Decisiones abiertas

- **Tope de vueltas T y meta típica** (propuesta: T = 15 vueltas duras, 2 intentos por hueco, meta de 8 a 10 en una persona cooperativa; hay unos 12 huecos obligatorios y una respuesta puede cerrar varios).
- **Cuántas respuestas seguidas sin progreso** antes de bajar la táctica (propuesta: 2).
- **Entrada de la convocatoria en la rebanada 1:** un resumen de restricciones en fixture (propuesto) o texto pegado.
- **Si la persona ve la bitácora en vivo** (propuesta: sí, plegable) o solo al final.
- **Qué pasa con la priorización:** si se queda un paso para elegir entre objetivos candidatos cuando el árbol de causas tiene más de una raíz.
- **Qué etiquetas más se quieren**; se parte de las cinco indicadas.

## 12. Riesgos

- **El guion del caso dorado es camino feliz.** Una persona real puede ser vaga, divagar o contradecirse; sin presupuesto de vueltas y táctica obligatoria la conversación podría no terminar. Mitigación: §6.1, batería de personas simuladas y una prueba piloto real.
- **La interpretación de la IA puede deformar lo dicho.** Mitigación: cita verificable, texto literal siempre guardado, confirmación de la persona.
- **Un solo JSON hace dos trabajos.** Si el modelo es flojo, puede atender peor la pregunta o el registro. Se mide con las métricas de §9 y con el servicio falso antes de gastar cupo.
- **Adelantar la convocatoria** obliga a la lectura de PDF antes que el diagnóstico (rebanada 3). Por eso la rebanada 1 usa una entrada ficticia.
- **Ajustar el problema a lo que financia el donante.** Mitigación: la convocatoria entra como restricciones y vocabulario; la raíz se guía por hechos citados, no por la convocatoria.

## 13. Cambios acordados tras la simulación

El ensayo con la convocatoria real de Nacional Monte de Piedad (`docs/11-simulacion-nmp-2026.md`) confirmó el método (raíz encontrada en la vuelta 3, objetivo firme en la 5) y agrega estos cambios a lo descrito arriba. Se integran al cuerpo del documento cuando se apruebe el ADR-010.

- **§4 El turno:** cada entrada del registro lleva `solidez` (observado, referido, supuesto, verificado). La salida suma `questions`: las preguntas como campos tipados (texto, opciones, número, sí o no) con «no sé» y «después».
- **§5 La bitácora:** una tabla de preguntas hechas con su estado (respondida, parcial, saltada, declinada); el código reabre las no cerradas. La confirmación es siempre explícita.
- **§6 Fases:** la apertura de la conversación es fija y de dos partes (resultado y obstáculo). Todo lo que el perfil de la institución ya contiene se confirma, no se pregunta; el código marca contradicciones entre la conversación y el perfil.
- **Plan de pruebas (nuevo):** lo genera el código desde los hechos críticos por debajo de «verificado»; la IA lo redacta con qué es, quién lo hace y qué documento queda.
- **Perfil de la institución ampliado (cambio a `05-modelo-datos.md`):** datos legales, instalaciones con detalle, ingresos por donante con tipo, alianzas, línea base y vigencia de documentos, con fecha de última confirmación por dato.
- **Extractor determinista de convocatorias (Fase 3, adelantada):** PDF con texto, PDF escaneado con OCR, formularios y complementos enlazados → JSON con página y fragmento de origen; la IA solo para lo ambiguo, con confirmación humana. El monto y otros datos que no están en el documento son campos obligatorios aparte, y las fechas se comparan con hoy.
- **Formulario progresivo (aclarado por Jaime):** el **código** ya tiene la estructura del formulario y la **IA solo la rellena**. Cada campo es una pregunta que la IA llena, con sus **opciones cerradas** si las ofrece y siempre un campo de **respuesta propia** (opciones, abierta, o ambas). La persona elige una opción o escribe la suya, como en un cuestionario de elección que se rellena en vivo. Se muestra de una en una, sin abrumar y sin depender de la memoria de la persona. El contrato de salida de la IA incluye por cada pregunta: texto, tipo (opciones, abierta o ambas), opciones, «no sé» y «después».
