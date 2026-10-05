# ADR-012 — `gemini-3-flash-preview` como modelo fuerte principal

**Estado:** Aceptada. Confirmada con la revisión real de NMP 2024: respondió a la primera en 18.5 s, $0.2493 MXN.

## Contexto
ADR-009 puso `gemini-3.5-flash` como principal porque 3.6, 3.7 y 3.8 devolvían 503. En la revisión real de NMP 2024, los cuatro modelos fuertes devolvieron 503 y `gemini-3.5-flash` tardó 111 s solo en decirlo (207 s en total hasta que respondió Flash-Lite).

## Prueba
`gemini_probe_live` con `PROBE_MODEL=gemini-3-flash-preview`:
- Petición completa (razonamiento medio + esquema JSON): 200 en 7.9 s, JSON válido y contenido correcto con los datos del caso.
- Sin razonamiento: 200 en 26.6 s (razona más cuando no se le indica nivel).
- Sin esquema: 200 en 14.3 s, pero el formato se rompe (textos donde pedimos listas, nombres de campo distintos). El esquema forzado es necesario.
- Límites de su cuenta: 5/min, 250 mil tokens/min, 20/día. Precio: USD 0.50 entrada, 3.00 salida por millón de tokens (el más barato de los fuertes; 3.5 cuesta el triple).
- Descartado en la misma sesión: `gemma-4-31b-it` (ver ADR-008).

## Decisión
Cadena fuerte por defecto: `gemini-3-flash-preview` → `gemini-3.5-flash` → 3.8 → 3.7 → 3.6. La revisión de convocatoria puede terminar en Flash-Lite (ADR-008).

## Riesgos
- Es un modelo *preview*: Google puede cambiarlo, retirarlo o cambiar su precio al salir de preview. Un 404 pasa al siguiente modelo de la cadena, y el precio está en `ai/settings.rs` con su fecha de lectura (octubre 2026).
- Con solo 20 llamadas al día, el cupo diario del principal se agota pronto; la cadena pasa al siguiente modelo cuando eso ocurre.

## Resultado de la revisión real (NMP 2024)
- Primera llamada, sin 503. Frente a Flash-Lite (13 s, $0.2382) encontró además dos criterios reales (prioridad a Colima, Campeche, Yucatán… y articulación con otros actores; ambos en el PDF), quitó la fecha del DOF (es un documento, no una fecha del proceso) y agregó el inicio de la preselección (1 abril).
- El código descartó 3 citas que no aparecían en la página y marcó 4 paráfrasis (el modelo reescribe el objetivo y las categorías en vez de copiarlos). Pendiente: mostrar la cita literal cuando el texto es una paráfrasis sospechosa.

## Ajuste de la cadena (octubre 2026)
- `gemini-3.5-flash` sigue siendo el principal de la cadena fuerte, pero queda **apagado** mientras responda 503 (`AiSettings.disabled_models`): no se le pregunta y la cadena empieza en el siguiente. Encenderlo es quitarlo de esa lista.
- **Cadena fuerte:** 3.5-flash (apagado) → `gemini-3-flash-preview`. 3.6, 3.7 y 3.8 salen de la cadena (seguían devolviendo 503).
- **Cadena ligera:** `gemini-3.5-flash-lite`, sin respaldo.
- **Familia Gemini 2.x descartada:** se probó añadir `gemini-2.5-flash` y `gemini-2.5-flash-lite` (USD 0.30/2.50 y 0.10/0.40; 5 y 10 por minuto, 20 por día) y dejó de funcionar. Sus precios y límites ya no están en los valores por defecto.
- Se quitó del código todo lo propio de 2.x (`thinkingBudget`, su regla en el servicio simulado y su prueba). Dato para quien lo retome: 2.x no acepta `thinkingLevel`, pide `thinkingBudget` en tokens.
- Con un solo modelo fuerte encendido, un 503 se reintenta una vez en él (el último de la cadena siempre recibe un reintento) y la revisión de convocatoria puede terminar en Flash-Lite.
- Los ajustes ya guardados en una instalación anterior conservan su cadena; la nueva entra por defecto solo en instalaciones nuevas o al restablecer.

## 3.8 vuelve a probarse y se descarta (octubre 2026)
Con una llave de otra cuenta, `gemini-3.8-flash` devolvió 503 en todas las corridas (antes respondió una de dos), y `gemini-3.5-flash` siguió igual. La cadena fuerte queda como estaba: `gemini-3.5-flash` (apagado) y `gemini-3-flash-preview`; la ligera, `gemini-3.5-flash-lite`. Los ajustes guardados por la app antes de este cambio (sin `disabled_models`) traían la cadena vieja con 3.8, 3.7 y 3.6 y sin Preview: al cargarlos se reemplaza por la actual, conservando lo que la persona eligió (proveedor, tope mensual, precios).

### Pista pendiente sobre 3.8 (probe con `PROBE_MODEL=gemini-3.8-flash`)
La petición completa (con `thinkingLevel: medium` y esquema) devolvió 503 «This model is currently experiencing high demand» en 0.6 s; sin `thinkingLevel`, 200 en 7.1 s, y sin `thinkingLevel` ni esquema, 200 en 12.3 s. Una sola muestra por variante, así que no es concluyente, pero el 503 llegó casi al instante (los de antes tardaban decenas de segundos). Si se quiere retomar 3.8: probar el etiquetado con `effort_strong` vacío (el proveedor decide cuánto razonar). Decisión de la persona: dejarlo fuera y trabajar con Preview, que dio estabilidad.
