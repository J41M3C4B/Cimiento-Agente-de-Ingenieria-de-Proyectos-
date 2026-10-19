---
tipo: bloque
id: IA-1
carril: ia
estado: hecho
prioridad: alta
fase:
rama: main (entró con la puesta al día del 2026-10-18)
adr:
creado: 2026-10-18
cerrado: 2026-10-18
---

## Qué y por qué

Tres fallas que encontró el líder al probar el chat:
- tras el aviso del escáner, «Llenarlo por mí» repetía la frase de la persona;
- el chat se quedaba en «Llenando…» sin haber dejado nada en la ventana;
- «alcaldía Benito Juárez» se tomaba por el nombre de una persona.

## Terminado cuando

- [x] La frase aparece una sola vez y llega una sola vez a la IA.
- [x] «Llenando…» empieza desde el chat solo si quedaron sugerencias en esa ventana.
- [x] Después de «alcaldía», «delegación» o «demarcación», el escáner no ve una persona.

## Plan

Hecho antes del tablero (commits `d483dda` y `7d70513` en la rama). La regla del escáner es una regla por caso: la bóveda (IA-2) la vuelve innecesaria.

## Cómo probarlo

Las tres casillas nuevas de la sección 8 del [[15-guion-pruebas-nucleo|guion]]:
- [x] Con la alcaldía ya guardada, «Llenarlo por mí» dice «Ya estaba guardado igual» y no se queda en «Llenando…».
- [x] Si el escáner pregunta y contestas, tu frase no vuelve a aparecer.
- [x] «Estamos ubicados en la alcaldía Benito Juárez» ya no pregunta por datos personales.

## Resultado

Ya responde correctamente pero creo que para este comportamiento, donde un dato ya se tiene en un solo mensaje se puede manejar la conversacion menos cuadrada o robotizada y sonar algo mas natural, por ejemplo: Ese es un dato de la instititución [pero al mismo tiempo ya se reviso que ese dato ya esta cubierto, entonces la IA responde en ese mismo mensaje si gusta compartir algun otro dato y el boton llenarlo por mi solo aparece cuando de verdad aplica, otra decision que la IA debe hacer y no el codigo, mostrar botones.]. Con  este nuevo funcionamiento hay que hacer que las respuestas de la IA dejen de ser tan cuadradas o literalmente escritas de gui de robot para que maneje un leguaje mas relajado, menos cuadrado y aprenda a decidir cuando decir todo en un solo mensaje y cuando decirlo en 2, por que ahora mismo es inutil activar el boton llenarlo por mi cuando se puedo haber validado de inmediato si ya estaba cubierto el dato en el formulario y entonces se le indica al usuario, se vita que el usuario haga una llamada al modelo que dara como respuesta que ya estaba guardado. 
![[{4F4B6BB1-6AF0-48B3-8735-1518D197F6A3}.png]]

**Validado por el líder (2026-10-18):** las tres casillas bien. Su propuesta de que el Asistente diga en la misma respuesta lo que ya está guardado, hable menos como robot y decida cuándo ofrecer el botón pasó al bloque [[IA-4 El Asistente decide y habla natural]].
