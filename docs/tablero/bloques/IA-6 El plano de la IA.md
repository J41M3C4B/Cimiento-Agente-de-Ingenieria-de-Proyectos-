---
tipo: bloque
id: IA-6
carril: ia
estado: backlog
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

(Propuesta; la aprueba el líder.)

- [ ] **El plano de la IA** (`docs/plano-de-la-ia.md`) dice:
  - dónde vive la plataforma;
  - sus piezas y un recorrido completo;
  - los tres niveles de ejecución;
  - la tarea;
  - el contrato común de resultados;
  - la política en cuatro puntos;
  - dónde se toca con el ERP y qué le pide;
  - el veredicto del código de hoy.

  El líder lo leyó y lo aprobó.
- [ ] **Las decisiones A1 a A7** están aprobadas por el líder, y las que tocan al ERP las revisó el riel ERP.
- [ ] **El ADR-003** las recoge.
- [ ] **Los planos, el plano del ERP y el modelo de trabajo** enlazan el plano de la IA, con su fila en «Ajustes hechos a los planos».
- [ ] **El tablero tiene lo que sigue:**
  - el IA-2 con su «Terminado cuando» ajustado;
  - los pedidos al riel ERP de la §8, como tarjetas o anotados en las que ya existen.

## Plan

1. Leer los planos, el plano del ERP, los ADR-001 y ADR-002, la investigación y, al final, el código, solo para el veredicto. Revisar la rama del ERP-8, que está en curso, para que el *workspace* cuadre.
2. Borrador del plano, revisado con el líder.
3. Revisar con el riel ERP las decisiones que lo tocan (A1, A2, A3, A5 y A7). La A2 va primero, porque cambia cómo el ERP-8 muda `Permission`.
4. El ADR-003, los enlaces y el PR.
5. Las tarjetas.

Es solo documentación: no cambia código.

## Cómo probarlo

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
