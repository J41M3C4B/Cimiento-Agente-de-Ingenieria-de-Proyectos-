---
adr: 43
estado: aceptada
fecha: 2026-10-18
tema: [producto, nucleo, ia, proyectos]
reemplaza_en_parte: ["[[ADR-032-monolito-modular-nucleo-y-modulos]]", "[[ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento]]"]
tags: [adr]
---
# ADR-043 · SociAI es un ERP con la IA en el núcleo; el MVP es el núcleo y los proyectos

**Estado:** aceptada (2026-10-18, decisión de la persona dueña del proyecto). Cambia la visión, los principios, el mapa y el roadmap; el código cambia por fases (§7).

## Contexto

Cimiento nació como un agente para armar proyectos de donativos en un asilo y una casa hogar I.A.P. Con el tiempo creció hacia un ERP (ADR-032: núcleo y módulos; ADR-034: la IA en todo el ERP), pero los documentos de base siguen diciendo lo de antes:

- la visión (`00-vision.md`) habla de «una app para asilos y casas hogar que arma proyectos»;
- los principios dicen que la IA «solo» pregunta, razona y redacta, y que todo lo demás es una regla del código;
- el plan siguiente era construir los módulos (Personal, Beneficiarios, Instalaciones, Finanzas, Donantes) uno por uno.

Las pruebas del núcleo mostraron el costo de pensar así. Cada caso nuevo pedía una regla más: una palabra más para el escáner («alcaldía Benito Juárez» no es una persona), una frase más para reconocer un documento, una trampa más en la batería. En producción eso no termina nunca, y frena a una persona que no entiende por qué el programa le pregunta.

## Decisión

### 1. Qué es SociAI

Un **ERP para organizaciones de la sociedad civil y sin fines de lucro, con la IA como núcleo y no como complemento**. La IA existe para quitar la fricción y la complejidad técnica de un ERP de siempre: la persona cuenta y sube sus papeles, y el programa ordena, llena, avisa y prepara.

- Su filosofía está enfocada en este sector desatendido (asistencia social y organizaciones parecidas), pero **su uso es genérico**: no supone que la institución es un asilo o una casa hogar I.A.P.
- El nombre del producto es SociAI. El repositorio y el código siguen llamándose `cimiento`.

### 2. El MVP: centralizar la institución y resolver los proyectos

El MVP es la parte que se valida con las dos instituciones que ya prueban el programa. No es el objetivo completo del producto.

1. **El núcleo: centralizar y digitalizar la institución.** Inicio, «Mi institución» (sus ventanas y su Expediente), el Asistente, cuentas y permisos, seguridad y respaldo.
2. **Los proyectos: el problema que hace despegar el negocio.** Las instituciones pierden tiempo redactando e ideando proyectos inviables, o algo que ni ellas entienden. El MVP lleva de una convocatoria a un proyecto que **cabe** en ella y que la institución **entiende**: si conviene, cómo plantearlo y su redacción. Parte de lo que dejaron las fases 2 a 4 (convocatoria, diagnóstico, redacción y guía en Word), revisado con esta visión.

### 3. Lo que queda congelado

**Personal, Beneficiarios, Instalaciones, Finanzas y Donantes** quedan congelados hasta validar el MVP:

- **Se ocultan de la app:** el riel, «Mi institución», el primer inicio, Inicio y el mapa de lugares del Asistente dejan de mostrarlos.
- **Su código se queda.** Compila, sus pruebas siguen corriendo y `cargo test architecture` los sigue revisando. Volver a encenderlos es barato.
- **No se trabaja en ellos.** No se agrega nada nuevo ni se arregla nada que no rompa el núcleo o los proyectos.
- Lo que el núcleo o los proyectos tomaban de ellos (la ocupación, las cifras rápidas, el balance) se resuelve en la fase que los oculta: el núcleo se queda con lo que necesita (por ejemplo, sus cifras rápidas) o deja de mostrarlo.

### 4. El núcleo es genérico desde ya

El núcleo habla de **cualquier organización de la sociedad civil**:

- **Figura legal:** la lista incluye las comunes (A.C., I.A.P., I.B.P., S.C., fundación…). Lo propio de una figura, como el folio de la Junta para las I.A.P. e I.B.P., aparece solo cuando corresponde.
- **A quién atiende:** las opciones no suponen un asilo ni una casa hogar.
- **Expediente y textos:** el Expediente pide los documentos según la figura y lo que hace la institución, y los textos y el manual no hablan de «residentes», «niñas» ni «asilo» salvo en lo que la institución escribe.
- **Las dos instituciones** son los casos de validación y los ejemplos, no el modelo del programa.

Qué cambia exactamente lo decide una revisión del núcleo (fase MVP-3): una lista de lo que está atado a un asilo o a una casa hogar I.A.P., con una recomendación por punto que decide la persona dueña del proyecto.

