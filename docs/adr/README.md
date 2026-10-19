# Registro de decisiones de arquitectura (ADR)

Cada decisión importante se documenta aquí. Formato: contexto, decisión, consecuencias, estado. Una decisión no se edita: si cambia, se crea un ADR nuevo que la reemplaza.

Cada ADR empieza con sus propiedades (para Obsidian y Dataview, ver el [Mapa del proyecto](../00-mapa-del-proyecto.md)): `adr` (número), `estado` (`propuesta`, `aceptada`, `reemplazada`), `fecha`, `tema`, y cuando aplica `reemplaza`, `reemplaza_en_parte`, `reemplazado_por` y `reemplazado_en_parte_por` con enlaces `[[ADR-0NN-…]]`. Un ADR nuevo las lleva desde el principio, y el que reemplaza actualiza las del ADR reemplazado.

**Se empieza en limpio desde el ADR-043 (2026-10-18).** Los ADR 001 a 042 están archivados en [`docs/archivo/adr/`](../archivo/adr/README.md) y no se consultan en el trabajo diario. Los números siguen desde el 044.

| ADR | Decisión | Estado |
|---|---|---|
| 043 | SociAI es un ERP para organizaciones de la sociedad civil con la IA en el núcleo; el MVP es el núcleo y los proyectos; Personal, Beneficiarios, Instalaciones, Finanzas y Donantes congelados y ocultos; el núcleo genérico desde ya; la IA decide dentro de la casa con cuatro límites del código (el valor de un dato personal no sale, nada se guarda sin una persona, la IA no calcula, permisos) en lugar de reglas por caso | Aceptada (2026-10-18); se aplica por bloques del tablero |
| 044 | La bóveda: hacia la IA van marcas, nunca el valor de un dato personal | Reservado (bloque IA-2) |
| 045 | Sin chats; tres patrones para todo lo que usa IA: determinista, IA acotada (operadores) y agentes; siempre el nivel más bajo que resuelve, elegido con las cuatro preguntas de `docs/agents/patrones-de-ia.md`; la capa de IA sigue siendo propia, sin framework | Propuesta (2026-10-19) |
