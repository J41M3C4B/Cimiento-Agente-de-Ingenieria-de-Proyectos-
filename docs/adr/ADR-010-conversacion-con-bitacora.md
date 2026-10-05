# ADR-010 · Conversación guiada con bitácora estructurada, convocatoria primero

**Estado:** Propuesta. No se implementa hasta aprobarla. Si se aprueba, reemplaza la parte del diagnóstico y el orden de etapas de `02-flujo-funcional.md` y los ADR-003 y ADR-005 siguen igual.

## Contexto
La corrida real del caso dorado (ADR-009) mostró que la IA pregunta sin memoria del proyecto: la pregunta de seguimiento solo ve su dimensión, las repreguntas se solapan con dimensiones posteriores y el resumen omite alternativas que el caso pedía. Detalle y medidas en `docs/10-metodologia-conversacion.md`.

## Decisión (propuesta)
- Los proyectos arrancan por la **convocatoria**: se elige y se confirman sus requisitos antes del diagnóstico; sus requisitos entran como restricciones.
- Cada turno de la IA es **un solo JSON** con el mensaje para la persona y un `record` etiquetado (respondió, punto clave, objetivo, punto a revisar, decisiones abiertas), sin llamada de extracción aparte. El prompt del sistema fija esa estructura de forma permanente.
- El **código** lleva una **bitácora estructurada** (no un RAG vectorial) y le pasa a la IA un resumen de ella en cada llamada. El código decide la fase y el hueco; la IA solo redacta la pregunta de ese hueco y etiqueta lo dicho.
- Lo que la IA registra necesita **cita textual verificable** de lo que dijo la persona; sin ella baja a hipótesis. La persona confirma «lo que entendí».
- **La conversación termina por construcción, no por buena voluntad:** la IA lleva la iniciativa de cara a la persona (hipótesis para confirmar, opciones, avance), y el código impone un tope global de vueltas, máximo 2 intentos por hueco y una táctica más fácil tras 2 vueltas sin progreso; un hueco que no cierra pasa a pendiente y nunca bloquea. Se prueba con una batería de personas simuladas y se cierra con una prueba piloto real, porque el caso dorado con respuestas escritas es solo el camino feliz.
- El método es de lo general a lo particular, por fases con criterio de salida evaluado por código (marco, situación, raíz, objetivo, alcance, viabilidad). Los 5 porqués son una técnica opcional dentro de la fase de raíz.
- La bitácora se exporta como JSON de memoria de la institución: solo lo confirmado, revisado por el escáner, y al reutilizarse entra como propuesta a reconfirmar.
- El RAG vectorial se reserva para texto grande (convocatorias y documentos pasados).
- **Tras la simulación con una convocatoria real** (`docs/11-simulacion-nmp-2026.md`), se agrega: solidez por hecho (observado, referido, supuesto, verificado); plan de pruebas generado desde la bitácora; formulario progresivo con estado por pregunta; perfil de la institución ampliado como fuente de verdad (lo que contiene se confirma, no se pregunta); y **extracción determinista de la convocatoria a JSON** (la IA solo para lo ambiguo, con confirmación humana). Detalle en `10-metodologia-conversacion.md` §13.

## Consecuencias
- Cambia el orden de etapas (`domain/stage.rs`, interfaz y documentos), el modelo de datos (tablas de bitácora y de turnos) y los prompts del diagnóstico, y obliga a adelantar la lectura de convocatorias de la Fase 3.
- El costo por vuelta sube unos $0.01 MXN (estimado, a medir) y se espera que baje el número de vueltas; el resumen final se arma desde la bitácora.
- El riesgo principal es que la interpretación de la IA deforme lo dicho; se acota con la cita verificable y la confirmación.
- Se valida primero con la simulación gratuita y después con una corrida real del caso dorado.