### 5. La IA decide dentro de la casa

La arquitectura es la casa de la IA, no un vigilante aparte. Un agente serio no trae un glosario de reglas: el entorno le pone **pocos límites que no crecen con cada caso**, y dentro de ellos decide.

**Los límites que pone el código:**

1. **El valor de un dato personal no sale del equipo.** Hacia la IA va una marca, nunca el valor (la bóveda, ADR-044). Personal y Beneficiarios nunca mandan sus fichas.
2. **Nada se guarda sin una persona.** La IA propone; una persona acepta; el código guarda (ADR-003, ADR-034).
3. **La IA no calcula.** Totales, presupuestos y validaciones son del código.
4. **Permisos, no reglas de lenguaje.** Cada agente tiene sus herramientas, cada campo dice qué clase de dato acepta y cada pantalla qué puede ver quién.

**Fuera de esos límites, decide la IA:** qué es cada cosa, a qué campo va, qué documento es, qué preguntar y cómo decirlo. El código revisa lo que se puede revisar sin adivinar: que una cita esté en el texto, que una cifra esté en la cita, que un valor quepa en su campo.

**Cuando un problema pide una regla más para un caso** (una palabra más para el escáner, una frase más para reconocer un tipo), se busca primero el límite o el permiso que la vuelve innecesaria. La regla por caso es el último recurso, y se anota por qué no hubo otra salida.

**El modelo se elige por medición y costo, y la seguridad no depende de él.** Seguimos validando con el modelo más barato, porque si funciona con él funciona con todos. Un modelo más fuerte mejora lo que sugiere, nunca cambia lo que sale del equipo ni lo que se guarda.

**El camino de la persona va hacia una sola conversación.** Contarle al Asistente y subirle los papeles debería bastar para dejar cubierto lo indispensable (nota «Observaciones Pruebas», problema 2). Se decide en su propio ADR, sobre la bóveda.

### 6. Lo que cambia en los documentos

- `00-vision.md` se reescribe con §1 a §5.
- En `docs/agents/principios-de-ingenieria.md`:
  - «Qué es el proyecto» y el principio 1 pasan a decir «la IA decide dentro de la casa»;
  - el principio 5 cambia con el ADR-044;
  - la estructura marca los módulos congelados.
- El roadmap abre la **Fase 11 · MVP**. De la Fase 10, la fase 5 de C4 («metodología de agentes para los módulos») queda en pausa con los módulos, y N5 pasa a la Fase 11.
- `docs/agents/metodologia-lectores.md` sigue valiendo para el Expediente; sus partes de módulos esperan.

### 7. Fases

Un bloque a la vez, cada uno probado en la app antes del siguiente.

| Fase | Qué | Cómo se sabe que terminó |
|---|---|---|
| **MVP-0** | Este ADR y la base reescrita | La visión, los principios, el mapa y el roadmap dicen lo mismo que este ADR |
| **MVP-1** | Ocultar los módulos congelados | La app solo muestra el núcleo y Proyectos; todas las pruebas pasan; nada del núcleo lee un módulo oculto para mostrarlo |
| **MVP-2** | La bóveda (ADR-044): el chat del núcleo sin la ventana de «datos personales» | Ningún valor de la bóveda aparece en ninguna solicitud a la IA, en todas las baterías; una corrida real con el modelo más barato |
| **MVP-3** | El núcleo genérico: revisión, decisiones y cambios | Una institución que no es asilo ni casa hogar llena el primer inicio y su Expediente sin opciones ni textos ajenos |
| **MVP-4** | Proyectos sobre la base nueva | Las dos instituciones preparan un proyecto con una convocatoria real y lo entienden |

## Consecuencias

- **ADR-032 sigue valiendo** como arquitectura (un núcleo, módulos que solo hablan por su `api`, capas que solo miran hacia abajo). Lo que cambia es el plan: los módulos esperan.
- **ADR-034 sigue valiendo** como forma de trabajar de la IA (agentes con herramientas, propuestas que acepta una persona). Lo que cambia es el alcance: solo el núcleo y Proyectos.
- **Lo que ya está hecho se queda.** El Expediente, los Lectores, el Asistente, el Capturista y la política de llenado son el núcleo del MVP.
- **Queda la deuda de reglas por caso.** Lo que hoy es regla por caso (las palabras del escáner, las frases de cada tipo de documento) se revisa en MVP-2 y MVP-3 con el criterio de §5, sin borrarlo antes de tener con qué reemplazarlo.
- **Criterio del MVP:**
  - las dos instituciones dejan su institución completa sobre todo contando y subiendo papeles;
  - preparan un proyecto con una convocatoria real en una fracción del tiempo de hoy, y lo entienden;
  - el costo de IA por proyecto se mantiene en pocos pesos;
  - ningún valor de un dato personal llega a la IA.
