# 12 · Validación con instituciones

**Fecha:** 2026-10-05. **Fuente:** retroalimentación reportada por el autor tras la demostración y el uso de la herramienta por dos instituciones de asistencia privada. Es evidencia **cualitativa** (dos instituciones, sin medición de tiempos): sirve para orientar el producto, no para afirmar un rendimiento general.

## Condiciones de la prueba

- Las instituciones usaron la aplicación con información real suya.
- La IA corrió con una **API de pago** contratada con la cuenta de la dirección de una de las instituciones, justamente para quedar bajo los términos del proveedor que establecen que el contenido no se usa para entrenar sus modelos. Durante el desarrollo, con datos ficticios, se usó el nivel gratuito (ADR-007).
- **A la IA solo llegan agregaciones** (cuántas personas, totales, cifras ya sumadas por el código) y lo que la persona escribe en el chat, siempre después del escáner. Las fichas individuales de personal y beneficiarios se quedan en el equipo (ADR-020, ADR-023).
- Pendiente de la Fase 7, aún sin hacer: revisión legal de privacidad (menores y datos de salud), aviso de privacidad y consentimiento, y revisión formal de los términos del proveedor.

## Qué se probó

- El recorrido completo: lectura de la convocatoria, conversación de diagnóstico, objetivo, borrador y guía.
- **Convocatorias de la JAP y de la JAPEM**, que son las de uso cotidiano para estas instituciones: siete en total. No aparecieron problemas nuevos, salvo que **en 2 de 7 el título de la convocatoria salió inconsistente**. Es el límite que ya estaba documentado: el campo `nombre` no es estable cuando la portada es decorativa (ADR-015), y la persona lo revisa y lo corrige.

## Lo que dijeron

- Les sorprendió la **reducción de carga de trabajo** al armar proyectos. (Percepción de las personas; no se midieron tiempos antes y después.)
- Dijeron que el proceso es **muy sencillo** y no imaginaban que existiera una herramienta así: un «cerebro» que las ayuda a armar sus proyectos.
- La usaron también para **consultar las agregaciones de su propia institución** con otros fines, distintos de los donativos.
- Pidieron **agregar módulos para digitalizar sus procesos**, empezando por empleados y beneficiarios, y para conocer sus números con más detalle.

## La queja, y lo que revela

Una sola: las preguntas del chat les parecieron **algo cortantes**, y les costó entender **por qué el chat daba más peso a la causa de fondo que a la primera respuesta**. Al explicarles el método de los cinco porqués, quedó claro; pero la herramienta no lo explica sola.

Diagnóstico: cada pregunta de seguimiento llega sin recapitular. La persona no ve cómo lo que dijo antes lleva a la pregunta siguiente, y por eso el cambio de rumbo parece arbitrario aunque sea el centro del método.

## Cambios que se derivan

1. **Cada porqué recapitula.** Cada pregunta de seguimiento debe retomar la premisa anterior con un contexto muy breve y luego preguntar. Ejemplo: «Como la camioneta les está consumiendo gran parte del presupuesto, entonces… ¿por qué…?». Cambio en el prompt `conversation_turn` y en su prueba con personas simuladas.
2. **Hacer visible el progreso del razonamiento:** mostrar, en palabras sencillas, por qué la causa de fondo pesa más que la primera respuesta («lo primero que contaron es una consecuencia; seguimos hasta llegar a lo que la origina»).
3. **Módulos de digitalización:** ampliar el padrón de personal y beneficiarios (ADR-020) con más detalle y con tableros de cifras. Las reglas de privacidad no cambian: el detalle por persona se queda en el equipo y a la IA solo llegan totales.

## Lo que esta validación NO prueba

- No hay medición de tiempo ahorrado ni de calidad del proyecto final frente a uno hecho a mano.
- Son dos instituciones, ambas conocidas del autor, con una convocatoria cotidiana de la misma familia.
- Falta la prueba piloto sin acompañamiento: la explicación del método la dio el autor en persona, y esa es justamente la parte que la herramienta debe resolver sola.
