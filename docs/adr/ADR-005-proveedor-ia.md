# ADR-005 · Proveedor de IA intercambiable con niveles

**Estado:** Aceptada

## Contexto
Hay que equilibrar calidad, costo y confidencialidad, y los modelos cambian seguido.

## Decisión
- Trait `AiProvider` con implementaciones para API en la nube (Anthropic) y local (Ollama).
- Dos niveles, `Light` y `Strong`, asignados por tarea.
- Nombres de modelo y precios en configuración, no en código.
- Llamadas solo desde Rust; llave en el llavero.

## Consecuencias
- Se puede mover una tarea a un modelo local sin tocar el dominio.
- Hay que mantener actualizada la tabla de precios para que el contador de gasto sea útil.
