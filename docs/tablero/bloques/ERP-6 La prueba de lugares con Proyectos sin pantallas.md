---
tipo: bloque
id: ERP-6
carril: erp
estado: backlog
prioridad: alta
caja: Mi institución
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

**`main` está en rojo desde que se fusionó el DIS-4** (`e2df0bf`). Falla una prueba de `cargo test`: `core::places::tests::every_page_of_the_bar_has_its_screen_in_the_manual`, con el mensaje «projects must keep its page and its screen, to be turned back on».

La causa es un choque entre dos bloques que se fusionaron bien cada uno por su lado:
- **El ERP-4** dejó `"projects"` en `HIDDEN` (`core/modules.rs`). La regla del ADR-043 dice que un módulo congelado conserva su página y su pantalla del manual, para poder volver a encenderlo.
- **El DIS-4**, por decisión del líder, sacó las pantallas de Proyectos de `src/` (recuperables con la etiqueta `proyectos-congelado`). Con ellas salió `projects` del tipo `Page` de `Shell.tsx`, que es justo lo que la prueba busca.

Se necesita que **`cargo test` vuelva a pasar en `main`**, con una regla que diga qué conserva un módulo congelado cuyas pantallas ya salieron del programa. El IA-1 no se puede fusionar con `main` en rojo. Este bloque no lo causó: su rama no toca esos archivos.
