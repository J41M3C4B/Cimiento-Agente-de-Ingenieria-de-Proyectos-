---
tipo: bloque
id: DIS-4
carril: diseno
estado: backlog
prioridad: alta
caja: Interfaces de los módulos
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Parte del [[DIS-3 Las pantallas sin la IA vieja|DIS-3]], que cubre Mi institución y el Asistente. El IA-1 exige que **ningún módulo llame a la IA por su cuenta**, y el ERP-4 retira de Proyectos sus tareas de IA. Las pantallas de Proyectos están hechas alrededor de ellas, así que hay que decidir cómo se ven **sin** esas llamadas antes de que los comandos se retiren.

Hoy las pantallas de Proyectos llaman a los comandos que el ERP-4 retira. **La lista cerrada está en el «Plan» del ERP-4** («Contrato para el DIS-4»): conversación y resumen, `needs_propose`, `drafting_prepare`, `section_draft`, `sections_draft_all`, las convocatorias (`project_create_from_call`, `call_*`) y `project_job`.

Se necesita que **las pantallas de Proyectos dejen de usarlos**, y que lo que queda se vea completo y usable a mano (escribir, editar y confirmar). Lo que hacían se vuelve a construir como capacidades de la plataforma, con el chat y las confirmaciones del DIS-1 y el DIS-2; no se conserva.

## Terminado cuando

Propuesta del riel de diseño; la aprueba el líder. Decisión de fondo (2026-10-20): lo que dependa de la IA **se quita, no se sustituye a mano**; lo que queda se ve completo y se usa sin IA.

- [ ] **Ninguna pantalla de Proyectos usa los comandos retirados** del contrato del ERP-4 (ni sus `invoke` por nombre). Una prueba en `src/` lo demuestra, como la del DIS-3.
- [ ] **Un proyecto se recorre a mano de punta a punta, sin chat, sin conversación y sin convocatoria:** crearlo, elegir su necesidad, escribir y confirmar sus textos, armar presupuesto y calendario, revisarlo y exportar la guía en Word.
- [ ] **Los pasos que ve la persona son cuatro** (los que quedan en el contrato): Priorización, Redacción, Revisión y Lista. El indicador de pasos, las tarjetas de proyecto y Inicio dicen lo mismo.
- [ ] **Un proyecto nuevo** se crea con lo que el contrato permite: lo que la institución quiere lograr (`initial_request`) y quién da el apoyo. No pide archivos, nombre de convocatoria, quién convoca ni año.
- [ ] **Priorización a mano:** la persona ve sus necesidades, agrega una (título y descripción, con la revisión de datos personales de siempre) y elige la del proyecto. No hay propuesta automática ni calificación a mano: no hace falta para avanzar.
- [ ] **Redacción a mano:** el objetivo y tres partes (presupuesto, calendario y textos) con su estado, que se abren en su ventana como hoy; sin «redactar con IA», sin «guía corta» y sin preparar el borrador. Lo que ya se escribió o confirmó antes se ve igual, también un texto antiguo marcado como borrador de la IA.
- [ ] **Sin convocatoria en pantalla:** no hay panel de convocatoria ni lectura, y las tarjetas de proyecto y de Inicio no muestran quién convoca ni cuándo cierra; muestran el tipo de apoyo.
- [ ] **Un proyecto que todavía está en una etapa que desaparece** (si llega a la pantalla antes de la migración del ERP-4) se muestra en Priorización y avisa con claridad si algo no puede seguir; no se rompe ni queda en blanco.
- [ ] **Sin textos huérfanos ni promesas falsas:** se retira de `es-MX.ts` y de `src/` lo que solo usaba la IA de Proyectos, y la bienvenida y el arranque dejan de decir que «el asistente» hará preguntas o borradores.
- [ ] **Pruebas:** se retiran las de la conversación, el diagnóstico y las convocatorias; las de lo que se queda pasan; hay una prueba nueva por cada pantalla que cambia. `pnpm test` y `pnpm build` pasan.
- [ ] **Revisado en la app real (WebView2) con el Rust del ERP-4**, en claro y oscuro: recorrer un proyecto de punta a punta. Con el Rust actual un proyecto nuevo nace en «Diagnosis» y no puede avanzar sin conversación, así que no se puede probar completo sin el ERP-4.
- [ ] **Se fusiona antes que el ERP-4 y se valida junto con él** (ver «Riesgos»).

## Plan

**Rama:** `diseno/dis-4-proyectos-sin-la-ia-vieja`, de `main` al día, en la carpeta `sociai-diseno`. Solo `src/` y este bloque. `lib/tauri.ts` y `lib/types.ts` no se tocan: las líneas de los comandos retirados y `Stage` los cambia el ERP-4 después (contrato del ERP-4).

**El diseño: una sola columna, sin chat.** Hoy cada paso es una conversación (`ConversationChat`) con un panel al lado (`Workspace`, con la convocatoria como índice). Sin IA no hay conversación ni convocatoria, y el panel queda sin función. Cada paso pasa a ser la pantalla de lo que se hace en él, dentro de la carpeta del proyecto (el color y el título de siempre), con el indicador de pasos arriba y las acciones («Continuar», «Volver») al pie. Todo con los componentes que ya existen (`Folder`, `Steps`, `Card`, `Inset`, `Tag`, `Bar`, `Modal`, `ListRow`); no se inventa ninguno.

