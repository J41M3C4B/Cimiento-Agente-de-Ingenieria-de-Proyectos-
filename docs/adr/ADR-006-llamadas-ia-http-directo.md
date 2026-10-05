# ADR-006 · Llamadas a la IA por HTTP directo, con salidas estructuradas

**Estado:** Aceptada (Fase 2). Falta confirmarla con una llamada real (caso dorado).

## Contexto
Anthropic no tiene SDK oficial para Rust. El proveedor debe ser intercambiable (ADR-005), la llave nunca sale de Rust, y la respuesta debe ser JSON que el código valida.

## Decisión
- `AnthropicProvider` llama a `POST /v1/messages` con `reqwest` (TLS nativo de Windows, `native-tls`; la TLS por defecto de `reqwest` 0.13 exige herramientas extra para compilar).
- **Salidas estructuradas** (`output_config.format = json_schema`) y además validación propia con `jsonschema` antes de aceptar. Si no valida, un reintento con el error; si falla otra vez, mensaje amable y captura manual.
- **Prompt del sistema fijo** con `cache_control` para aprovechar el caché; el contexto variable va después.
- Modelos por nivel en configuración (`app_settings`, clave `ai.settings`): `Light = claude-haiku-4-5`, `Strong = claude-sonnet-5-5`. Los precios y el tipo de cambio también son configurables.
- No se envían parámetros de muestreo (`temperature`, etc.) ni `tool_choice` forzado (rechazados por los modelos nuevos). `effort` solo en el nivel fuerte y no en Haiku (lo rechaza).
- En modelos que lo admiten se pide `fallbacks: "default"` (reintento seguro si el servicio declina una solicitud).
- Cada llamada pasa por `ai::pipeline::run`: tope mensual → escáner sobre todo el prompt (se tapa y se registra `scanner.leak_prevented`) → proveedor → validación → registro en `ai_usage` con costo estimado en pesos.
- La llave vive solo en el llavero (`anthropic-api-key`); no hay comando que la devuelva a la pantalla.

## Consecuencias
- Sin SDK hay que seguir a mano los cambios de la API. Mitigación: la forma de la petición está probada con un servidor simulado (`wiremock`) y la llamada real se verifica con `cargo test golden_case_live -- --ignored`.
- El modelo no "calcula": puntajes, totales y etapas son código. Para que tampoco invente cifras en texto, el código revisa que todo número del resumen aparezca en lo que dijo la persona (`domain/figures.rs`); si no, se pide una corrección una vez y, si persiste, se avisa a la persona.
