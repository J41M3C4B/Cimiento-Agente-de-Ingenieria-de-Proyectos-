# 07 · IA y costos

## Reparto código / IA

| Código (determinista, costo cero) | IA (solo esto) |
|---|---|
| Leer PDF, Word, Excel | Formular la siguiente pregunta del diagnóstico |
| Detectar y tapar datos sensibles | Sintetizar el diagnóstico y replantear la necesidad |
| Cálculos: presupuesto, IVA, totales, porcentajes, puntajes | Proponer necesidades y alternativas |
| Checklist de requisitos tipificados | Extraer requisitos de una convocatoria nueva (una vez) |
| Llenar Excel y Word, validar formatos | Redactar secciones del proyecto |
| Control de etapas, bitácora, cifrado | Responder preguntas abiertas de cuestionarios |
| Mapeo de campos al perfil | Explicar en lenguaje sencillo por qué algo no cumple |

Prueba para decidir: *¿se puede escribir como regla?* → código.

## Proveedor intercambiable

```rust
pub enum ModelTier { Light, Strong }

pub struct AiRequest {
    pub task: AiTask,              // enum de tareas conocidas
    pub tier: ModelTier,
    pub system: String,            // prompt fijo versionado (cacheable)
    pub context: Vec<ContextBlock>,// fragmentos mínimos
    pub user: String,
    pub output_schema: serde_json::Value, // JSON Schema esperado
    pub max_output_tokens: u32,
}

#[async_trait]
pub trait AiProvider {
    async fn complete(&self, req: AiRequest) -> Result<AiResponse, AiError>;
    fn name(&self) -> &str;
}
```

Implementaciones: `AnthropicProvider` y `GeminiProvider` (APIs en la nube, ADR-006 y ADR-007) y, más adelante, `OllamaProvider` (local). Además de `complete`, el trait expone `model_name(tier)` (el pipeline marca el ritmo por modelo) y `check()` (valida llave y modelos sin generar). Los nombres de modelo **no se escriben en código**: van en configuración por nivel (`ai.model.light`, `ai.model.strong`) para actualizarlos sin recompilar.

## Asignación de nivel por tarea

| Tarea | Nivel | Salida máx. aprox. |
|---|---|---|
| `conversation.turn` (un turno de la conversación del diagnóstico, ADR-017) | Light | 500 tokens |
| `diagnosis.summary` | Strong | 1,200 |
| `prioritization.propose_needs` | Strong | 1,000 |
| `call.extract_requirements` (una vez por convocatoria) | Strong | 2,000 |
| `call.explain_gap` | Light | 300 |
| `drafting.section` (una sección del proyecto, ADR-018) | Strong | 1,500 por sección |
| `questionnaire.open_answer` | Light | 400 |
| `questionnaire.label_ambiguous` | Light | 200 |

Si la calidad de una tarea Light no alcanza en pruebas, se sube a Strong y se documenta.

## Agentes y herramientas (ADR-034)

