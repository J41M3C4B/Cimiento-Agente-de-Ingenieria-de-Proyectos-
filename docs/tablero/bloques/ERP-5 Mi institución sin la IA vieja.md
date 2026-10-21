---
tipo: bloque
id: ERP-5
carril: erp
estado: hecho
prioridad: alta
caja: Mi institución
pedido_por: IA-1
depende_de:
rama: erp/erp-5-mi-institucion-sin-la-ia-vieja
adr:
creado: 2026-10-20
cerrado: 2026-10-20
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones. Para cumplirla, el IA-1 retira la IA vieja que vive dentro de Mi institución y saca los vectores de `core`:
- el Asistente, el Capturista y sus herramientas (`core::agents::assistant`, `capturist` y `tools`);
- la IA del Lector (`core::reader::ask`, `core::reader::fill` y la lectura con IA al adjuntar, `core::reader::upload::after_upload`);
- los vectores (`core::archive::vectors`), que pasan a la plataforma.

Estas partes de Mi institución usan algo de eso:
- `core/profile/forms.rs`: la vista de un formulario usa `capturist::Fill` y `fill_of_form`;
- `core/overview.rs` y la prueba de `core/places.rs`: recorren `capturist::FILLABLE`;
- `commands/records.rs`: al adjuntar un documento, `suggest_from` llama a la lectura con IA y emite `record-suggested`;
- `core/onboarding/service.rs`: lee la configuración de la IA (`ai::settings`) para saber si está lista;
- `core/archive/search.rs`: la búsqueda por significado usa los vectores.

Se necesita que **Mi institución deje de depender de esas piezas y de la IA**. Lo que hacían se vuelve a construir sobre la plataforma; no se conserva.

Las propuestas por campo (`core::agents::proposals`) y la lectura con reglas del Lector (`core::reader`, sin IA) no se retiran en el IA-1.

## Terminado cuando

Aprobado por el líder el 2026-10-20 (con las recomendaciones del riel ERP). Las casillas se marcan cuando cada punto queda hecho.

