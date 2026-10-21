---
tipo: bloque
id: ERP-4
carril: erp
estado: en-curso
prioridad: alta
caja: Proyectos
pedido_por: IA-1
depende_de:
rama: erp/erp-4-proyectos-sin-ia-propia
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones, y la nueva arquitectura manda sobre el código viejo.

Hoy Proyectos tiene sus propias tareas de IA (conversación, diagnóstico, prioridades, redacción y lectura de convocatorias), y por eso usa `crate::ai` en su código y en sus comandos. Se necesita que **Proyectos no use la IA de ninguna forma**, para que la prueba del IA-1 pase. Lo que hacían esas tareas se vuelve a construir después como capacidades de la plataforma, no se conserva.

## Terminado cuando

Aprobado por el líder el 2026-10-20. Decisión: lo que dependa de la IA se **quita**, no se sustituye a mano. Las casillas se marcan cuando cada punto queda hecho.

- [ ] **Proyectos no usa la IA de ninguna forma.** Ni `modules/projects` ni sus comandos importan `crate::ai` para trabajar con un modelo, y la prueba estricta del IA-1 deja de listar archivos de Proyectos.
- [ ] **El flujo queda en lo que no usa IA.** Las etapas son `Profile → Prioritization → Drafting → Review → Ready`. `CallSelection` y `Diagnosis` desaparecen: sus condiciones (`root_cause_confirmed`, `diagnosis_summary_confirmed` y `call_confirmed`) las producía la IA. En «Prioritization» la persona agrega, califica y elige a mano la necesidad del proyecto.
- [ ] **Un proyecto que estaba en una etapa que desaparece no se pierde.** Una migración al final de `migrations.rs` lo pasa a `Prioritization`. Los proyectos de las demás etapas no cambian, y ningún dato se borra: las conversaciones, los resúmenes y las lecturas de convocatoria se quedan en la base, sin uso, hasta que el líder decida.
- [ ] **La redacción funciona sin convocatoria:** con requisitos vacíos, el plan de secciones es el de siempre por defecto. Escribir y confirmar cada sección, el presupuesto, el calendario, la revisión y la guía en Word siguen funcionando a mano. Una prueba recorre un proyecto de punta a punta sin ningún modelo.
- [ ] **Los comandos retirados (lista de abajo) dejan de existir** en Rust (`lib.rs`, `commands/`, la lista de permisos de `core::access::domain`) y en `lib/tauri.ts` y `lib/types.ts`. Los que se quedan conservan nombre, permiso y firma.
- [ ] **Los comandos de configuración de la IA** (`ai_status`, `ai_set_provider`, `ai_models`, `ai_set_models`, `ai_set_key`, `ai_clear_key`, `ai_check`, `ai_usage_report`, `ai_set_cap`) **no cambian**: son de la plataforma (IA-1). Salen de `commands/diagnosis.rs` hacia su propio archivo, sin tocar su código.
- [ ] **Pruebas:** se retiran las de la IA de Proyectos (conversación, diagnóstico, redacción, lectura de convocatorias); las de lo que se queda pasan; `test_support.rs` y lo de `ai/dry_run_tests.rs` que llama a funciones retiradas se ajusta en este mismo PR, avisando al riel IA. `cargo test` y `pnpm test` pasan sin red ni gasto.
- [ ] **Se fusiona después del DIS-4 y antes del IA-1.** Con los comandos aún en Rust, las pantallas del DIS-4 compilan y funcionan sin llamarlos.

## Plan

**Quién va primero.** El que fija el contrato: este bloque. Lo primero que sale es el contrato de abajo (comandos y etapas), para que el DIS-4 pueda escribir su «Terminado cuando» y construir las pantallas. Mientras tanto este bloque construye lo de Rust. Se fusiona en el orden DIS-4, luego ERP-4, y al final IA-1.

### Contrato para el DIS-4

**Comandos que se retiran** (las pantallas deben dejar de usarlos):

