---
tipo: bloque
id: ERP-4
carril: erp
estado: hecho
prioridad: alta
caja: Proyectos
pedido_por: IA-1
depende_de:
rama: erp/erp-4-proyectos-sin-ia-propia
adr:
creado: 2026-10-20
cerrado: 2026-10-20
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones, y la nueva arquitectura manda sobre el código viejo.

Hoy Proyectos tiene sus propias tareas de IA (conversación, diagnóstico, prioridades, redacción y lectura de convocatorias), y por eso usa `crate::ai` en su código y en sus comandos. Se necesita que **Proyectos no use la IA de ninguna forma**, para que la prueba del IA-1 pase. Lo que hacían esas tareas se vuelve a construir después como capacidades de la plataforma, no se conserva.

**Decisión del líder (2026-10-20): Proyectos se congela por completo**, como Personal, Beneficiarios, Instalaciones y Finanzas. No se avanza en él hasta que el núcleo (Mi institución y Documentos, con la plataforma) esté listo: lo que se hiciera ahora se rehace después. Por eso este bloque es **mínimo**: ocultar el módulo y quitar solo lo que llama al modelo. No rediseña etapas, ni datos, ni pantallas.

## Terminado cuando

Aprobado por el líder el 2026-10-20 (alcance mínimo por el congelamiento). Las casillas se marcan cuando cada punto queda hecho.

- [x] **Proyectos está congelado como los demás módulos.** `"projects"` entra en `HIDDEN` (`core/modules.rs`): la barra, el mapa de lugares y el manual no lo muestran, y nada del núcleo lo lee para mostrarlo. Su código sigue compilando y probándose.
- [x] **Proyectos no usa la IA de ninguna forma.** Ningún archivo de `modules/projects` importa `crate::ai`, y la prueba estricta del IA-1 deja de listarlos.
- [x] **Solo se quita lo que llama al modelo:** la lectura de convocatorias, la conversación, el resumen, las necesidades con IA, la redacción con IA (`draft_section`, `prepare_plan`, `draft_all`) y el seguimiento de esos trabajos (`jobs`). Los comandos que los arrancan salen también, con su permiso en la lista de `core::access::domain`.
- [x] **No se toca lo demás:** ni el modelo de etapas, ni el almacenamiento, ni las migraciones, ni los datos que ya haya en la base.
- [x] **Los comandos `ai_*` (configuración de la IA) no cambian:** salen de `commands/diagnosis.rs` a su propio archivo, `commands/ai.rs`, sin tocar su comportamiento.
- [x] **Pruebas:** *(`cargo test --lib`: 722 pasan; `pnpm test`: 218 pasan; `pnpm build` pasa. Con la rama del IA-1 fusionada, su prueba estricta ya no lista nada de Proyectos.)* se retiran las de lo que sale; las de lo que se queda pasan. `test_support.rs` y lo de `ai/dry_run_tests.rs` que llama a funciones retiradas se ajusta en este mismo PR, avisando al riel IA. `cargo test` y `pnpm test` pasan sin red ni gasto.
- [x] **El DIS-4 ya no hace falta:** con Proyectos oculto, sus pantallas no se ven. *(Las líneas de los comandos retirados en `lib/tauri.ts` y `lib/types.ts` **no** se quitaron, a propósito: las pantallas de Proyectos, ocultas y del riel de diseño, aún las importan y `pnpm build` fallaría. Se quitan cuando Proyectos se rehaga. Inicio sí sigue ofreciendo Proyectos: pedido DIS-5.)*
- [x] **Se fusiona antes del IA-1,** sin esperar al DIS-4.

## Plan

**Rama:** `erp/erp-4-proyectos-sin-ia-propia`, rehecha desde `main` limpio. Un primer intento más amplio (nuevo modelo de etapas, migración, flujo a mano) se descartó por el congelamiento; queda guardado solo en local, en `wip/erp-4-stage-model-without-ai`, por si sirve de referencia.

**Los pasos:**
1. Ocultar Proyectos y ajustar las pruebas que lo daban por visible.
2. Quitar el código de IA de `modules/projects`, archivo por archivo, dejando lo determinista (la vista de la conversación, las necesidades, el presupuesto, el calendario, la revisión y la guía).
3. Quitar los comandos que arrancan IA y sus permisos; mover los `ai_*` a `commands/ai.rs`.
4. Ajustar las pruebas y los ayudantes (`test_support.rs`, `ai/dry_run_tests.rs`).
5. Quitar de `lib/tauri.ts` y `lib/types.ts` lo retirado.
6. `cargo test`, `pnpm test`, finales de línea, `rebase` sobre `main`, PR con «Cómo probarlo», y el bloque a «Revisión».

