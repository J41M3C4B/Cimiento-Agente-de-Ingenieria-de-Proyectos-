---
tipo: bloque
id: IA-6
carril: ia
estado: cerrado
prioridad: media
fase: —
patron: [acotada, agente]
rama:
adr: "[[ADR-045-tres-patrones-de-la-ia]]"
creado: 2026-10-19
cerrado: 2026-10-20
---

## Qué y por qué

Salió de IA-5. En el código todo lo que usa IA se llama «agente»: el Capturista y los tres Lectores corren por `ai::agent::run` con `tools: &[]` y un `fn agent()`, igual que el Asistente. Quien lee el código no ve qué patrón es cada pieza ([[ADR-045-tres-patrones-de-la-ia]]), y un operador nuevo copiaría el molde equivocado.

Separar en el código:
- **operador** (patrón 2): una llamada, o una secuencia fija, sin herramientas;
- **agente** (patrón 3): el bucle con herramientas y topes.

Los dos van por el mismo pipeline. El cambio es de nombres y de forma, sin cambiar lo que hace ninguna pieza; las tareas en `ai_usage` se quedan como están para no romper las mediciones.

## Terminado cuando

(Por definir.)

- [ ] El Capturista y los Lectores son operadores en el código; el Asistente es el único agente.
- [ ] Un operador no puede tener herramientas: lo impide su tipo, no una revisión.
- [ ] Todas las pruebas y baterías en seco pasan igual que antes.

## Plan

Conviene hacerlo junto con IA-4 o justo antes, porque IA-4 agrega un operador que enruta y vuelve al Capturista herramienta del Asistente.

## Cómo probarlo

## Resultado

**Se cerró sin hacerse (decisión del líder del 2026-10-20, [[ADR-048-el-flujo-de-datos|ADR-048]]):** son solo nombres, y F3 y F4 rehacen esas piezas.

