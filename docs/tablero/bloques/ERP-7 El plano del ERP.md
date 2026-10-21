---
tipo: bloque
id: ERP-7
carril: erp
estado: hecho
prioridad: alta
caja: Módulos; Contratos de capacidades; Datos operativos del ERP; Autenticación, permisos y almacén de auditoría
pedido_por:
depende_de:
rama: erp/erp-7-plano-del-erp
adr: ADR-002
creado: 2026-10-20
cerrado: 2026-10-20
---

## Qué y por qué

Los planos dicen qué partes tiene SociAI y cómo pasa una petición, pero no cómo se arma por dentro el ERP: si Mi institución es el núcleo, qué es común a todos los módulos, qué forma tiene un módulo y cómo se reparten los datos entre ellos. Sin eso, cada módulo vuelve a nacer con su propia forma. Este bloque escribe **el plano del ERP** desde los planos y la investigación, sin acomodar el código de hoy, para que los dos rieles depuren el código con él y se levante el núcleo.

## Terminado cuando

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [x] **El plano del ERP** (`docs/plano-del-erp.md`) dice qué es el núcleo, la forma de un módulo, cómo se reparten los datos entre módulos, cómo se prepara y aplica un cambio, el mapa funcional y el veredicto del código de hoy. El líder lo leyó y lo aprobó.
- [x] **Las seis decisiones** (D1 a D6) están aprobadas por el líder.
- [x] **El ADR-002** las recoge. *(Queda aprobado al validar el bloque.)*
- [x] **Los planos y el modelo de trabajo** enlazan el plano del ERP, con su fila en «Ajustes hechos a los planos».
- [x] **El tablero tiene lo que sigue:** ERP-8 y ERP-9 en Backlog; ERP-1, ERP-2 y ERP-3 reacomodados sobre el núcleo; el pedido IA-5 al riel IA.

## Plan

1. Leer los planos, la investigación y, al final, el código, solo para el veredicto.
2. Borrador del plano, revisado con el líder: aprobado el 2026-10-20 con las seis decisiones, como se recomendaban. El líder aclaró que «hacia afuera» se refería a cómo se reparten los datos entre módulos; lo que sale hacia el SAT, los bancos y los financiadores se toma en cuenta, pero todavía no se construye.
3. El ADR-002, los enlaces y el PR.
4. Las tarjetas: ERP-8, ERP-9, el pedido IA-5, y los cambios a ERP-1, ERP-2 y ERP-3.

Es solo documentación: no cambia código.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/22

**Pruebas:** `cargo test`: 626 pasan y 0 fallan; `pnpm test`: 224 pasan. Solo es documentación.

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-erp`, rama `erp/erp-7-plano-del-erp`.
- Antes: nada.
- Arrancar: no hay nada que arrancar. Se lee `docs/plano-del-erp.md` en GitHub (en el PR) o en VS Code con una vista previa de Mermaid.
- Al terminar: nada.

**Qué hacer y qué debe pasar:**

- [x] Los cuatro diagramas del plano se ven dibujados en GitHub.
- [x] El plano y el ADR-002 dicen lo que se aprobó.

## Hallazgos

## Resultado

Fusionado en `main` el 2026-10-20 con *squash* (PR #22, `fd4056e`), validado por el líder.

- **El plano del ERP** (`docs/plano-del-erp.md`) y el **ADR-002**, aceptado, con las seis decisiones (D1 a D6).
- **Los planos y el modelo de trabajo** enlazan el plano del ERP.
- **En el tablero:** ERP-8 (aprobado y en curso) y ERP-9; ERP-1, ERP-2 y ERP-3 sobre el núcleo; el pedido IA-5 al riel IA.
- Al hacer `rebase` sobre el IA-1, el índice de ADR quedó con las dos filas.
- Pruebas: `cargo test` 626 pasan; `pnpm test` 224 pasan. Solo es documentación.