- Un agente lo define el código (`ai::agent::Agent`): su oficio, su nivel, la lista cerrada de herramientas, el tope de pedidos de herramientas, el tope de texto por encargo y el esquema de su respuesta. Su tarea en `ai_usage` es `agent.<nombre>`.
- **Protocolo en JSON sobre el recorrido de siempre** (`prompts/agent_protocol.v1.md`): en cada paso el modelo responde `{"tool", "args", "answer"}`. Cada paso pasa por presupuesto, escáner, ritmo, validación del esquema y `ai_usage`, igual con Gemini, Anthropic u Ollama.
- Rust revisa que la herramienta sea de la lista del agente y que los argumentos tengan su forma, la corre y le devuelve el resultado en el paso siguiente. Una herramienta fuera de la lista nunca corre; el modelo se entera y sigue.
- Al llegar al tope de pedidos se le pide responder con lo que tiene; si vuelve a pedir, el encargo termina (`step_limit`). Al pasar el tope de texto, también (`token_limit`).
- Las herramientas solo leen y revisan el permiso de quien pregunta. Las del núcleo hoy: `fill_state` (qué tan completos están los datos y qué falta, sin valores) y `sheet_section` (una parte de la ficha de la institución: `institution`, `money`, `people`, `staff` o `facilities`, con las mismas reglas de la ficha completa), `manual_search` y `manual_entry` (el manual del programa, nunca datos de la institución) y `documents_search` (pasajes de los documentos de la institución, con su documento y página; IA5).
- El prompt del agente (oficio, protocolo y herramientas) es el mismo en todos los pasos, así que el proveedor lo puede guardar en caché.
- Lo que la IA quiera cambiar es una propuesta (`ai_proposal`) que acepta una persona.
- El manual del programa (`docs/manual/`) responde el «?» de cada campo y la búsqueda de Ayuda **sin IA**: no gasta nada.
- **El Asistente (IA3, `core/agents/assistant.rs`):** modelo ligero, hasta 4 pedidos de herramientas y 40,000 tokens por pregunta. Herramientas: `manual_entry`, `manual_search`, `fill_state` y `sheet_section`; no recibe la ficha completa, pide la parte que toca. El código le dice la pantalla, el campo, su entrada del manual y la ventana donde se llena; revisa que «Ir a…» sea un lugar conocido y que cada fuente sea una entrada del manual. Sin IA, responde el manual. **Costo medido** (2026-10-10, Gemini `gemini-3.5-flash-lite`, 3 preguntas): de 1 a 2 llamadas, 1,850 a 3,780 tokens de entrada y unos 110 de salida; **de $0.015 a $0.027 MXN por pregunta**, de 2 a 3 segundos.
- **La búsqueda en los documentos (IA5, `core/archive/search.rs`):** por palabras (FTS5, sin costo) y por significado con los vectores de `gemini-embedding-2` (0.20 USD por millón de tokens de entrada; tabla de precios). Los vectores del expediente se calculan una vez por página, en segundo plano, y cada pregunta pide uno: cada solicitud queda en `ai_usage` como `embed.records`, con tokens estimados, y cuenta para el tope. Un expediente de 200 páginas cuesta menos de un peso. Desde C3 el Asistente ya no busca: le pregunta al Lector del Expediente (abajo). **Costo medido** (con la herramienta `documents_search` de entonces) (2026-10-12, una pregunta sobre el expediente ficticio): 2 llamadas del Asistente y 1 de vectores, **$0.034 MXN**, 2.4 s.
- **El Capturista (IA4, `core/agents/capturist.rs`):** modelo ligero, una sola llamada y sin herramientas. El código le da la ventana, los campos que la IA puede ver con lo guardado y la conversación; devuelve sugerencias (`ai_proposal`) con las palabras de la persona de donde salieron, y el código descarta lo que no cabe, lo protegido y lo que la persona no dijo. **Costo medido** (2026-10-10, Gemini `gemini-3.5-flash-lite`, un párrafo para «Su institución»): 1 llamada, 1,580 tokens de entrada y 295 de salida; **$0.0224 MXN**, menos de 2 segundos.
  - **Batería de C4** (2026-10-15, `fixtures/capturista.json`, 12 casos en las cuatro ventanas, dos rondas): oficio v2, sin nota (las palabras para la persona las pone el programa). 23 de 24 casos, 136 de 138 campos, 0 inventados; 24 llamadas, 38,012 tokens de entrada y 4,104 de salida, **$0.40 MXN** (unos $0.017 por caso), de 1.6 a 6 s. El código tiró la suma de camas que el modelo propuso en las dos rondas.
  - **Oficio v3** (2026-10-15, 14 casos, dos rondas): la persona no usa las palabras de los campos («viven», «abuelitos»). **28 de 28 casos, 154 de 154 campos, 0 inventados**; 33 llamadas (3 respuestas vacías que el bucle volvió a pedir), 54,277 tokens de entrada y 4,772 de salida, **$0.52 MXN**. El código tiró las sumas (20 + 15, y 5 libres + 25 que viven). Darle el «Qué poner» del manual con cada campo se midió y empeoró (24 de 28): no se usa.
  - **Oficio v4, ADR-041** (2026-10-15, 15 casos, dos rondas): copia los datos propios de la institución (domicilio, teléfono, RFC) letra por letra; caso nuevo con el domicilio completo. **29 de 30 casos, 187 de 188 campos, 0 inventados**; 30 llamadas, 52,782 tokens de entrada y 5,702 de salida, **$0.56 MXN** (unos $0.019 por caso), de 0.8 a 1.9 s. El código tiró 7 propuestas (las sumas, una edad supuesta de «personas mayores» y «voluntarios» sin cifra). El fallo: una ronda no propuso el año dicho con palabras («desde dos mil diez»).