**Qué cambia en cada pantalla** (lo visto en el código):

1. **Página del proyecto** (`ProjectPage.tsx`, `Workspace.tsx`, `steps.ts`): deja de leer `conversation_get`; el proyecto sale de `project_list` y los datos de cada paso, de su propio comando (`needs_get`, `drafting_get`, `review_get`). Sale el panel de la convocatoria y su botón. Los pasos son `PRIORITIZATION`, `DRAFTING`, `REVIEW`, `READY`. «Volver» ofrece solo pasos anteriores que existen.
2. **Nuevo proyecto** (`NewProjectForm.tsx`): de dos pasos con archivos a **una pantalla**: «¿Qué quiere lograr?» (texto), tipo de apoyo y la revisión de datos personales. Llama a `project_create` y luego a `project_set_donor_kind`.
3. **Priorización** (`ObjectivesStage.tsx`): lista de necesidades con «Elegir», «Agregar una necesidad» y, al pie, la elegida y «Continuar». Sin propuesta ni «pensando…».
4. **Redacción** (`DraftingStage.tsx`, `ProjectIndex.tsx`, `TextsEditor.tsx`, `SectionCard.tsx`): el objetivo y las tres partes con su estado como contenido principal; cada una abre su ventana (`BudgetEditor`, `ScheduleEditor`, `TextsEditor`). Salen: preparar, la pregunta de propuesta propia como burbuja del chat (queda como interruptor dentro de «Textos»), las tres tarjetas de cómo redactar, «Redactar lo pendiente», «Guía otra vez» y el botón de IA de cada sección.
5. **Revisión y Lista** (`ReviewStage.tsx`, `ReadyStage.tsx`): el resultado y la guía pasan a ser el contenido principal, con sus botones al pie.
6. **Tarjetas** (`ProjectFolder.tsx`, `projectCall.ts`, `core/home/Ongoing.tsx`): sin quién convoca ni fecha de cierre; el tipo de apoyo ocupa su lugar. Los textos «lo que sigue» (`es.home.nextByStage`) se ajustan a cuatro pasos.
7. **Se borran:** `ConversationChat`, `ChatParts`, `DiagnosisPanel`, `CallConfirm`, `CallReading`, `CallCardView`, `CallIndex`, `useProjectJob` y sus pruebas.
8. **Textos** (`es-MX.ts`): sale lo de la conversación, el diagnóstico, las convocatorias y los avisos de IA que ya nadie use; se reescribe la bienvenida y el arranque (`onboarding`, `help`) sin «el asistente».

**Los pasos:**
1. La rama y la lista exacta de importaciones y llaves a retirar (búsqueda en `src/`).
2. Página del proyecto y pasos sin chat (1, 3, 5), con pruebas.
3. Nuevo proyecto (2) y Redacción (4), con pruebas.
4. Tarjetas e Inicio (6); borrar lo huérfano (7); textos (8).
5. Prueba de arquitectura de `src/` con los comandos de Proyectos (la misma del DIS-3, ampliada).
6. `pnpm test`, `pnpm build`, revisión en WebView2 real con el Rust del ERP-4, `rebase` sobre `main`, finales de línea y PR.

**Orden para fusionar:** DIS-4 primero, luego ERP-4, y al final IA-1.

**Lo que no entra aquí:**
- `lib/tauri.ts` y `lib/types.ts` (ERP-4).
- Calificar las necesidades a mano (`need_rate`): el comando se queda, pero no hace falta para avanzar. Se agrega si el líder lo pide.
- Un chat o una ayuda con IA dentro de Proyectos: lo dibujan DIS-1 y DIS-2 sobre las capacidades nuevas.
- El nombre de un proyecto nuevo y cambiarlo después (ver «Hallazgos»).

**Riesgos:**
- **Mientras DIS-4 esté en `main` sin el ERP-4, un proyecto nuevo nace en «Diagnosis» y no puede avanzar** (su avance pide la conversación que ya no se ve). Por eso DIS-4 y ERP-4 se validan y se fusionan seguidos, el mismo día.
- La revisión en la app real necesita el Rust del ERP-4. Hasta que su rama exista, se prueba lo que sí se puede (Priorización con un proyecto ya creado, redacción, revisión) y el resto con el ERP-4.
- Es un bloque grande (unos 25 archivos de `src/` y sus pruebas). Si el líder prefiere, se parte en dos PR seguidos: pasos y página; luego nuevo proyecto y tarjetas.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

- **El título de un proyecto nuevo es el comienzo de lo que se escribió** (`project_create` corta `initial_request` a 80 caracteres) y no hay comando para cambiarlo. Antes lo ponía el nombre de la convocatoria. Para el ERP: un `project_rename`, o un título aparte en `project_create`; el líder decide si va en el ERP-4 o aparte.
- **Para el ERP-4:** su contrato debe decir en qué etapa nace un proyecto con `project_create` (aquí se supone `Prioritization`, pues `Diagnosis` desaparece) y que `Profile` sigue siendo un estado previo.
- **El IA-1** debería depender de este bloque, además del ERP-4 y el ERP-5.
- **`docs/manual/`** habla del diagnóstico y de la convocatoria; es del riel ERP.

## Resultado
