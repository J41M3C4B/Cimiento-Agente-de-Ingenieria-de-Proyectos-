# ADR-021 · El borrador lo prepara la IA; la persona revisa, pone costos y corrige

**Estado:** Aceptada (2026-10-04), a petición de la persona dueña del proyecto. Implementada: `drafting_service.rs` (`prepare_plan`, `draft_all`, `confirm_all_texts`), tareas `drafting.plan` y `drafting.all`, prompts `drafting_plan.v1` y `drafting_all.v1`, migración `0012_drafting_plan.sql`, y las pantallas de la conversación, el resumen, los objetivos y la redacción.

## Contexto
La redacción pedía a la persona revisar sección por sección (una decena de formularios), escribir desde cero el presupuesto y el cronograma, y leer frases de la convocatoria que a veces no se entendían (por ejemplo «El proyecto deberá contemplar una visita de seguimiento…»: ¿afirmación, pregunta, algo que escribir?). La IA ya sabía el problema, la causa de fondo y el objetivo, pero no se usaba para adelantar nada de eso.

## Decisión
1. **Al llegar a la redacción, una sola llamada prepara todo lo que la IA ya sabe** (`drafting.plan`, una vez por proyecto): para cada sección un título corto y una explicación **en sencillo** (si es un texto, un dato o una condición que solo hay que aceptar), las **partidas del presupuesto sin precio** y un **cronograma propuesto**. Lo guarda el código y lo marca como propuesta (`origin = ai_assumption`).
2. **El código valida lo que propone la IA.** Una cantidad solo se conserva si alguien la dijo (misma comprobación de cifras que ya existía); los meses deben ser válidos y caber en la duración máxima de la convocatoria (o 24); nunca pisa partidas ni actividades que la persona ya tenga.
3. **La persona solo escribe costos.** Las partidas llegan con costo 0; el presupuesto **no se puede confirmar** mientras alguna no tenga costo (`BudgetIncomplete`). Al tocar una línea pasa a ser de la persona.
4. **Los textos se redactan solo si la persona lo pide** (`drafting.all`): una llamada con **todo el esquema** de secciones, como **borrador completo** o como **guía breve** (3 a 5 puntos por sección). También puede escribirlos ella. Respeta lo que la persona escribió o confirmó, repite una vez si hay cifras que nadie dio y las marca en la sección. **Confirmar lo que está listo** se hace en bloque; lo que lleva cifras dudosas o quedó por revisar se mira una por una.
5. **Lo necesario para el presupuesto sale de la conversación y del objetivo elegido**, no de preguntas nuevas: como el objetivo se elige después del chat, la propuesta se hace al entrar a la redacción con el resumen, la causa de fondo y ese objetivo.
6. **Pantallas:** el diagnóstico es una pantalla dividida (la convocatoria en partes que se abren, con las fuentes como marcas con nota; la conversación con animaciones de «pensando» y «escribiendo»); el resumen del diagnóstico se abre en una ventana, en partes que se abren; desde el paso 2 la convocatoria queda en una línea que abre su resumen; el objetivo se elige con un clic y la calificación detallada queda plegada; la redacción tiene tres pestañas (presupuesto, cronograma, textos).

## Consecuencias
- Dos llamadas a la IA más por proyecto (una de preparación, una de redacción si se pide), con tope de salida de 4,000 y 9,000 tokens. Probadas con el modelo simulado (`cargo test drafting_service`); falta correrlas contra Gemini real y medir el consumo antes de usarlas con datos de verdad.
- Si la IA no está disponible, nada se guarda y la persona puede reintentar o seguir a mano como antes.
- Pendiente: que el prompt de la lectura de la convocatoria clasifique sus requisitos (texto, dato, condición) y no solo la IA de la redacción; hoy lo hace la explicación en sencillo.
