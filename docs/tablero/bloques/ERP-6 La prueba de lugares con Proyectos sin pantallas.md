---
tipo: bloque
id: ERP-6
carril: erp
estado: en-curso
prioridad: alta
caja: Mi institución
pedido_por: IA-1
depende_de:
rama: erp/erp-6-proyectos-fuera
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

**Decisión del líder (2026-10-21): el módulo Proyectos sale por completo.** No se busca una regla para un módulo congelado sin pantallas: Proyectos deja de existir en el programa. Una regla de ese tipo solo serviría para un módulo con código y sin pantalla, y la memoria del proyecto dice que lo que no tiene consumidor se retira. Las pantallas ya salieron con el DIS-4 (#19); este bloque saca el resto: el código de Rust, sus comandos y su lugar en el manual.

## Terminado cuando

Aprobado por el líder el 2026-10-21. Las casillas se marcan cuando cada punto queda hecho.

- [x] **`cargo test` vuelve a pasar en `main`**, incluida `every_page_of_the_bar_has_its_screen_in_the_manual`. No porque se afloje la prueba, sino porque Proyectos ya no es un módulo congelado: no está en `HIDDEN`, ni en el manual, ni en `Shell.tsx`.
- [x] **El módulo `modules/projects` (unas 5 200 líneas, con sus pruebas) sale del código:** sus comandos (`project_*`, `need_*`, `drafting_*`, `section_*`, `budget_*`, `schedule_*`, `review_get`, `guide_export`, `conversation_get`, `diagnosis_summary_*`), su registro en `lib.rs`, sus permisos en `core::access::domain`, `ProjectsError` en `error.rs`, su capa en `architecture_tests.rs` y los ayudantes de prueba que solo él usaba.
- [ ] **Se puede recuperar:** una etiqueta de git, `proyectos-rust-congelado`, apunta al último `main` que tiene el módulo de Rust sin IA (la de `proyectos-congelado`, del DIS-4, apunta a antes de quitarle la IA).
- [x] **No se borra ningún dato de la persona.** Las tablas y las migraciones se quedan: un respaldo y una restauración siguen funcionando, y quien vuelva a encender Proyectos encuentra sus datos. `core/security.rs` y `storage/backup.rs` siguen conociendo esas tablas.
- [x] **El manual ya no tiene la pantalla de Proyectos** (`screen.projects` y lo que diga de ella).
- [x] **Pruebas:** `cargo test`, `pnpm test` y `pnpm build` pasan, sin red ni gasto.
- [x] **Se fusiona antes del IA-1** (#20), que no puede entrar con `main` en rojo.

## Plan

**Rama:** `erp/erp-6-proyectos-fuera`, de `main` al día.

**Pasos:**
1. La etiqueta `proyectos-rust-congelado` en el `main` actual.
2. Quitar `modules/projects` y lo que solo él usa: los comandos, el registro, los permisos, `ProjectsError`, la capa de `architecture_tests`.
3. Quitar `projects` de `HIDDEN` y la pantalla `screen.projects` del manual; ajustar las pruebas que lo nombraban.
4. Revisar lo que el compilador diga que quedó sin consumidor, y apuntarlo en «Hallazgos» en vez de arrastrarlo (por ejemplo, la lectura de convocatorias de `documents/canonical`, que solo Proyectos usaba, y la puerta `core::api` que existía para él).
5. `cargo test`, `pnpm test`, `pnpm build`, finales de línea, PR con «Cómo probarlo», y el bloque a «Revisión».

**Lo que no entra:** la lectura de convocatorias y las demás piezas de `documents/` que quedarían sin dueño (lo decide el líder aparte), y borrar las tablas.

**Riesgos:**
- Pruebas de otros módulos que usaban ayudantes de Proyectos (`test_support`, las del puente del núcleo): se ajustan, sin cambiar lo que prueban.
- `ai/dry_run_tests.rs` (apagado) nombra a Proyectos; no compila, así que no estorba, y el riel IA lo porta.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

## Resultado
