---
tipo: bloque
id: IA-6
carril: ia
estado: revision
prioridad: alta
caja: Runtime; Enrutador; Planificador; Registro; Gestor de contexto; Ejecutor; Estado, límites y ciclo de vida; Entrega de resultados; Operadores y agentes de IA; Clasificación aplicada, política de datos, validación, confirmación y auditoría de tareas
pedido_por:
depende_de:
rama: ia/ia-6-plano-de-la-ia
adr: ADR-003
creado: 2026-10-21
cerrado:
---

## Qué y por qué

**Lo pidió el líder el 2026-10-21, antes de aprobar el IA-2.** Los planos dicen qué cajas tiene la plataforma de IA, y el [plano del ERP](../../plano-del-erp.md) dice cómo se arma el ERP por dentro. Falta **el plano de la IA**:
- cómo se arman por dentro la plataforma, sus piezas, la tarea, la entrega y la política;
- cómo se acopla al ERP, punto por punto.

Con los dos planos, el riel IA cuadra el suyo con el del ERP, y el riel ERP cuadra el suyo con el de la IA. Sale de los planos y de la investigación, sin acomodar el código de hoy.

## Terminado cuando

Aprobado por el líder el 2026-10-21. Las casillas se marcan cuando cada punto queda hecho.

- [x] **El plano de la IA** (`docs/plano-de-la-ia.md`) dice:
  - dónde vive la plataforma;
  - sus piezas y un recorrido completo;
  - los tres niveles de ejecución;
  - la tarea;
  - el contrato común de resultados;
  - la política en cuatro puntos;
  - dónde se toca con el ERP y qué le pide;
  - el veredicto del código de hoy.

  El líder lo leyó y lo aprobó.
- [x] **Las decisiones A1 a A7** están aprobadas por el líder (2026-10-21), y lo que toca al ERP está pedido en el ERP-10.
- [x] **El ADR-003** las recoge. *(Queda aprobado al validar el bloque.)*
- [x] **Los planos y el modelo de trabajo** enlazan el plano de la IA, con su fila en «Ajustes hechos a los planos». El enlace desde el plano del ERP lo pone el riel ERP (ERP-10).
- [x] **El tablero tiene lo que sigue:**
  - el IA-2 con su «Terminado cuando» ajustado;
  - los pedidos al riel ERP de la §8, juntos en el ERP-10.

## Plan

1. Leer los planos, el plano del ERP, los ADR-001 y ADR-002, la investigación y, al final, el código, solo para el veredicto. Revisar la rama del ERP-8, que está en curso, para que el *workspace* cuadre.
2. Borrador del plano, revisado con el líder: aprobado el 2026-10-21 con las siete decisiones, como se recomendaban.
3. El ADR-003, los enlaces y el PR.
4. Las tarjetas: el pedido ERP-10 con lo que toca al ERP (A1, A2, A3, A5 y A7; la A2 va primero, porque cambia cómo el ERP-8 muda `Permission`), y el IA-2 ajustado.

Es solo documentación: no cambia código.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/23

**Pruebas:** `cargo test`: 587 pasan y 0 fallan; `pnpm test`: 224 pasan. Solo es documentación.

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-ia`, rama `ia/ia-6-plano-de-la-ia`.
- Antes: nada.
- Arrancar: no hay nada que arrancar. Se lee `docs/plano-de-la-ia.md` en GitHub o en Obsidian, donde los diagramas se ven dibujados.
- Al terminar: nada.

**Qué hacer y qué debe pasar:**

- [ ] Los cuatro diagramas del plano se ven dibujados.
- [ ] La §7 cuadra con el plano del ERP, punto por punto.

## Hallazgos

## Resultado
