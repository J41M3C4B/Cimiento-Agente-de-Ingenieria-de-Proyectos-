# ADR-023 · Los procesos de IA sobreviven a la navegación y la IA lee una ficha completa de «Mi institución»

**Estado:** Aceptada (2026-10-04), a raíz de las pruebas del flujo completo con proyectos propios. Alcance: diagnóstico (conversación), resumen y paso a objetivos.

## Contexto
Tres problemas al probar de principio a fin:
1. Al armar el resumen solo cambiaba el texto del botón; el chat no mostraba que la IA trabajaba y parecía atorado.
2. Al cambiar de sección durante un proceso, parecía que se cancelaba y había que volver a pedirlo. No se cancelaba nada: Rust corre cada llamada hasta el final y guarda el resultado. Lo que se perdía era el estado «en curso» de la pantalla (`useState`/`useRef`), la página (`App.tsx` desmontaba «Proyectos» y perdía el proyecto abierto) y el backend no sabía qué corría ni evitaba duplicados: un clic de más hacía una segunda llamada (cuota doble) y podía sobrescribir el resumen ya confirmado.
3. La IA «olvidaba» o inventaba datos de la institución conforme avanzaban las etapas. No hay recuperación (RAG) de la institución: es un volcado de texto reconstruido en cada llamada, y era pobre (sin presupuesto, ingresos, dependencia, edades, estado ni notas de instalaciones; con valores en inglés; sin distinguir «cero» de «no capturado»). Las etapas posteriores no veían la conversación, solo el resumen redactado por la IA.

## Decisión
1. **Registro de procesos en Rust** (`jobs.rs`): un proceso de IA por proyecto a la vez (`turn`, `summary`, `needs`), en memoria (la llamada muere con el programa). Un segundo intento recibe `already_running`, que la pantalla trata como «esperar», no como error. El comando `project_job` dice qué corre y cómo terminó el último (una sola vez). La pantalla (`useProjectJob`) lo pregunta al volver y, mientras corre, lo vuelve a preguntar cada 1.5 s; al terminar relee lo escrito.
2. **«Proyectos» siempre montado** (`App.tsx`, oculto con `hidden` en otras secciones): se conservan el proyecto abierto y el estado local. Las demás páginas siguen desmontándose (la de IA hace sondeo y no debe correr oculta).
3. **El resumen no sobrescribe lo de la persona:** si ya está confirmado o lo editó ella, un pedido tardío o repetido responde `Skipped` sin llamar a la IA.
4. **Animación del resumen en el chat** (`tail` de `DiagnosisPanel` con `Thinking`), y al terminar el chat lo dice y ofrece abrirlo.
5. **Ficha de la institución** (`institution_context.rs`, reemplaza `profile_context`): todo lo capturado que puede llegar a la IA, en español sencillo y agregado; cada sección vacía dice «no capturado» y se avisa que eso no es cero; las sumas las hace el código. Cabecera «confirmados» o «BORRADOR». **Nunca:** nombres, contacto, RFC, sueldos, ni cuánto paga cada persona (solo cuántas pagan cuota; ADR-020), ni el rango de edades de un grupo de una sola persona. No lleva números incidentales (versiones, fechas), porque la revisión de cifras trata todo número de la ficha como dicho por la persona.
6. **Reglas de fuente** (`institution_rules.v1.md`, solo para las tareas que reciben el perfil, no para la lectura de convocatorias): el perfil es la única fuente de hechos de la institución; «no capturado» no es cero; lo que falta se declara o se pregunta, no se supone; los borradores de IA son propuestas, no hechos; las contradicciones se preguntan. Prompts nuevos: `conversation_turn.v2`, `diagnosis_summary.v3`, `prioritization_propose_needs.v4`.
7. **Los objetivos leen lo que dijo la persona** (`person_words`) además del resumen, y cada bloque del contexto dice qué tan confiable es.

## Consecuencias
- Sin snapshot del perfil por proyecto (ADR-016: institución global, proyecto independiente): la cabecera «BORRADOR/confirmados» deja ver si lo que lee la IA ya fue confirmado.
- El resto de las etapas (redacción) recibe la ficha nueva sin cambios, pero sus propuestas de IA (presupuesto, cronograma) siguen pasando a etapas posteriores como datos del programa: queda para la siguiente ronda.
- Probado en seco (sin cuota): `cargo test jobs`, `cargo test institution_context`, `cargo test diagnosis_service` (incluye `every_ai_call_of_the_first_steps_reads_the_whole_institution`). Falta la corrida real con Gemini sobre los proyectos de la persona.
