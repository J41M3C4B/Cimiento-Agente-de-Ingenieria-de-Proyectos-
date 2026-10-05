# ADR-009 · `gemini-3.5-flash` como modelo fuerte principal

**Estado:** Aceptada (Fase 2). Reemplaza los modelos por defecto de los ADR-007 y ADR-008; el resto de esas decisiones sigue igual.

## Contexto
En las corridas reales del caso dorado, `gemini-3.8-flash`, `gemini-3.7-flash` y `gemini-3.6-flash` devolvieron `503 UNAVAILABLE` («alta demanda») en todos los intentos, incluida la cadena de respaldo completa (ADR-008). En la sonda siguiente (`gemini_probe_live`), `gemini-3.5-flash` respondió 200 con la petición fuerte exacta del resumen. Es **una sola muestra** y a otra hora: no prueba que el 3.5 esté menos saturado, solo que respondió cuando los otros tres no.

## Decisión
- Nivel fuerte por defecto: `gemini-3.5-flash` primero y, como respaldo en este orden, `gemini-3.8-flash` → `gemini-3.7-flash` → `gemini-3.6-flash`. El nivel ligero sigue en `gemini-3.5-flash-lite` sin respaldo.
- Cuesta el doble que los nuevos ($1.50 / $9.00 contra $0.75 / $3.75 de precio de introducción). Para el volumen de pruebas es poco (ver abajo); cuando los modelos nuevos se estabilicen se puede volver a ponerlos al frente cambiando solo la configuración.
- Los modelos nuevos quedan como respaldo, no se descartan: cada uno tiene su propio cupo diario.

## Confirmado con el servicio real
- `generateContent` acepta `responseMimeType` + `responseJsonSchema` con `anyOf` + `null` y `thinkingConfig.thinkingLevel = "medium"`, en `gemini-3.5-flash` y `gemini-3.5-flash-lite` (200 y respuesta válida con el esquema completo del resumen). Esto confirma los puntos 1 y 2 de «lo no verificado» del ADR-007 **para esos dos modelos**.
- Los identificadores `gemini-3.5-flash-lite` y `gemini-3.5-flash` existen.
- `usageMetadata` trae `thoughtsTokenCount` y el cálculo de uso funciona: el razonamiento llega a ser el 70 % de lo que se cobra como salida.
- El nivel ligero: 9 a 11 llamadas por caso dorado, 0.77 s en promedio, unos 750 tokens de entrada y 25 de salida cada una.
- Flash-Lite también devuelve algún 503 suelto (1 de 11); el reintento único lo resolvió.

## Todavía sin confirmar
- Los modelos 3.6, 3.7 y 3.8 nunca contestaron 200: sus identificadores y que acepten esos mismos parámetros siguen sin verificarse.
- Que el 503 cuente contra el cupo diario (se asume que sí).
- El resto de la lista del ADR-007: columnas de los límites, hora de reinicio del día, cómo marca el servicio el 429 «por día».

## Mediciones reales (una muestra por modelo, petición fuerte del resumen)
| Modelo | Tiempo | Salida (de ella, razonamiento) | Costo a precio de pago |
|---|---|---|---|
| `gemini-3.5-flash`, razonamiento medio | 10.5 s | 2,259 (1,603) | ≈ $0.022 USD ≈ $0.41 MXN |
| `gemini-3.5-flash-lite`, razonamiento medio | 9.9 s | 2,371 (1,739) | ≈ $0.0063 USD ≈ $0.12 MXN |
| `gemini-3.5-flash-lite`, sin razonamiento | 2.9 s | 671 (0) | ≈ $0.0020 USD ≈ $0.04 MXN |

(Entrada de 1,179 tokens en los tres.) Los tres devolvieron un resumen válido que replantea la necesidad, usa 14, 6 y 3 y marca preguntas abiertas; la calidad frente a los criterios del caso dorado se juzga con `golden_case_live`, no con estas sondas.

## Consecuencias
- Un resumen cuesta unas 11 veces más con el 3.5 Flash que con Flash-Lite sin razonamiento (≈ $0.41 contra ≈ $0.04 MXN). Si la calidad de Flash-Lite alcanza para el resumen, es una palanca de costo y de tiempo (3 s contra 10 s) que no se ha decidido.
- El razonamiento medio domina el tiempo del resumen (unos 10 s). Bajarlo a `low` (`effort_strong`) es la otra palanca.