- **El Lector del Expediente con IA (C2, `core/reader/fill.rs`):** «Llenar con mis documentos» por ventana y «Llenar todo» (4 ventanas). Modelo ligero, una llamada por ventana y sin herramientas. El código busca por campo los 3 pasajes del Expediente (palabras y vectores), le da a la IA los campos vacíos que puede ver, con cada opción y su nombre en la pantalla, y los pasajes numerados. Se queda solo con lo que cita un pasaje mostrado, con palabras que están en él, una cifra que está en esas palabras y un valor que cabe; sin documentos, sin campos vacíos o sin pasajes, no llama. **Costo medido** (2026-10-13, Gemini `gemini-3.5-flash-lite`, expediente ficticio de 13 documentos): 4 llamadas, 11,428 tokens de entrada y 1,660 de salida; **$0.14 MXN las 4 ventanas** (de $0.020 a $0.047 por ventana), de 1.6 a 3.6 s cada una; 18 de 18 campos bien, 0 inventados, 18 de 18 páginas correctas.
- **El Asistente como orquestador (C3, `core/agents/assistant.rs`, oficio v7) y el Lector que contesta (`core/reader/ask.rs`):**
  - El Asistente recibe el mapa del Expediente (documentos por su tipo y su estado, nunca qué dicen ni el nombre del archivo) y le pregunta al Lector con `record_reader`.
  - El Lector hace una llamada con el modelo ligero, sobre 4 pasajes de su cajón (o de un documento nombrado); el código revisa la cita y las cifras.
  - **Costo medido** (2026-10-13, Gemini `gemini-3.5-flash-lite`, batería de 13 preguntas, dos rondas):
    - 26 de 26 bien, 16 de 16 citas con la página esperada, 0 inventados;
    - por pregunta, 1 o 2 llamadas del Asistente y, si toca, 1 del Lector: de 1,600 a 6,800 tokens de entrada y unos 300 de salida, **unos $0.04 MXN**, de 1 a 7 s;
    - las 26 preguntas: 50 llamadas del Asistente ($0.75), 21 del Lector ($0.26) y 21 solicitudes de vectores ($0.02), **$1.03 MXN**.
  - Algunas preguntas tardaron unos 45 s: el programa espera para no pasar de 15 llamadas por minuto del plan gratuito; con una persona preguntando, no pasa.
  - **Oficio v8 y el mapa de lugares** (2026-10-14, ADR-039; batería de 14 preguntas: se agregó «ese mismo» después de una respuesta del registro, y el respaldo se pregunta desde «Mi institución»):
    - primera ronda: 13 de 14; el seguimiento contestó «adultas mayores» de memoria, sin Lector ni cita. El código ahora dice en el contexto de qué documento salió la respuesta anterior y su clave;
    - tercera y cuarta ronda (v8 final, dos rondas): **25 de 28, 0 inventados**, 16 de 18 citas con su página. El respaldo va a «Seguridad» 2 de 2; el seguimiento, 1 de 2 (la otra ofrece «Llenar con mis documentos» en lugar de preguntarle al Lector); «¿Qué documentos tengo?», 1 de 2 (da las cifras sin nombrarlos); el régimen fiscal, 1 de 2 (sin cita);
    - una vez contó mal los documentos («dieciséis» con 13 en el mapa): ahora el mapa trae los documentos por estado con sus cifras, hechas por el código;
    - las 28 preguntas: 50 llamadas del Asistente ($0.87), 20 del Lector ($0.26), **$1.14 MXN**; la lista de lugares del oficio sale del mapa, unos 300 tokens más por llamada.
  - **Oficio v9, C4 fase 2** (2026-10-15; batería de 16 preguntas: se agregaron un dato contado después de hablar del poder y «30 espacios y viven 12 niñas»):
    - línea base con el v8 (dos rondas): 24 de 32; los dos casos nuevos 0 de 2;
    - v9 final (dos rondas): **32 de 32, 18 de 18 citas, 0 inventados**;
    - las 32 preguntas: 51 llamadas del Asistente ($0.94), 19 del Lector ($0.25) y 20 solicitudes de vectores ($0.02), **$1.21 MXN**; unos 600 tokens más por llamada que el v8 (el mapa dice qué lleva cada ventana).
  - **Funciones nativas de Gemini contra nuestro protocolo JSON** (misma batería, dos rondas, mismo ritmo): JSON, 26 de 26 y $0.72 MXN del Asistente con 89,397 tokens de entrada; nativas, 25 de 26 y $1.10 MXN con 175,000, porque cada paso reenvía la conversación y las declaraciones. Mismo tiempo. Se queda el protocolo JSON (ADR-036, precisiones C3).

