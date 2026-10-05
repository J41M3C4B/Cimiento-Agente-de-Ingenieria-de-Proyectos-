# ADR-003 · La IA propone, el código decide y escribe

**Estado:** Aceptada

## Contexto
Se necesita confiabilidad (cifras correctas, formatos intactos) y costo bajo.

## Decisión
- La IA solo hace tareas de lenguaje y razonamiento definidas en `07-ia-y-costos.md`.
- Toda salida de IA es JSON validado contra esquema.
- Cálculos, validaciones, lectura y escritura de archivos son código determinista.
- Todo dato lleva `origin`; lo que viene de la IA requiere confirmación humana antes de exportarse.

## Consecuencias
- Más código al inicio, menos errores y menor costo después.
- Las cifras dentro de textos redactados por la IA se insertan con marcadores.
