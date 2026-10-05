# ADR-007 · Gemini como proveedor principal por ahora (escenario B)

**Estado:** Aceptada (Fase 2). Falta confirmarla con una llamada real: `cargo test gemini_smoke_live -- --ignored --nocapture`.

## Contexto
Para no gastar mientras se prueba, se decidió usar los modelos de Google incluidos en la suscripción de quien desarrolla. Se comparó el costo por proyecto en tres escenarios con tamaños de llamada estimados (no medidos): todo Claude ≈ $22 MXN, todo Gemini ≈ $8 MXN (≈ $15 desde 2027), mixto ≈ $18 MXN. El costo no decide: decide la calidad de la redacción, que hay que medir con el caso dorado. Los datos de prueba son todos ficticios (`fixtures/`), así que la condición del plan gratuito de Google (el contenido puede usarse para mejorar sus productos) no aplica por ahora; **sí aplica a datos reales y entra en la revisión de términos de la Fase 7**.

## Decisión
- Un `GeminiProvider` junto al `AnthropicProvider` (ADR-005), elegido en `ai.settings.provider` (por defecto `gemini`). Cada proveedor tiene su llave en el llavero (`gemini-api-key`, `anthropic-api-key`) y sus modelos por nivel: Light = `gemini-3.5-flash-lite`, Strong = `gemini-3.8-flash`.
- **API `generateContent`** (`POST /v1beta/models/{modelo}:generateContent`, llave en el encabezado `x-goog-api-key`, nunca en la URL). Google la llama "legacy, pero totalmente soportada". Se prefirió a la API Interactions (beta) porque `generateContent` no guarda nada en el servidor y Interactions guarda cada llamada un día salvo que se pida `store=false`.
- Salida estructurada: `generationConfig.responseMimeType = application/json` + `responseJsonSchema`. Se quita `additionalProperties` del esquema enviado (no se sabe si Gemini lo admite) y no se pierde nada, porque `pipeline::validate` sigue validando contra el esquema completo.
- Razonamiento: en el nivel fuerte se manda `thinkingConfig.thinkingLevel` con el valor de `effort_strong` (vacío = el del modelo). El razonamiento se cobra como salida y cuenta contra `maxOutputTokens`, así que el nivel fuerte suma `thinking_headroom` (4,000) y el ligero `light_headroom` (200), para que una respuesta corta no se corte por pensar.
- Uso: `output_tokens` = respuesta + razonamiento; `input_tokens` = entrada sin lo que vino del caché (Gemini cuenta lo cacheado dentro de la entrada). El modelo que se registra es el pedido, no el `modelVersion` de la respuesta, para que la tabla de precios lo reconozca. Un nombre con sufijo se cobra como su familia (prefijo más largo).
- **Precios con fecha:** `gemini-3.6/3.7/3.8-flash` cuestan $0.75 / $3.75 hasta el 31-dic-2026 y $1.50 / $7.50 desde el 1-ene-2027 (`ModelPrice.then`). Sin esto el contador subestimaría el gasto en enero.
- **Cuidar las llamadas limitadas** (la pantalla de Google muestra límites por modelo; se leyeron como por minuto / tokens por minuto / por día):
  - `RateLimit` por modelo en `ai.settings` (Flash-Lite 15 / 250 mil / 500; los Flash 5 / 250 mil / 20).
  - Antes de cada intento el pipeline cuenta lo gastado: al llegar al límite del día **no llama** (`AiError::QuotaReached`, estado `quota_reached`, el flujo sigue sin ayuda automática); al llegar al del minuto **espera** a que salga la llamada más vieja de la ventana en lugar de provocar un 429. El día se cuenta como las últimas 24 horas, que solo puede ser más estricto que el de Google.
  - Un 429 del servicio que menciona "per day" se trata como final y no se reintenta.
  - Los intentos sin internet no cuentan contra ningún límite.
- **Métricas en tiempo real** (migración `0003_ai_metrics.sql`): cada intento, también los fallidos, deja una fila en `ai_usage` con `latency_ms`, `thought_tokens` y `error_kind`. `ai::metrics::usage_report` resume por modelo (llamadas, fallidas, uso del minuto y del día contra el límite, tiempo medio y p95, tokens, razonamiento, caché, costo) y por tarea. La pantalla «Ayuda automática» lo muestra y se actualiza cada 5 s; las pruebas en vivo imprimen el mismo reporte en texto.
- **Probar sin gastar:** `AiProvider::check` consulta `GET /v1beta/models/{modelo}` por cada modelo configurado: valida la llave y que el nombre exista sin generar nada. La pantalla lo expone como «Probar la conexión».

## Prueba en seco («como si yo fuera el modelo»)
`ai/dry_run_tests.rs` levanta un servicio Gemini falso por HTTP real que **rechaza con 400** cualquier petición que no siga la forma documentada y responde con una política de modelo escrita a mano. Con él corren de verdad el proveedor, el pipeline, el escáner, la base, el ritmo y las métricas: el caso dorado completo (8 llamadas ligeras y 2 fuertes), datos personales que no salen, cupo diario local y del servicio, 429 por minuto con reintento, llave rechazada, respuesta cortada o bloqueada, JSON inválido con corrección, y caché. Se comprobó con una mutación (agregar `temperature` a la petición) que 10 de esas pruebas fallan, así que la prueba no es vacía.

## Lo que NO está verificado (se confirma con `gemini_smoke_live`, una sola llamada)
La simulación solo conoce la API tal como está documentada; no prueba que la documentación se haya leído bien. Pendiente de confirmar con el servicio real:
1. Que `generateContent` acepte `responseJsonSchema` con `anyOf` + `null`, y `thinkingConfig.thinkingLevel` en estos modelos (la documentación actual muestra `thinking_level` solo para la API Interactions). Si lo rechaza, el primer error lo dirá; `effort_strong` vacío quita el campo.
2. Los identificadores `gemini-3.5-flash-lite` y `gemini-3.8-flash` (la comprobación gratuita los valida antes de generar).
3. Que las tres columnas de la tabla de límites sean por minuto / tokens por minuto / por día, y a qué hora se reinicia el día. Los límites son configuración y se corrigen sin recompilar.
4. Que la consulta de modelos no cuente contra el límite de generación (la documentación no lo dice).
5. Cómo detecta el servicio real un 429 «por día»: hoy se busca "perday" / "per day" en el cuerpo del error.
6. Si una respuesta cortada por `MAX_TOKENS` cobra tokens: hoy se registra con 0 (el costo real de esos intentos no aparece; subestima levemente).

## Actualización (2026-10-05)
Para el uso con las instituciones se contrató una **API de pago** con la cuenta de la dirección de una de ellas, para quedar bajo los términos del proveedor que establecen que el contenido no se usa para entrenar sus modelos. La condición del plan gratuito descrita arriba aplicó solo al desarrollo con datos ficticios. A la IA siguen llegando solo agregaciones (ADR-020, ADR-023). La revisión legal y de términos de la Fase 7 sigue pendiente.

## Consecuencias
- Los números de costo y tiempo de la simulación salen de tamaños estimados, no del servicio: no son métricas reales. Las reales salen de la primera corrida en vivo.
- El nivel gratuito tiene pocas llamadas fuertes al día (20): el caso dorado completo usa 2, pero cada reintento del resumen suma.
- Cambiar de proveedor es cambiar un ajuste; el escenario mixto (Light con Gemini, Strong con Claude) pide separar el proveedor por nivel y no se hizo.