## Técnicas de ahorro

1. **Contexto mínimo:** se recuperan solo los fragmentos relevantes (FTS5 + vectores). Nunca documentos completos.
2. **Prompt fijo cacheable:** instrucciones del sistema al inicio y sin cambios entre llamadas para aprovechar el caché del proveedor.
3. **Salidas cortas en JSON** con esquema y `max_output_tokens` ajustado.
4. **Procesar una vez:** cada convocatoria se convierte en `call_template` y se reutiliza.
5. **Sin IA cuando el dato existe:** mapeo de campos al perfil.
6. **Repreguntas acotadas:** máximo 2 por dimensión del diagnóstico.
7. **Reintento único:** si el JSON no valida, un reintento con el error; si vuelve a fallar, se muestra un mensaje amable y se permite captura manual.

## Control de gasto

- Cada llamada registra tokens en `ai_usage` y un costo estimado en pesos con una tabla de precios **configurable** (los precios cambian; actualizar al configurar).
- Pantalla sencilla: "Este mes van $X de $Y".
- Tope mensual configurable. Al llegar al 80 % se avisa; al 100 % se pausan las funciones de IA y todo lo demás sigue funcionando.
- Meta de diseño: un proyecto completo en el orden de **pocos pesos a algunas decenas**. Se verifica con el caso dorado en Fase 2 y Fase 4.

## Prompts

- Viven como archivos en `src-tauri/src/ai/prompts/` con versión (`conversation_turn.v1.md`).
- Cada prompt incluye: rol, tono (ver `08-estilo-redaccion.md`), reglas de no inventar datos, formato de salida JSON.
- Instrucción común a todos: *"Si no tienes un dato, no lo inventes: agrégalo a `open_questions`. Las cifras van como marcadores `{{nombre}}`, no como números escritos por ti."*
- Cambiar un prompt = nueva versión + correr el caso dorado.

## Privacidad en la llamada

- Antes de enviar: segunda pasada del escáner sobre el prompt completo.
- Nunca se envían: datos de contacto institucionales, nombre de la representante legal, nada de nivel rojo.
- Revisar y documentar los términos de retención del proveedor antes de usar datos reales (ver `03-gobernanza-datos.md`).

