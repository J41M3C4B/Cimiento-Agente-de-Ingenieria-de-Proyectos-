---
adr: 45
estado: aceptada
fecha: 2026-10-19
tema: [ia, arquitectura, metodologia]
tags: [adr]
---
# ADR-045 · Tres patrones: determinista, IA acotada y agentes; sin chats

**Estado:** aceptada (2026-10-19). La decisión es del líder en la sesión del 2026-10-19; el texto y la metodología los escribió el CTO y el líder los aprobó el mismo día. El número 044 queda para la bóveda (ADR-043 §5).

## Contexto

Al revisar la arquitectura de la IA (bloque IA-5) se vio que llamamos «agente» a todo lo que pasa por `ai::agent::run`:

- **El Capturista y los tres Lectores** (al subir, al llenar y al contestar) hacen una sola llamada, sin herramientas, con la entrada y la salida definidas por el código. No son agentes: son funciones con IA.
- **El Asistente es el único agente:** decide qué herramienta pedir, hasta cuatro, en un bucle. Aun así, la mayor parte de su oficio es enrutar entre cinco caminos conocidos, escritos como casos en el prompt (versión 11).

Sin un criterio para elegir, cada falla se arregla donde cae: un caso más en el oficio del Asistente, o una lista de frases más en el código (`BACK_TO_IT`, `claims_done`, las palabras del escáner). Es la misma «regla por caso» que el ADR-043 quiere quitar, ahora también en los prompts. Y una sola decisión puede quedar partida en dos llamadas unidas por un botón, como pasa con «Llenarlo por mí» (IA-4).

La base sí está bien hecha, y no cambia:
- un solo camino para toda llamada (`ai::pipeline`);
- el proveedor intercambiable;
- las propuestas que acepta una persona (`ai_proposal`);
- los formularios descritos como datos;
- la revisión de citas y cifras por el código;
- las herramientas con permiso por rol;
- las baterías medidas.

## Decisión

### 1. Sin chats

SociAI no tiene un modelo que conteste texto libre sin nada detrás. La caja de conversación es una puerta: lo que la persona escribe entra a un operador, y toda respuesta sale de un operador o de un agente, con su esquema y la revisión del código.

### 2. Tres patrones

| Patrón | Qué es | Quién decide los pasos |
|---|---|---|
| **1 · Determinista** | Reglas y código: la capa más robusta y el núcleo de todo | El código |
| **2 · IA acotada** | Tareas que se conocen de principio a fin: la misma entrada, los mismos pasos y la forma del resultado definida. No hay decisiones de camino a medio proceso. La pieza se llama **operador** | El código; la IA hace el juicio en un punto fijo |
| **3 · Agentes** | Autonomía acotada por permisos: los pasos cambian según lo que decide el modelo y lo que encuentra. Solo donde hay ambigüedad real y el camino no se puede enumerar | El modelo, dentro de sus herramientas y topes; el código pasa a segundo plano |

Que la IA **decida** (qué es un dato, a dónde va, qué ofrecer) no la vuelve agente: ese juicio cabe en un operador. Es agente solo cuando controla el camino.

### 3. Siempre el nivel más bajo

- El patrón se elige con las cuatro preguntas de [`docs/agents/patrones-de-ia.md`](../agents/patrones-de-ia.md) y se anota en el bloque con su por qué.
- Subir de nivel pide una medición que muestre que el de abajo no alcanza.
- Una pieza baja de nivel cuando su batería lo muestra.

### 4. Cómo se componen

- Un agente usa como herramientas funciones deterministas de solo lectura y operadores.
- Un operador no llama a nadie.
- Un agente nunca llama a otro agente: hay un solo nivel de delegación.

### 5. Una ficha por pieza

Cada operador y cada agente nace con su ficha (tarea, entrada, salida, revisión del código, modelo y topes, oficio, batería), escrita antes del código. Las fichas están en `patrones-de-ia.md`, y un operador nuevo copia la de uno que ya existe.

### 6. La capa de IA sigue siendo propia

No adoptamos un framework (LangChain o parecidos). Lo que ofrecería ya lo tenemos en una capa delgada que entendemos: el proveedor, el pipeline, el bucle de herramientas, la búsqueda y la salida estructurada. Además, esconde los prompts y las llamadas, y eso dificulta garantizar que todo pasa por el escáner.

Con Rust el costo es mantener nuestros clientes de Gemini y Anthropic, porque no hay SDK oficial. La ventaja de Rust está en lo local (PDF, OCR, búsqueda, escáner, cifrado) y en lo seguro (tipos y esquemas), no en la velocidad del modelo, que corre en la nube.

## Consecuencias

- **El tablero del carril `ia` se ordena con los patrones:**
  - IA-4 rehace el Asistente como un operador que enruta, con el Capturista como herramienta;
  - IA-6, nuevo, separa operador y agente en el código;
  - IA-2 (la bóveda) y IA-3 no cambian de fondo.
- **Los bloques llevan la propiedad `patron`** (`determinista`, `acotada`, `agente`).
- **La deuda queda a la vista:** la tabla de `patrones-de-ia.md` §6 dice qué está hoy en el patrón equivocado y qué bloque lo resuelve.
- **El nombre del producto es SociAI** en todo lo que se escribe desde hoy. El repositorio, las carpetas y el código siguen llamándose `cimiento` (ADR-043 §1).
