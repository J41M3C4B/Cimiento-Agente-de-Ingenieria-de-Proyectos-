---
tipo: bloque
id: ERP-6
carril: erp
estado: hecho
prioridad: alta
caja: Mi institución
pedido_por: IA-1
depende_de:
rama: erp/erp-6-proyectos-fuera
adr:
creado: 2026-10-20
cerrado: 2026-10-21
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
- [x] **Se puede recuperar:** una etiqueta de git, `proyectos-rust-congelado`, apunta al último `main` que tiene el módulo de Rust sin IA (la de `proyectos-congelado`, del DIS-4, apunta a antes de quitarle la IA).
- [x] **No se borra ningún dato de la persona.** Las tablas y las migraciones se quedan: un respaldo y una restauración siguen funcionando, y quien vuelva a encender Proyectos encuentra sus datos. `core/security.rs` y `storage/backup.rs` siguen conociendo esas tablas.
- [x] **El manual ya no tiene la pantalla de Proyectos** (`screen.projects` y lo que diga de ella).
- [x] **Pruebas:** *(`cargo test`: 626 pasan, 0 fallan; `pnpm test`: 224 pasan; `pnpm build` pasa.)* `cargo test`, `pnpm test` y `pnpm build` pasan, sin red ni gasto.
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

PR: https://github.com/J41M3C4B/SociAI/pull/21

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-erp`, rama `erp/erp-6-proyectos-fuera`.
- Antes: nada.
- Arrancar: `pnpm tauri dev`.
- Al terminar: cerrar la app.

**Qué hacer y qué debe pasar:**

- [ ] La app arranca con una base que ya tenía proyectos, sin errores, y no hay rastro de Proyectos.
- [ ] Inicio, Mi institución, Documentos y la configuración de la ayuda automática funcionan igual.
- [ ] En *Ayuda*, buscar «proyecto convocatoria»: no sale ninguna pantalla de Proyectos.
- [ ] Un respaldo se crea y se restaura sin error.

## Hallazgos

- **Borrar un proyecto sigue funcionando:** aprobar la solicitud de borrado de un proyecto lo hace ahora `core::access` (antes llamaba al módulo), con una prueba nueva. `resolve_request` ya no recibe el callback `delete_project`. Las tablas de Proyectos y sus datos se quedan.
- **`core::api` salió también** (49 líneas): era solo la puerta de Proyectos y no tenía consumidor.
- **Para diseño:** `lib/tauri.ts` y `lib/types.ts` conservan llamadas y tipos de comandos que ya no existen. No se quitaron porque `HomePage.test.tsx` (DIS-4) importa `projectList`. Hay que ajustar esa prueba y limpiar los dos archivos juntos.
- **Decisión del líder (2026-10-21) sobre lo que quedó sin consumidor:** la lectura de convocatorias (`documents/canonical`: `assemble`, `card`, `contract`, `normalize`, `package`, `retrieve`, `run`, `summary`) y lo que solo ella usaba (`documents/docx.rs`, `chunking.rs`, `pdf.rs`, `reading.rs`, y partes de `ai/`: `golden`, versiones de *prompts*) **quedan detenidas, no eliminadas: desacopladas hasta que se puedan conectar bien a un núcleo sólido.** El compilador las marca «nunca usadas»; es lo esperado y no se retiran. No hace falta un bloque ERP-7 para borrarlas.
- **El manual conserva frases viejas** en los campos de Mi institución («aparece en los documentos y proyectos que arma el programa», «la ayuda automática lo usa para…») y en la pantalla de Ayuda («los pasos de un proyecto»). No es de este bloque; entra en la pasada del manual.

## Resultado

Fusionado a `main` con *squash* en el PR #21 (`5e6bffc`), probado por el líder el 2026-10-21. `main` vuelve a verde.

**Qué quedó:**
- Proyectos salió del programa: `modules/projects` (unas 5 200 líneas con sus pruebas), sus 27 comandos, su registro y permisos, `ProjectsError`, su capa en `architecture_tests` y `test_support`. `projects` ya no está en `HIDDEN` ni en el manual.
- Recuperable con la etiqueta `proyectos-rust-congelado`. Ningún dato se borró: las tablas y migraciones se quedan.
- Aprobar el borrado de un proyecto lo hace ahora `core::access` (con prueba nueva); `resolve_request` ya no recibe el callback. `core::api` salió también.
- `cargo test`: 626 pasan. `pnpm test`: 224 pasan. `pnpm build` pasa.

**Lo que queda para otros rieles:**
- **Diseño:** `lib/tauri.ts` y `lib/types.ts` conservan llamadas y tipos de comandos que ya no existen; `HomePage.test.tsx` (DIS-4) importa `projectList`. Se ajusta esa prueba y se limpian ambos archivos juntos.
- **IA:** el IA-1 (#20) ya puede entrar a `main`. `ai/dry_run_tests.rs` sigue apagado y nombra a Proyectos.
- **Manual:** frases viejas en los campos de Mi institución y en Ayuda («los pasos de un proyecto»); entran en la pasada del manual.
