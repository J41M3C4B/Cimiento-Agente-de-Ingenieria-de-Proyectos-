# ADR-034 · La IA en todo el ERP: un equipo de agentes con herramientas, conocimiento recuperable y un asistente siempre a la mano

**Estado:** aceptada (2026-10-09); se aplica por bloques (IA1 a IA8). Amplía el principio 1 (qué hace la IA) y el ADR-023 (la ficha de la institución); no reemplaza los principios 2 a 8.

## Contexto

Hoy la IA trabaja en línea recta: cada tarea (`conversation.turn`, `diagnosis.summary`, `drafting.*`, `call.*`) recibe un volcado completo de la institución (`core::ai_sheet`, unos miles de caracteres) y devuelve un JSON. Funcionó para armar proyectos, pero no escala al ERP:

- **La IA no puede pedir lo que necesita.** Recibe todo o nada: la ficha completa aunque solo haga falta un dato, y nunca el contenido de los documentos de la institución (nadie los lee; ver `docs/14-auditoria-nucleo.md` §3.3).
- **Cada módulo nuevo engorda la ficha.** Con Donantes, el expediente y los historiales, el volcado crece en cada llamada.
- **La persona no tiene a quién preguntar.** Las directivas y religiosas tienen poca o nula práctica con la computadora. Hoy, cuando no entienden un campo, llaman al técnico. Los textos de ayuda fijos no alcanzan: necesitan preguntar con sus palabras, en la pantalla donde están, y recibir una respuesta que conozca su institución.
- **Hay trabajos distintos que hoy no tienen dueño:** leer un acta y proponer datos, llenar un formulario a partir de lo que la persona cuenta, investigar en los datos de la institución. Un solo prompt que hace de todo sería caro, difícil de probar y peligroso.

Lo que ya existe y se aprovecha:
- **El recorrido de cada llamada** (`ai/pipeline.rs`): escáner, proveedor con cadena de respaldo, validación contra esquema, ritmo por modelo, tope mensual y registro en `ai_usage`.
- **Embeddings y búsqueda por significado** (`documents/canonical/retrieve.rs`), hoy en memoria y solo para convocatorias.
- **Verificación de citas por código** (lectura canónica, ADR-015).
- **Revisión de cifras** (`ai/figures.rs`): la IA no puede decir un número que no esté en su contexto.
- **`sqlite-vec` y FTS5** sobre la base cifrada (probado en la Fase 0).
- **El registro de procesos** (`jobs.rs`, ADR-023), que evita llamadas dobles.

## Decisión

### 1. Un equipo, no un modelo que hace todo

La IA se organiza como un equipo pequeño. Cada **agente** tiene un oficio, un nivel de modelo, una lista cerrada de herramientas, un tope de pasos y un esquema de salida. Los define el código, no la IA.

| Agente | Oficio | Nivel | Herramientas | Qué entrega |
|---|---|---|---|---|
| **Asistente** | La puerta de entrada en toda la app. Responde «¿qué pongo aquí?», «¿para qué sirve?», «¿cómo hago…?» y «¿dónde está…?», sabiendo en qué pantalla y en qué campo está la persona | Ligero | manual, catálogo de campos, estado de llenado, resumen de la institución, delegar | respuesta con sus fuentes; botones «Ir a…» o «Llenarlo por mí» |
| **Capturista** | Propone valores para un formulario a partir de lo que la persona cuenta en el chat o de lo que ya está en otra parte del ERP | Ligero | catálogo de campos, datos de la institución | **propuestas** de valor por campo, nunca un guardado |
| **Lector** | Lee documentos de la institución y propone datos (el acta, la constancia fiscal, los oficios). Las reglas fijas van primero y la IA solo lee lo que no tiene formato | Ligero; fuerte si hace falta | fragmentos de un documento, catálogo de campos | propuestas con cita (documento y página) verificada por el código |
| **Investigador** | Responde preguntas sobre la institución cruzando módulos («¿nos alcanzan los baños para las personas que atendemos?») | Fuerte | indicadores y agregados de cada módulo, búsqueda en el expediente | respuesta con sus fuentes; cada cifra sale de una herramienta |
| **Proyectista** | Lo que hoy hace Proyectos: conversación, diagnóstico y redacción | Como hoy | ficha de la institución, herramientas de consulta, convocatoria | lo de siempre (ADR-017, ADR-018) |
| **Lector de convocatorias** | La lectura canónica (ADR-015) | Como hoy | — | sin cambio |

