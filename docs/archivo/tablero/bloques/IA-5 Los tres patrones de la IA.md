---
tipo: bloque
id: IA-5
carril: ia
estado: hecho
prioridad: alta
fase: MVP-2
patron: [determinista, acotada, agente]
rama: ia/ia-5-tres-patrones
adr: "[[ADR-045-tres-patrones-de-la-ia]]"
creado: 2026-10-19
cerrado: 2026-10-19
---

## Qué y por qué

Lo pidió el líder el 2026-10-19 al revisar la arquitectura de la IA. Llamábamos «agente» a todo lo que usa IA, y sin un criterio cada falla se arreglaba donde caía: un caso más en un prompt o una lista de frases más en el código.

La decisión: **sin chats, y tres patrones** (determinista, IA acotada y agentes), con una metodología que diga siempre cuál toca, para que cada operador nuevo (como el Capturista o el Lector) se diseñe igual.

## Terminado cuando

(Pedido por el líder en la sesión del 2026-10-19.)

- [x] Un ADR registra la decisión: sin chats, tres patrones, el nivel más bajo que resuelve, cómo se componen, y la capa de IA propia sin framework.
- [x] Una metodología en `docs/agents/` dice cómo elegir el patrón (las cuatro preguntas), la ficha de cada patrón, las señales de un patrón equivocado y cómo nace una pieza nueva.
- [x] Lo que existe hoy está clasificado por patrón, con su deuda y el bloque que la resuelve.
- [x] Los bloques del carril `ia` están ajustados a los patrones, con la propiedad `patron`.
- [x] Las instrucciones y la memoria dicen SociAI, y que `cimiento` es solo el nombre del repositorio y del código.

## Plan

Solo documentos, sin código. En su propia carpeta (`D:\Agente Proyectos\cimiento-ia`) para no tocar la del carril `erp`.

1. [[ADR-045-tres-patrones-de-la-ia]], en estado «propuesta» hasta que el líder lo lea. El 044 queda para la bóveda.
2. [[patrones-de-ia|docs/agents/patrones-de-ia.md]]: las cuatro preguntas, lo que vale en los tres, las señales, las fichas, cómo nace una pieza y la tabla de lo que hay hoy.
3. Principios (un principio 9), `07-ia-y-costos.md` y la metodología de los Lectores apuntan a ella. `CLAUDE.md` la carga.
4. Tablero:
   - IA-4 se replantea con los patrones (operador que enruta y el Capturista como herramienta);
   - IA-6, nuevo, separa operador y agente en el código;
   - IA-2 lleva su patrón (IA-3 vive en la rama de ERP-1 y lo recibe después de que se fusione);
   - la plantilla de bloque lleva `patron`.

## Cómo probarlo

- [x] Lee el [[ADR-045-tres-patrones-de-la-ia|ADR-045]]. ¿Dice lo que decidiste? Si sí, cambia su `estado` a `aceptada`.
- [x] Lee [[patrones-de-ia]] y pon a prueba las cuatro preguntas con un caso que tengas en mente, por ejemplo un operador para Proyectos o la bienvenida del primer inicio. ¿Te lleva al patrón que esperabas?
- [x] Revisa la tabla de §6 («Lo que tenemos hoy» y la deuda). ¿Falta algo o algo está mal clasificado?
- [ ] Revisa [[IA-4 El Asistente decide y habla natural|IA-4]] y contesta su pregunta: ¿el botón «Llenarlo por mí» se queda como permiso o pasa a ser «Ver sugerencias»? *(Sigue abierta en IA-4: se contesta al definir su «Terminado cuando».)*

## Resultado

Validado por el líder el 2026-10-19: leyó el ADR, entendió la división y lo aprobó. Fusionado a `main` con *squash*. Solo documentos; no cambió código.

- [[ADR-045-tres-patrones-de-la-ia|ADR-045]] **aceptado**: sin chats, tres patrones (determinista, IA acotada con operadores, agentes), siempre el nivel más bajo y la capa de IA propia, sin framework.
- La metodología vive en [[patrones-de-ia|docs/agents/patrones-de-ia.md]] y la carga `CLAUDE.md`. Los patrones también son la frontera entre los dos CTO ([[roles-y-tablero|Roles y tablero]], escrito por el carril `erp`).
- Todos los bloques `ia` llevan su `patron`: IA-2 e IA-3 son deterministas; IA-4 e IA-6, operador y agente.
- **Queda abierto:** la pregunta del botón, en IA-4.