**Riesgos:**
- Las condiciones de avance de algunas etapas (causa raíz y resumen confirmados, convocatoria confirmada) quedan sin forma de cumplirse. Es aceptable: el módulo está congelado, y esas etapas se rediseñan cuando exista la plataforma.
- `ai/dry_run_tests.rs` (riel IA) usa la conversación y el resumen como excusa para probar al proveedor. Se apaga ese módulo de pruebas, no se borra; el riel IA lo porta a una prueba que no dependa de Proyectos.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/18

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-erp`, rama `erp/erp-4-proyectos-sin-ia-propia`.
- Antes: nada (ya está instalado y el OCR copiado).
- Arrancar: `pnpm tauri dev`.
- Al terminar: cerrar la app.

**Qué hacer y qué debe pasar:**

- [x] En la barra superior ya no aparece «Proyectos».
- [x] La pantalla de la ayuda automática sigue igual: proveedor, llave, tope de gasto y «Comprobar».
- [x] En *Ayuda*, buscar «proyecto convocatoria»: no sale la pantalla de Proyectos.
- [x] Inicio todavía muestra «Nuevo proyecto» (es el DIS-5); pulsarlo no debe romper la app, aunque Proyectos no se vea. *(No lo he probado en la app real: solo las pruebas automáticas.)*

## Hallazgos

- **Inicio sigue ofreciendo Proyectos** («Nuevo proyecto», proyectos en curso, el paso de proyectos) y llama a `project_list`. Es del riel de diseño: pedido **DIS-5**, en Backlog.
- **`ai/dry_run_tests.rs` está apagado, no borrado** (`#[cfg(any())]` en `ai/mod.rs`): ~30 pruebas del proveedor y de la tubería (cuota, respaldo de modelos, caché, reintentos) que usaban la conversación de Proyectos como excusa. El riel IA debe portarlas a un impulsor propio.
- **`call_reading_get` y `call_reading_confirm` salen** aunque no arrancan IA: sin la lectura no tienen sentido.
- Las condiciones de avance de las etapas de diagnóstico y de convocatoria quedan sin forma de cumplirse. Es aceptable con el módulo congelado.

- **Para el IA-1 y el riel IA:** este bloque ya no depende del DIS-4 (el IA-1 lo tiene en `depende_de`). `ai/dry_run_tests.rs` y `test_support.rs` llaman a funciones de Proyectos que este bloque retira (`start_conversation`, `send_message`, `generate_summary`, `draft_section`…); se ajustan aquí.
- **El contrato de comandos que se le dio al DIS-4 (2026-10-20) deja de valer** por el congelamiento.

## Resultado

Fusionado a `main` con *squash* en el PR #18 (`3b56cd0`), probado por el líder el 2026-10-20 en la app.

**Qué quedó:**
- Proyectos está congelado como los demás módulos (`"projects"` en `HIDDEN`): no aparece en la barra, en el mapa de lugares ni en el manual.
- `modules/projects` ya no usa la IA. Salieron la lectura de convocatorias, los turnos de la conversación, el resumen, las necesidades con IA, la redacción con IA y `jobs.rs`, con 14 comandos y sus permisos. Etapas, almacenamiento, migraciones y datos no se tocaron.
- Los comandos `ai_*` de configuración viven ahora en `commands/ai.rs`, sin cambios (con `AiStatus`).
- `cargo test --lib`: 722 pasan. `pnpm test`: 218 pasan. `pnpm build` pasa. Con la rama del IA-1 fusionada, su prueba estricta ya no lista nada de Proyectos.

**Lo que queda para otros rieles:**
- **IA-1:** su prueba solo lista ya lo que él mismo retira (`core::agents`, `core::reader`, `core::archive::vectors`, `documents::canonical::run`). Su tarjeta aún tiene a DIS-4 y ERP-4 en `depende_de`.
- **IA:** `ai/dry_run_tests.rs` está apagado (`#[cfg(any())]`), no borrado: ~30 pruebas del proveedor y la tubería que hay que portar a un impulsor que no dependa de Proyectos.
- **Diseño:** DIS-5 (Inicio sin Proyectos), en Backlog. Las líneas de los comandos retirados siguen en `lib/tauri.ts` y `lib/types.ts` porque las pantallas ocultas de Proyectos las importan; se quitan cuando Proyectos se rehaga.