- **Un solo nivel de delegación:** la persona habla con el Asistente; el Asistente puede pasarle un encargo a un especialista. Un especialista no delega a otro. Así el costo y el flujo tienen tope y se pueden seguir.
- **El código elige al agente,** no un modelo «orquestador». La pantalla dice desde dónde se pregunta (`screen`, `field`) y Rust decide quién responde. Una pantalla de Proyectos sigue usando al Proyectista; el panel del asistente usa al Asistente.
- **Donantes** (cuando exista) tendrá su agente con la misma forma.

### 2. Las herramientas: la IA pide, el código decide y responde

- **La IA solo pide.** Responde un JSON con `{"tool": nombre, "args": {...}}` o con su respuesta final. Rust valida los argumentos contra el esquema de la herramienta, revisa que el agente la tenga permitida, revisa el permiso de la persona (ADR-028), la ejecuta y le devuelve el resultado. Tope de pasos por agente (de inicio, 4 para el Asistente) y de tokens por encargo.
- **Protocolo en JSON sobre el recorrido de siempre,** no la función nativa de cada proveedor. Así cada paso pasa por el escáner, la validación y `ai_usage`, y funciona igual con Gemini, Anthropic y Ollama. Si la medición muestra que la función nativa es claramente mejor, se cambia en un ADR nuevo (principio 7).
- **Las herramientas son de solo lectura.** Ninguna borra, guarda ni manda nada fuera.
- **Lo que la IA quiere cambiar es una propuesta.** Se guarda en `ai_proposal`, que lleva: destino (formulario, registro y campo), valor, `origin` (`ai_assumption` o `document`), `source_ref`, agente, estado (pendiente, aceptada o rechazada), quién la resolvió y cuándo. La persona la acepta con un clic y entonces corre el guardado de siempre, con su validación y su escáner. **La IA sigue sin escribir** (principio 3).
- **Quién define cada herramienta (ADR-032):** la base define el tipo `Tool` y el bucle de agentes. El núcleo registra las herramientas de la institución y de los módulos, porque los conoce por su `api`. Proyectos registra las suyas sobre `core::api`. **Los módulos no saben que existe la IA.**
- **Lo que devuelve una herramienta obedece las reglas de siempre:** solo agregados, atributos personales solo de grupos de 3 o más (ADR-027, ADR-029), y nunca contacto, nombres, RFC, cuentas, sueldos ni cuánto paga cada persona. Las reglas de la ficha (ADR-023) se vuelven pruebas de cada herramienta. La herramienta es la frontera, no el prompt.

### 3. El conocimiento: tres fuentes, cada una con su manera de buscar

| Fuente | Qué es | Cómo se busca | ¿Datos personales? |
|---|---|---|---|
| **Manual del ERP** | Qué es cada pantalla y cada campo, para qué sirve, qué módulos lo usan, cómo se hace cada cosa. Vive en el repositorio, versionado y escrito según `08-estilo-redaccion.md` | FTS5 y vectores; se indexa al instalar o al actualizar | No |
| **Expediente de la institución** | El texto limpio de sus documentos (ADR-033) | FTS5 y vectores guardados en `sqlite-vec`, con citas verificadas | No (pasaron por el escáner); amarillo: solo los fragmentos necesarios |
| **Datos de los módulos** | Personas, espacios, dinero, indicadores | **No se buscan por significado:** se consultan con herramientas que devuelven agregados calculados por el código | Nunca llegan en crudo |

- **Determinista primero:** un dato estructurado nunca se adivina con búsqueda por significado; se pide a su dueño.
- **Búsqueda híbrida en Rust:** FTS5 y vectores combinados por posición, tope de fragmentos por pregunta y cita (documento y página) que el código verifica antes de mostrarla.
- **Los embeddings se guardan** (`document_chunk_vec`). Hoy se recalculan en cada lectura; ya estaba en el roadmap. Una tabla `vec_model` registra con qué modelo se calcularon, para volver a calcularlos si el modelo cambia.

### 4. El asistente siempre a la mano

- **Un panel de la app,** no de una página: se abre desde cualquier pantalla y sabe dónde está la persona (`screen`, `field` y el registro abierto, por su id y nunca su contenido).
- **Junto a cada campo, un «?»:** primero muestra **sin IA, al instante y sin costo** lo que el manual dice de ese campo (qué poner, para qué sirve, qué lo usa). «Preguntar más» abre el panel con la pregunta ya escrita. La mayoría de las dudas se resuelven sin gastar.
- **Lo que puede hacer por la persona:**
  - explicar;
  - llevarla a la pantalla correcta («Ir a…»);
  - llenar por ella: delega al Capturista, que deja propuestas en el formulario resaltadas como sugeridas, para aceptar o corregir una por una.
