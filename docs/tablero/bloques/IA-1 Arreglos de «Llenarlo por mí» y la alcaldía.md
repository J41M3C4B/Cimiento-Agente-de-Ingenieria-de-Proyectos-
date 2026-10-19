---
tipo: bloque
id: IA-1
carril: ia
estado: revision
prioridad: alta
fase:
rama: claude/quirky-newton-tuel4h
adr:
creado: 2026-10-18
cerrado:
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
- [ ] Con la alcaldía ya guardada, «Llenarlo por mí» dice «Ya estaba guardado igual» y no se queda en «Llenando…».
- [ ] Si el escáner pregunta y contestas, tu frase no vuelve a aparecer.
- [ ] «Estamos ubicados en la alcaldía Benito Juárez» ya no pregunta por datos personales.

## Resultado