- [x] **Mi institución no usa la IA vieja ni la plataforma de IA.** Fuera de `commands/`, nada del código de Mi institución (`core/`, salvo lo que retira el IA-1) importa `crate::ai`, `core::agents::{assistant, capturist, tools}`, `core::reader::{ask, fill, upload}` ni `core::archive::vectors`. Una búsqueda lo demuestra, y la prueba de fronteras sigue pasando.
- [x] **Las propuestas por campo siguen funcionando sin el Asistente.** `proposals_reject` vive en `commands/institution.rs`, con el mismo nombre, permiso y firma. Las propuestas salen de `core/agents/` (a `core/proposals.rs`), de modo que `core/agents/` solo contiene lo que el IA-1 borra. La excepción del Capturista (`told_own`) desaparece. Las pruebas de propuestas, de la lectura con reglas y de la revisión de sugerencias pasan sin cambio de comportamiento.
- [x] **La búsqueda del Expediente es solo por palabras.** `core/archive/search.rs` no usa vectores, no pide ningún vector y `Found` no trae `by_meaning`. `documents_search` responde por palabras, sin internet y sin gasto. La búsqueda por significado vuelve como herramienta de la plataforma, no aquí.
- [x] **Al adjuntar un documento ya no hay lectura con IA.** `commands/records.rs` no llama a `after_upload` ni emite `record-suggested`. La lectura con reglas, que deja sugerencias con documento y página, sigue igual.
- [x] **`FILLABLE` desaparece.** *(`FormView.fill` y `RecordSuggested` salieron de `types.ts` al fusionarse el DIS-3. `Fill` se queda hasta el IA-1, porque `AssistantOutcome` lo usa.)* `core/overview.rs` y la prueba de `core/places.rs` usan `institution::forms::FORMS` y el mapa de lugares.
- [x] **El onboarding no sabe que existe la IA.** `core::onboarding::service` recibe `ai_ready` como dato; el comando lo calcula desde la configuración de la plataforma. La tarjeta «Llave puesta» de la bienvenida y de Inicio queda igual.
- [x] **El manual ya no habla del Asistente.** Salen `howto.assistant`, `howto.fill` y `howto.fill_documents` de `docs/manual/programa.md`, y lo que el `README.md` del manual dice del asistente. El índice del manual y las pruebas de `places.rs` pasan.
- [x] **Pruebas:** *(`cargo test`: 788 pasan; `pnpm test`: 218 pasan; `pnpm build` pasa.)* `cargo test` (desde `src-tauri/`) y `pnpm test` pasan sin red ni gasto. Hay una prueba nueva por cada punto que cambia (rechazar propuestas, búsqueda sin vectores, onboarding con `ai_ready` dado, formularios sin `fill`).
- [x] **Se fusiona después del DIS-3 y antes del IA-1.** *(DIS-3 ya está en `main` (#16); la rama está rebasada sobre él.)* Con el código del Asistente aún presente (lo borra el IA-1), la app compila y funciona sin llamarlo desde Mi institución.

## Plan

**Rama:** `erp/erp-5-mi-institucion-sin-la-ia-vieja`, de `main` al día, en la carpeta `sociai-erp`. Se toca `core/`, `commands/`, `docs/manual/`, y de `src/` solo `lib/types.ts` (y `lib/tauri.ts` si hace falta); las pantallas son de DIS-3.

**Lo que no se toca:** `core/agents/{assistant, capturist, tools}`, `core/reader/{ask, fill, upload}` y `core/archive/vectors.rs`. Los retira o mueve el IA-1; aquí solo se quitan las piezas de Mi institución que los usan, para que el IA-1 pueda borrarlos sin dejar nada colgando. Los comandos `assistant_ask`, `capture_ask` e `institution_fill_*` tampoco: los quita el IA-1 con su registro en `lib.rs`.

**Decisiones tomadas con el líder (2026-10-20), según los planos:**
1. **Propuestas:** siguen hasta que el IA-2 traiga la aprobación atada a los parámetros. Se mueven a `core/proposals.rs`; el comando, a `commands/institution.rs`.
2. **Búsqueda:** solo palabras. El vector de la pregunta y la búsqueda de vecinos son de la plataforma (regla 1 y regla 3 del plano global).
3. **Onboarding:** la configuración de la IA es de la plataforma y se queda (IA-1); Mi institución recibe el dato, no lo calcula.
4. **`FILLABLE`:** es la lista de `FORMS`; el lugar de cada ventana ya lo da `places::direct`. Se borra, no se mueve. `describe`, `plain`, `open_fields` y `reason` mueren con `fill`, `ask` y `upload` (IA-1).
5. **Manual:** es del ERP y se compila en la app; se limpia aquí.

**Los pasos:**
1. La rama y la lista exacta de usos (búsqueda en `src-tauri/src/`).
2. Propuestas: mover `proposals.rs` a `core/proposals.rs`, actualizar las rutas, quitar `told_own` y mover `proposals_reject`.
3. Búsqueda: quitar vectores de `search.rs`, `Found.by_meaning` y el parámetro `embedder` de `documents_search`.
4. Quitar `suggest_from` y `record-suggested` de `commands/records.rs`.
5. Quitar `FILLABLE`: `overview.rs`, `places.rs` (prueba), `FormView.fill` y los tipos del front.
6. Onboarding: `ai_ready` como parámetro, calculado en `commands/onboarding.rs`.
7. Manual: quitar las tres secciones y revisar el `README.md`.
8. `cargo test`, `pnpm test`, búsqueda del primer punto, revisar finales de línea, `rebase` sobre `main`.
9. PR con «Cómo probarlo», y el bloque a «Revisión».

**Riesgos:**
- Hasta el IA-4, la búsqueda por significado no existe. El líder ya lo aceptó (2026-10-20).
- Las pruebas de `core/agents/*`, `core/reader/*` y de `core/archive/vectors.rs` que usan lo que se mueve (`proposals`, `search`) se ajustan en rutas, no en lógica; el resto lo borra el IA-1.
- `ai/dry_run_tests.rs` y otras pruebas del riel IA pueden usar `search_documents` con `embedder`: si ocurre, se avisa al IA en vez de tocarlas.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/17

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-erp`, rama `erp/erp-5-mi-institucion-sin-la-ia-vieja`.
- Antes: `pnpm install` (ya hecho en esa carpeta). El OCR ya está en `src-tauri/resources/ocr`.
- Arrancar: `pnpm tauri dev`.
- Al terminar: cerrar la app. No deja datos fuera de su carpeta de datos.

**Qué hacer y qué debe pasar:**

- [x] En *Documentos*, buscar «extintores» con un expediente cargado: salen pasajes con documento y página, sin internet y sin pedir nada a la IA.
- [x] En *Mi institución*, con una sugerencia pendiente (subiendo la constancia fiscal ficticia), «Descartar» funciona y la sugerencia no vuelve.
- [x] Adjuntar un documento al Expediente: llegan los avisos de la lectura con reglas (lo que se llenó y las sugerencias), y ninguna sugerencia «de la IA».
- [x] En *Inicio* (administrador, recién instalado): la tarjeta de ayuda automática dice «Llave puesta» solo si hay llave guardada.
- [x] En *Ayuda*, buscar «asistente»: no aparece ninguna sección que explique usarlo.

*(En esta rama el botón ✨ y «Llenar con mis documentos» siguen en pantalla: los quita el DIS-3, y los comandos, el IA-1.)*

## Hallazgos

- **Corrección al plan:** `proposals_reject` quedó en `commands/mod.rs`, junto a `form_save`, no en `commands/institution.rs` (ese es de la junta y los bancos).
- **Para el riel IA:** `reader/upload.rs` es casi todo apoyo de la lectura con IA (`asked`, `detect`, `context`, `speaking`): el IA-1 debe borrarlo entero, no solo `after_upload`. Se borraron 4 pruebas del Capturista que comprobaban la excepción retirada (`told_own`). Más detalle en el PR.
- **Para el DIS-3:** quitar `fill` de `FormView` obliga a borrar una línea en `FormWindow.test.tsx` (el dato de ejemplo). Lo hace este bloque al rebasar sobre el #16.
- En una corrida de `pnpm test` hecha mientras compilaba Rust salieron «7 errors» con 29 archivos; repetida sola, pasó limpia (36 archivos, 218 tests). No guardé el detalle del primer error.

## Resultado

Fusionado a `main` con *squash* en el PR #17 (`65a218a`), probado por el líder el 2026-10-20 en la app.

**Qué quedó:**
- Mi institución ya no usa la IA ni la plataforma fuera de `commands/`. Lo que sigue importando lo viejo es solo lo que el IA-1 retira (`core/agents/{assistant, capturist, tools}`, `core/reader/{ask, fill, upload}` y `archive/vectors`).
- Las propuestas por campo viven en `core/proposals.rs`, sin la excepción del Capturista. `proposals_reject` quedó en `commands/mod.rs`, con el mismo nombre, permiso y firma.
- La búsqueda del Expediente es solo por palabras. La parte de significado, con sus pruebas, está en `archive/vectors.rs`, lista para que el IA-1 la lleve a la plataforma.
- Adjuntar un documento no llama a la IA. `FILLABLE` y `FormView.fill` ya no existen.
- El onboarding no conoce la IA: el núcleo deja `ai_ready` en `None` y el comando lo completa desde la configuración de la plataforma.
- El manual ya no habla del Asistente ni de llenar con IA.
- `cargo test`: 788 pasan. `pnpm test`: 218 pasan. `pnpm build` pasa.

**Lo que queda para el IA-1** (detalle en el PR #17):
- Borrar `core/agents/{assistant, capturist, tools}` y `core/reader/{ask, fill, upload}` completos, incluido `reader/upload.rs` entero, no solo `after_upload`, y todas sus pruebas.
- Mover `archive/vectors.rs` a la plataforma; `commands/mod.rs::index_documents` y `lib.rs` aún llaman a `index_in_background`.
- Quitar de `commands/assistant.rs` y de `lib.rs` los comandos del Asistente, y de `lib/types.ts` el tipo `Fill` junto con `AssistantOutcome`.
