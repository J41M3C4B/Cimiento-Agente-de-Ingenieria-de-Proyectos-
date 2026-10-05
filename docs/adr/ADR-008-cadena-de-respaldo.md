# ADR-008 · Cadena de modelos de respaldo

**Estado:** Aceptada (Fase 2). Falta verla funcionar con el servicio real (los 503 son intermitentes).

## Contexto
En la primera corrida real del caso dorado el modelo fuerte (`gemini-3.8-flash`) contestó `503 UNAVAILABLE: "This model is currently experiencing high demand"` en los dos intentos del resumen (confirmado después con una llamada de diagnóstico: 8.3 s hasta el error). El nivel fuerte dependía de un solo modelo, justo el más nuevo y el que más se satura, y el reintento a los 1.5 s no sirve ante una saturación. El problema no es solo de pruebas: en uso real el resumen caería al borrador manual cada vez que el servicio se sature.

## Decisión
- Cada nivel tiene una **cadena**: el modelo principal y, después, sus respaldos (`AiSettings::chain_of`). Por defecto, en Gemini: fuerte = `gemini-3.8-flash` → `gemini-3.7-flash` → `gemini-3.6-flash` (mismo precio, $0.75 / $3.75, y cada uno con su propio cupo diario); ligero = solo `gemini-3.5-flash-lite` (el siguiente Flash cuesta cinco veces más). Anthropic no tiene respaldos propios: ya usa el `fallbacks` del servidor (ADR-006). La cadena es configuración (`gemini_light_fallbacks`, `gemini_strong_fallbacks`).
- El pipeline recorre la cadena (`call_provider`). **Pasa al siguiente modelo de inmediato, sin esperar ni reintentar el que falló**, cuando el fallo es del modelo y no de la petición: saturación o error 5xx, 429, cupo del día agotado (el que avisa el servicio o el que cuenta el pipeline localmente, sin gastar la llamada) o modelo inexistente (404).
- **No** pasa al siguiente cuando el fallo se repetiría igual: sin internet (se reintenta una vez), llave rechazada, solicitud rechazada, respuesta cortada o con formato inválido.
- Cuando no hay a quién pasar (cadena de uno, o ya se llegó al último), se conserva el comportamiento anterior: un reintento a los 1.5 s en fallos temporales y después error final. La app sigue sin ayuda automática como antes.
- Cada intento queda registrado con el modelo que lo recibió: el fallo bajo el modelo que falló y la respuesta, con su costo, bajo el que contestó. El reporte y la pantalla marcan los modelos de respaldo («de respaldo»).
- El trait `AiProvider` suma `model_chain(tier)` y `complete_with_model(req, model)`; ambos tienen implementación por defecto, así que un proveedor sin respaldos no cambia.

## Consecuencias
- Un resumen puede haberlo escrito un modelo distinto al principal. La calidad puede variar entre versiones; para comparar, las métricas dicen qué modelo contestó cada llamada.
- Los respaldos suman cupo: con la cadena por defecto, hasta 60 llamadas fuertes al día en el plan gratuito en lugar de 20.
- Cada intento fallido a un 5xx cuenta para el límite local del modelo (se asume que cuenta también para el del servicio; sin verificar).
- Los identificadores `gemini-3.7-flash` y `gemini-3.6-flash` están sin verificar con el servicio real; la comprobación gratuita de la app («Probar la conexión») hoy valida solo los modelos principales.
- Si todos los modelos de la cadena fallan, el resumen cae al borrador armado con las respuestas, como antes.

## Ajustes tras la corrida real de NMP 2024 (cinco 503 seguidos)
- **Los fallos 5xx no cuentan para el cupo local.** El servicio no los cobra a la llave; seguían apareciendo en el reporte, pero ya no llenan «x/5 por minuto» ni «x/20 por día».
- **Flash-Lite como último recurso, solo para la revisión de la convocatoria** (`AiTask::light_model_may_step_in`). Es la única tarea cuyo resultado se verifica con citas contra el PDF y que nunca bloquea la subida. Las preguntas y el resumen del diagnóstico no bajan de modelo.
- **Gemma 4 31B (`gemma-4-31b-it`) descartado tras probarlo:** rechaza `thinkingLevel` (400), con JSON forzado no respondió en 120 s, y sin él tardó 88 s y devolvió su razonamiento mezclado con el texto, no JSON. Límites de su cuenta: 30/min, 16 mil tokens/min, 14.4 mil/día. Se puede reevaluar con `PROBE_MODEL=<id> cargo test gemini_probe_live -- --ignored --nocapture`.

- **Respuesta cortada (`truncated`):** el modelo llenó todo su espacio de salida sin terminar (en la práctica, un bucle de razonamiento). La cadena pasa al siguiente modelo, igual que con un 503 (ver ADR-013).