| Grupo | Comandos |
|---|---|
| Conversación y resumen | `conversation_get`, `conversation_start`, `conversation_send`, `conversation_retry`, `diagnosis_summary_generate`, `diagnosis_summary_edit`, `diagnosis_summary_confirm` |
| Necesidades con IA | `needs_propose` |
| Redacción con IA | `drafting_prepare`, `section_draft`, `sections_draft_all` |
| Convocatorias | `project_create_from_call`, `call_brief_make`, `call_reading_get`, `call_reading_confirm`, `call_reading_retry` |
| Trabajos en segundo plano de la IA | `project_job` |

**Comandos que se quedan, sin cambios:** `project_list`, `project_create`, `project_delete`, `project_set_color`, `project_set_donor_kind`, `project_advance`, `project_go_back`, `needs_get`, `need_add`, `need_rate`, `need_select`, `drafting_get`, `drafting_set_asks`, `sections_confirm_all`, `section_save`, `section_confirm`, `budget_save_item`, `budget_delete_item`, `budget_confirm`, `schedule_save_activity`, `schedule_delete_activity`, `schedule_confirm`, `review_get`, `guide_export`.

**Lo que cambia de forma** (el DIS-4 lo ve en `types.ts`):
- `Stage` pierde `call_selection` y `diagnosis`.
- `ProjectRow.call_reading_id` sigue existiendo y vendrá siempre vacío para un proyecto nuevo.
- `DraftingView.requirements` vendrá vacío cuando no hay convocatoria, y `DraftingView.plan_ready` deja de significar nada (nadie prepara el borrador).
- El estado de cada sección ya no será `draft_ai` para texto nuevo; pueden quedar secciones antiguas con ese estado y se muestran como cualquier borrador.
- `AiStatus`, los avisos de «la ayuda automática no está disponible» y el seguimiento de trabajos (`project_job`) dejan de aparecer en las respuestas de Proyectos.

**Qué debe poder hacer la persona sin IA:** crear un proyecto, elegir su necesidad, escribir y confirmar cada sección, armar presupuesto y calendario, revisar, y exportar la guía en Word.

### Los pasos
1. Este contrato en el tablero, para el DIS-4 (hecho al escribirlo).
2. La rama `erp/erp-4-proyectos-sin-ia-propia`, de `main` al día.
3. Domain: quitar `CallSelection` y `Diagnosis` de `Stage` y de `StageFacts`; ajustar `advance`, `go_back` y los códigos de error.
4. Migración: pasar a `Prioritization` los proyectos de las etapas que salen.
5. Retirar el código de IA de `modules/projects` (`conversation`, la parte con IA de `calls`, `diagnosis` y `drafting`, y `jobs`) y los comandos de la lista.
6. Mover los comandos `ai_*` a su archivo; ajustar `lib.rs`, la lista de permisos, `lib/tauri.ts` y `lib/types.ts`.
7. Una prueba de punta a punta sin modelo, y ajustar `test_support.rs` y `ai/dry_run_tests.rs`.
8. `cargo test`, `pnpm test`, finales de línea, `rebase` sobre `main`, PR con «Cómo probarlo», y el bloque a «Revisión».

**Riesgos:**
- Un proyecto con datos de IA ya guardados: no se borra nada, se deja de mostrar.
- `lib/types.ts` lo toca también el DIS-4: este bloque solo quita lo retirado, después de que el DIS-4 se fusione.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

**Decisión del líder (2026-10-20): camino B.** Proyectos no avanza hasta que el núcleo esté listo. Lo que dependa de la IA **se quita, no se sustituye a mano**: las etapas cuya condición de avance la produce la IA (`root_cause_confirmed` y `diagnosis_summary_confirmed` de la conversación y el resumen, y `call_confirmed` de la lectura de la convocatoria) se retiran hasta que existan las capacidades nuevas. El «Terminado cuando» se escribe cuando el ERP-5 termine (a lo más un bloque en curso por riel), con el líder.

**Opción 1 (líder, 2026-10-20):** se quitan las etapas de diagnóstico y de selección de convocatoria; el flujo queda en lo que no usa IA. La lista cerrada de comandos está en el «Plan» (contrato para el DIS-4).

**Para el IA-1 y el riel IA:** `ai/dry_run_tests.rs` y `test_support.rs` llaman a funciones de Proyectos que este bloque retira (`start_conversation`, `send_message`, `generate_summary`, `draft_section`…). Hay que acordar quién ajusta esas pruebas.

## Resultado
