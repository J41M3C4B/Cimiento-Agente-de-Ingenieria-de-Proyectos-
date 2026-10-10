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
- Las herramientas solo leen y revisan el permiso de quien pregunta. Las del núcleo hoy: `fill_state` (qué tan completos están los datos y qué falta, sin valores) y `sheet_section` (una parte de la ficha de la institución: `institution`, `money`, `people`, `staff` o `facilities`, con las mismas reglas de la ficha completa).
- El prompt del agente (oficio, protocolo y herramientas) es el mismo en todos los pasos, así que el proveedor lo puede guardar en caché.
- Lo que la IA quiera cambiar es una propuesta (`ai_proposal`) que acepta una persona.
- El manual del programa (`docs/manual/`) responde el «?» de cada campo y la búsqueda de Ayuda **sin IA**: no gasta nada.

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
- **Proveedor activo: Gemini** (ADR-007, escenario B). Light `gemini-3.5-flash-lite` ($0.30/$2.50), Strong `gemini-3.5-flash` ($1.50/$9.00; ADR-009). Los `3.6`/`3.7`/`3.8-flash` ($0.75/$3.75 hasta el 31-dic-2026; $1.50/$7.50 desde el 1-ene-2027, ya cargado como precio con fecha) quedan de respaldo. Se cambia en «Ayuda automática»; cada proveedor usa su propia llave.
- **Cadena de respaldo (ADR-008):** si el modelo fuerte está saturado, sin cupo o no existe, pasa al siguiente (`gemini-3.5-flash` → `3.8` → `3.7` → `3.6`) sin esperar. El ligero no tiene respaldo.
- **Mediciones reales** del resumen (ADR-009): `3.5-flash` con razonamiento medio ≈ 10.5 s y ≈ $0.41 MXN; Flash-Lite sin razonamiento ≈ 2.9 s y ≈ $0.04 MXN.
- **Límites y ritmo:** el pipeline respeta los límites por modelo (`rate_limits`): no llama al llegar al límite del día y espera el del minuto. Ver ADR-007.
- **Métricas:** cada intento se registra con tiempo, tokens de razonamiento y motivo de falla; la pantalla «Ayuda automática» y `ai::metrics::format_report` (que imprimen las pruebas en vivo) muestran uso, límites, tiempos y costo equivalente a precio de pago.
- **Pruebas sin gastar llamadas:** `cargo test dry_run` corre el caso dorado completo contra un servicio Gemini simulado por HTTP real. `cargo test gemini_smoke_live -- --ignored --nocapture` hace la primera llamada real (una sola) y la comprobación gratuita de llave y modelos.
- Para que la IA no invente cifras, la regla "las cifras van como marcadores" se aplica en la Fase 4 (redacción). En el diagnóstico se usa la revisión por código de `domain/figures.rs`.
- El costo real del caso dorado se registra en `docs/09-roadmap.md` al correr `golden_case_live`.