## Sin IA no hay conversación (ADR-017)

Cimiento funciona **solo con IA**: no hay dos vías (con y sin). Si no hay internet, llave de API o presupuesto, lo que la persona escribió queda guardado y la pantalla ofrece «Intentar otra vez»; nada se inventa en su lugar. Lo que no es conversación (perfil, checklist, presupuesto, exportación) sigue siendo código y no depende de la IA.

## Estado de la implementación (Fase 2)

- Ver ADR-006. Código en `src-tauri/src/ai/` (`anthropic.rs`, `pipeline.rs`, `settings.rs`, `prompts.rs`) y `diagnosis_service.rs`.
- Tareas implementadas: `conversation.turn` (Light; antes `diagnosis.next_question`), `diagnosis.summary` (Strong), `prioritization.propose_needs` (Strong) y `call.canonical` (lectura de convocatorias). Las demás llegan con sus fases.
- Salida máxima del resumen: 2,000 tokens (más que los 1,200 previstos: un JSON cortado no sirve y el costo sigue al uso real). El nivel fuerte suma un margen configurable (4,000) para el razonamiento del modelo.
- Modelos y precios por defecto en `AiSettings` (editable): Light `claude-haiku-4-5` ($1/$5 por millón de tokens), Strong `claude-sonnet-5-5` ($2/$10). Tipo de cambio inicial 18.5 MXN/USD y tope mensual inicial $200: **actualizar al configurar**.
- **Proveedor activo: Gemini** (ADR-007, escenario B). Light `gemini-3.5-flash-lite` ($0.30/$2.50), Strong `gemini-3.5-flash` ($1.50/$9.00; ADR-009). Los `3.6`/`3.7`/`3.8-flash` ($0.75/$3.75 hasta el 31-dic-2026; $1.50/$7.50 desde el 1-ene-2027, ya cargado como precio con fecha) quedan de respaldo. `gemini-3.1-flash-lite` ($0.25/$1.50, precio de terceros por confirmar en la página de Google; mismos límites que 3.5 Flash-Lite hasta leerlos en AI Studio) se puede elegir a mano como ligero cuando 3.5 Flash-Lite está saturado; no es respaldo de la cadena ni tiene temperatura fija hasta medirlo. Se cambia en «Ayuda automática»; cada proveedor usa su propia llave.
- **Cadena de respaldo (ADR-008):** si el modelo fuerte está saturado, sin cupo o no existe, pasa al siguiente (`gemini-3.5-flash` → `3.8` → `3.7` → `3.6`) sin esperar. El ligero no tiene respaldo.
- **Mediciones reales** del resumen (ADR-009): `3.5-flash` con razonamiento medio ≈ 10.5 s y ≈ $0.41 MXN; Flash-Lite sin razonamiento ≈ 2.9 s y ≈ $0.04 MXN.
- **Límites y ritmo:** el pipeline respeta los límites por modelo (`rate_limits`): no llama al llegar al límite del día y espera el del minuto. Ver ADR-007.
- **Métricas:** cada intento se registra con tiempo, tokens de razonamiento y motivo de falla; la pantalla «Ayuda automática» y `ai::metrics::format_report` (que imprimen las pruebas en vivo) muestran uso, límites, tiempos y costo equivalente a precio de pago.
- **Pruebas sin gastar llamadas:** `cargo test dry_run` corre el caso dorado completo contra un servicio Gemini simulado por HTTP real. `cargo test gemini_smoke_live -- --ignored --nocapture` hace la primera llamada real (una sola) y la comprobación gratuita de llave y modelos.
- Para que la IA no invente cifras, la regla "las cifras van como marcadores" se aplica en la Fase 4 (redacción). En el diagnóstico se usa la revisión por código de `domain/figures.rs`.
- El costo real del caso dorado se registra en `docs/09-roadmap.md` al correr `golden_case_live`.