- **Sin IA** (sin internet, sin llave o sin presupuesto): el panel sigue buscando en el manual con FTS5 y muestra los artículos que encuentra. Toda la ayuda escrita funciona sin conexión; solo la conversación necesita la IA.
- **La conversación del asistente vive solo en memoria y se borra al cerrar la app** (decisión de la institución): son dudas de uso y lo que la IA ayuda a cambiar lo confirma la persona y queda en el historial de los datos. Lo que escribe la persona pasa por el escáner antes de ir a la IA, como todo texto.

### 5. Seguridad

- **Inyección por documentos:** el texto de un documento es dato, nunca instrucción. Las herramientas son de solo lectura y toda acción es una propuesta que acepta una persona, así que un documento malicioso no puede hacer nada.
- **Permisos:** cada encargo lleva la sesión de quien pregunta. Una herramienta no devuelve nada que esa persona no podría ver en su pantalla.
- **Bitácora:** el evento `ai.run` guarda agente, pasos, herramientas usadas (conteos) y resultado; nunca contenido.
- **Costo:** tope por encargo y el tope mensual de siempre. El Asistente usa el modelo ligero y el manual sin IA atiende las preguntas más comunes.

### 6. El principio 1 se amplía

«La IA solo hace: preguntas de diagnóstico, razonamiento sobre necesidades, redacción y resumen» pasa a ser:

> La IA hace: preguntas y conversación (diagnóstico y asistencia), razonamiento, redacción y resumen, y **propuestas** de datos con su fuente. Nunca calcula, nunca guarda y nunca decide sola: lo que propone lo acepta una persona y lo guarda el código.

## Orden de trabajo

Cada bloque termina con sus pruebas en seco (modelo simulado, sin gasto) y luego una corrida real acotada cuyas métricas se anotan en el roadmap.

| Bloque | Qué | Terminado cuando |
|---|---|---|
| **IA1** | Bucle de agentes en la base: `Agent`, `Tool`, protocolo JSON, topes, `ai.run` en la bitácora. Herramientas de solo lectura del núcleo (estado de llenado, secciones de la ficha). Tabla `ai_proposal` | Prueba en seco: un agente pide dos herramientas y responde; una herramienta no permitida se rechaza; el tope corta el bucle |
| **IA2** | Manual del ERP y catálogo de campos (sale del ADR-033) indexados; «?» junto a cada campo sin IA | Cada campo del catálogo tiene su entrada en el manual (prueba); la búsqueda funciona sin conexión |
| **IA3** | Panel del Asistente en toda la app, con pantalla y campo como contexto, delegación y «Ir a…» | Batería de preguntas simuladas por pantalla; corrida real con tope; costo por pregunta anotado |
| **IA4** | Capturista: propuestas en los formularios del catálogo, aceptar o rechazar una por una | Ninguna propuesta se guarda sin un clic (prueba); lo aceptado pasa por la validación y el escáner |
| **IA5** | Embeddings guardados y búsqueda híbrida en el expediente y el manual | Preguntas de referencia con su respuesta esperada; FTS5 solo contra FTS5 con vectores, medido |
| **IA6** | Lector de documentos del expediente: reglas primero, IA para lo que no tiene formato, citas verificadas | Con fixtures ficticios, el RFC sale de la constancia por reglas, sin IA |
| **IA7** | Investigador con herramientas de los módulos y revisión de cifras | Ninguna cifra de la respuesta falta en lo que devolvieron las herramientas (prueba) |
| **IA8** | Proyectista con herramientas además de la ficha | Medido contra hoy con el caso dorado: igual o mejor calidad con menos tokens; si no, se queda como está |

## Consecuencias

- **La ficha de la institución (`ai_sheet`) se queda** como resumen corto que recibe todo agente. El detalle se pide con herramientas. Se mide antes de acortarla.
- **El manual es parte del producto:** cambiar una pantalla incluye actualizar su entrada (una prueba lo exige para los campos del catálogo).
- **Más llamadas pequeñas en lugar de pocas grandes.** El costo por pregunta del Asistente se mide en IA3 y se anota en `07-ia-y-costos.md`.
- **`07-ia-y-costos.md` y los principios** se actualizan al aceptar este ADR.

## Alternativas descartadas

- **Un solo modelo con todo el contexto en cada llamada:** caro, crece con cada módulo y no lee documentos.
- **Agentes que conversan entre sí libremente o un orquestador que es otro modelo:** el costo y el comportamiento serían imposibles de acotar y de probar. El código elige y hay un solo nivel de delegación.
- **Herramientas que escriben directamente:** rompen el principio 3 y abren la puerta a que un documento malicioso cambie datos.
- **Meter los datos de los módulos a la búsqueda por significado:** mezclaría datos personales con texto recuperable y haría adivinar lo que el código ya sabe calcular.
- **Solo textos de ayuda fijos (tooltips):** no responden la duda con las palabras de la persona ni conocen su institución. Se quedan como la primera respuesta sin costo, no como la única.
