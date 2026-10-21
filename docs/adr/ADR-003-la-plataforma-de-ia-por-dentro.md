---
adr: 3
estado: aceptada
fecha: 2026-10-21
tema: [ia, plataforma, runtime, tareas, politica, arquitectura]
bloque: IA-6
tags: [adr]
---
# ADR-003 · La plataforma de IA por dentro y cómo se acopla al ERP

**Estado:** aceptada (2026-10-21). El líder aprobó el plano de la IA y sus siete decisiones en el bloque IA-6.

## Contexto

Los [planos](../planos-de-sociai.md) dicen qué cajas tiene la plataforma de inteligencia y por dónde pasa una petición. El ADR-001 fija el contrato de capacidad y el registro. El [plano del ERP](../plano-del-erp.md) (ADR-002) dice cómo se arma el ERP por dentro.

Faltaba lo mismo del lado de la IA: cómo se arma la plataforma, qué es una tarea, qué sale de la entrega, dónde se aplica la política y cómo se acopla al ERP sin que el ERP la conozca. Sin eso, cada bloque del riel IA decidiría su parte por su lado, y los dos rieles no tendrían contra qué cuadrar sus planos.

Este ADR fija esa arquitectura desde los planos y la investigación, sin acomodar lo que ya existe. El detalle, con sus diagramas, está en [el plano de la IA](../plano-de-la-ia.md).

## Decisión

### 1. Las piezas

La plataforma tiene doce piezas (plano, §3):
- P1, el runtime;
- P2, la tarea, con su estado, sus límites y su ciclo de vida;
- P3, el registro de capacidades;
- P4, el enrutador;
- P5, el planificador de flujos, operadores y agentes;
- P6, el gestor de contexto;
- P7, el ejecutor;
- P8, la aprobación;
- P9, la entrega de resultados;
- P10, la política de datos;
- P11, los modelos y proveedores;
- P12, la auditoría de las tareas.

### 2. Las reglas

- **Una sola puerta.** El chat, las pantallas y los hechos del calendario entran al runtime. No hay otra forma de invocar una capacidad.
- **La tarea es la unidad.** Lleva su id de correlación, su estado y sus límites, y cada turno se cierra con registro: terminada, pausada, denegada, por límite o con error.
- **Un solo ejecutor.** Es lo único que invoca una capacidad y lo único que llama a un modelo. Aplica antes y después los mismos controles: límites, permisos, política de datos y validación de los esquemas.
- **Tres niveles de ejecución:** flujo determinista, operador y agente. Se usa el más simple que sirve.
- **Los límites son de la tarea,** no del agente: pasos, reintentos, gasto reservado antes de cada llamada, tiempo y plazo de una pausa. Un ciclo sin progreso se corta, y un cambio incierto nunca se repite a ciegas.
- **La política es una sola decisión** sobre la clase del dato, el destino y la finalidad. Se aplica en cuatro puntos: antes del enrutador, al armar el contexto, en el ejecutor y en la entrega. Ninguna regla de qué ve la IA vive en un *prompt*.
- **Un contrato común de resultados:** respuesta, datos, aclaración, propuesta, estado y error, todos con el id de la tarea y su destino.

### 3. Las siete decisiones

| | Decisión |
|---|---|
| **A1** | La plataforma es un *crate* propio del *workspace*, que depende del contrato y del núcleo. Ni el núcleo ni un módulo pueden usarla. |
| **A2** | El contrato de capacidad va en el *crate* más bajo, y nombra el permiso por su id. El catálogo de permisos, los roles y la decisión de autorizar son de K2. Así el núcleo también publica capacidades. |
| **A3** | Las tareas se guardan en la base, en tablas de la plataforma (`ai_`), con las migraciones por dueño del K4. |
| **A4** | Una pantalla que nombra una capacidad entra al runtime como flujo determinista: sin enrutador ni modelo, y con los mismos controles. |
| **A5** | La clase de un dato en la salida de una capacidad sale del diccionario: el tipo de salida nombra el campo, y el `data` del contrato es el tope. |
| **A6** | Los operadores y los agentes se declaran en la plataforma, en el mismo registro (`ia.…`), y los invoca el mismo ejecutor. Ningún módulo declara un operador. |
| **A7** | Las reacciones a hechos y los vencimientos corren como tareas de un actor del sistema con permisos mínimos. Nunca aplican un cambio que pida aprobación: lo dejan preparado y avisan. |

### 4. El acoplamiento con el ERP

La tabla de la §7 del plano dice, para cada uno de los quince puntos donde se tocan, quién lo define y quién lo usa. Lo que la plataforma necesita del núcleo y todavía no existe (plano, §8) se pidió al riel ERP en el **ERP-10**:
- el contrato debajo del núcleo;
- la pregunta de autorización y las clases que ve cada cuenta (K2);
- la plataforma como dueña de sus tablas (K4);
- saber si una huella se aplicó (K5);
- el evento de tarea (K9);
- el actor del sistema (K8);
- los tipos de salida que nombran sus campos del diccionario.

## Lo que este ADR no decide

- **Las clases de datos definitivas y los valores de la política:** el ADR del IA-3.
- **La forma de la propuesta y de la huella:** el ERP-9, acordada con el riel IA antes de cerrarlo.
- **Cómo se ven los resultados:** el riel de diseño (DIS-1 y DIS-2).

## Consecuencias

- **El orden del riel IA** es el de la §11 del plano: el esqueleto sin IA (IA-2), el contrato y la plataforma como *crates* (IA-5), las clases y la política (IA-3), el enrutador con el primer operador (IA-4), el agente con su batería y las reacciones a hechos.
- **El IA-2 ajusta su «Terminado cuando»:** las tareas se guardan en la base, y una pantalla invoca una capacidad por el runtime sin modelo.
- **El ERP-8 cambia cómo muda `Permission`** (A2). Por eso es lo primero del ERP-10.
- **El código de hoy se depura** con la §10 del plano. El ciclo del agente pasa a usar capacidades por el ejecutor, y lo para la tarea. La hoja de la IA (`core/ai_sheet.rs`) se retira. El escáner queda como segunda capa de la política, y deja de decidirla.

## Alternativas descartadas

- **Que cada agente traiga sus controles,** como hoy: es la forma que los planos quieren evitar, y la que dejó el ciclo sin fin en `ai/agent.rs`.
- **Tareas solo en memoria:** una pausa o una aprobación se perderían al cerrar la app, y un cambio incierto no se podría revisar.
- **Que las pantallas llamen a las capacidades sin pasar por el runtime:** se saltarían la auditoría y los límites, y lo determinista tendría otro camino que lo que pide la IA.
- **El permiso como tipo del núcleo dentro del contrato:** el contrato dependería del núcleo, y el núcleo no podría publicar capacidades. El gestor de contexto tendría que leer los documentos por fuera del ejecutor.
