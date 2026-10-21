---
tipo: bloque
id: DIS-4
carril: diseno
estado: en-curso
prioridad: alta
caja: Interfaces de los módulos
pedido_por: IA-1
depende_de:
rama: diseno/dis-4-proyectos-sin-la-ia-vieja
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Parte del [[DIS-3 Las pantallas sin la IA vieja|DIS-3]], que cubre Mi institución y el Asistente. El IA-1 exige que **ningún módulo llame a la IA por su cuenta**, y las pantallas de Proyectos están hechas alrededor de la IA: cada paso es una conversación con un panel de convocatoria al lado.

**Decisión del líder (2026-10-20): Proyectos se congela por completo y queda fuera hasta que el núcleo esté listo.** Se rediseñará cuando exista Mi institución completa, las capacidades de la plataforma de IA y el chat (DIS-1), porque hoy no se puede decir cuál es su función real. Con el ERP, el líder congela el código y la lógica de Rust (ERP-4); este bloque saca Proyectos de **lo que ve la persona**.

**El pedido [[DIS-5 Inicio sin Proyectos congelado|DIS-5]] (del ERP-4) queda cubierto aquí:** que Inicio no ofrezca ni pregunte nada de Proyectos mientras esté congelado. Y hay una razón más para quitar las pantallas y no solo ocultarlas: el ERP-4 retira de `lib/tauri.ts` las llamadas de los comandos de IA de Proyectos, y las pantallas que quedaran en `src/` importan esas llamadas, así que `pnpm build` fallaría.

Se necesita que **ninguna pantalla ofrezca, abra ni llame a Proyectos**, y que la app quede completa y coherente sin él: solo el núcleo (Inicio, Mi institución y Documentos) y lo demás que ya está congelado.

## Terminado cuando

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [ ] **Proyectos no aparece en ninguna parte de la app:**
  - no hay pestaña «Mis proyectos» en la barra;
  - Inicio no tiene el botón «Empezar un proyecto nuevo» ni la carpeta larga «En curso / Notificaciones» (su «En curso» solo mostraba proyectos);
  - ningún «Ir a…», paso de arranque, dibujo de módulos o texto de ayuda lleva a Proyectos o lo promete.
- [ ] **Ninguna pantalla llama a un comando de Proyectos:** ni los de IA del contrato del ERP-4 ni el resto (`project_*`, `need_*`, `needs_get`, `drafting_*`, `section*`, `budget_*`, `schedule_*`, `review_get`, `guide_export`, `conversation_*`, `diagnosis_*`, `call_*`). Una prueba en `src/` lo demuestra, como la del DIS-3.
- [ ] **El código de las pantallas sale de `src/`** (`src/modules/projects/` y lo que solo ellas usaban, como `Ongoing`), y se puede recuperar: una etiqueta de git, `proyectos-congelado`, apunta al último `main` que las tiene.
- [ ] **La app queda completa sin él:** Inicio, Mi institución, Documentos, la configuración y los módulos ya congelados se ven y funcionan; nada queda en blanco, roto ni con un hueco que pregunte por Proyectos.
- [ ] **Sin promesas falsas:** la bienvenida, el arranque, la ayuda y los textos de `es-MX.ts` dejan de decir que SociAI «prepara proyectos» o que «el asistente hará preguntas o borradores». Los textos nuevos se proponen en el PR y los valida el líder.
- [ ] **Sin textos ni componentes huérfanos** de lo retirado (`conversation`, `diagnosis`, `calls`, `drafting`, `review`, `objectives`, `home.ongoing`, `home.nextByStage` y lo que solo ellos usaban).
- [ ] **Pruebas:** se retiran las de Proyectos; las de lo que se queda pasan y se ajustan las que contaban con la pestaña o con las tarjetas de proyecto; `pnpm test` y `pnpm build` pasan.
- [ ] **Revisado en la app real (WebView2),** en claro y oscuro: Inicio, la barra, Mi institución y la bienvenida de una cuenta nueva.
- [ ] **Se fusiona antes que el congelamiento del ERP:** con los comandos aún en Rust, la app compila y funciona sin llamarlos.

## Plan

**Rama:** `diseno/dis-4-proyectos-sin-la-ia-vieja`, de `main` al día, en la carpeta `sociai-diseno`. Solo `src/` y este bloque. `lib/tauri.ts` y `lib/types.ts` no se tocan: las líneas y los tipos de los comandos de Proyectos los quita el riel ERP al congelarlo. Esto aprovecha la regla que ya usó el DIS-3.

**Qué se retira y qué cambia** (visto en el código):

1. **La barra y el mapa de páginas** (`components/Shell.tsx`, `modules.ts`, `App.tsx`): sale «Mis proyectos» de `MODULES`, `projects` de `Page` y de `ModuleId`, `ProjectsPage` y los atajos de `App` (`openProject`, `newProject`, `onGoProjects`). Los tipos `Exclude<ModuleId, "projects">` pasan a ser `ModuleId`.
2. **Inicio** (`core/home/HomePage.tsx`): sale el botón de nuevo proyecto, la consulta `projectList` y la carpeta `Ongoing`. Queda lo demás: aviso de datos pendientes, cifras, «Por hacer», tarjeta de Mi institución y «Siguientes pasos». Se borra `Ongoing` y su prueba.
3. **Se borra `src/modules/projects/`** completo, con sus pruebas (unos 40 archivos): la página, el formulario, los pasos, la conversación, la convocatoria, la redacción, la revisión y la guía.
4. **Bienvenida y arranque** (`core/onboarding`, `es.onboarding`, `help`): se quitan los pasos y los dibujos que hablan de Proyectos o del asistente, y se proponen textos sin esas promesas. Lo que hoy dice «prepara los proyectos de su institución para pedir donativos» (`app.tagline`, `welcome`) lo reescribe el líder o lo valida.
5. **Textos** (`i18n/es-MX.ts`): sale lo de Proyectos y lo que solo ellos usaban. Los avisos de IA que usen la configuración de la IA o Mi institución se quedan.
6. **Prueba de arquitectura de `src/`** (`lib/retiredAi.test.ts`, la del DIS-3): se amplía con los comandos de Proyectos y con `modules/projects`.
7. **La etiqueta de git** `proyectos-congelado`, en el `main` que aún tiene las pantallas, antes de fusionar.

**Los pasos:**
1. La rama y la lista exacta de importaciones, llaves y pruebas a retirar (búsqueda en `src/`).
2. Barra, `App` y los tipos (1); Inicio (2); borrar `src/modules/projects/` (3).
3. Bienvenida, arranque y textos (4 y 5), con pruebas.
4. La prueba de arquitectura (6).
5. `pnpm test`, `pnpm build`, revisión en WebView2 real, `rebase` sobre `main`, finales de línea y PR.
6. Al fusionar: la etiqueta (7).

**Orden para fusionar:** DIS-4 primero. Después el congelamiento del ERP (ERP-4) y al final el IA-1.

**Lo que no entra aquí:**
- Rust, `lib/tauri.ts` y `lib/types.ts`: el riel ERP.
- Rediseñar Proyectos: queda para cuando el núcleo esté listo.
- Qué se pone en Inicio en lugar de «En curso»: no se inventa nada. Si el líder quiere otra cosa ahí, se pide aparte.

**Riesgos:**
- **Inicio pierde su carpeta larga.** Hoy su única función era mostrar proyectos y notificaciones aún vacías. El líder debe verlo así al validar.
- **Los textos de la bienvenida y el arranque son el rostro del producto.** Cambiar «prepara proyectos» cambia lo que SociAI dice que es; por eso los valida el líder.
- **Los datos de proyectos que ya existan** quedan en la base sin verse; los trata el ERP.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

- **Para el ERP-4:** la decisión del líder cambia su alcance de «retirar la IA de Proyectos» a «congelar Proyectos». El contrato de comandos que dejó para el DIS-4 ya no se usa: este bloque no necesita ninguno de Proyectos.
- **Para el ERP-4: no puede fusionarse antes que este bloque.** Su «Terminado cuando» dice «sin esperar al DIS-4», pero quita de `lib/tauri.ts` las llamadas de IA de Proyectos y las pantallas de `src/modules/projects/` aún las importan: `pnpm build` y `pnpm test` fallarían. El orden es este bloque, luego el ERP-4, luego el IA-1.
- **El IA-1** debería depender de este bloque, además del ERP-4 y el ERP-5.
- **`docs/manual/`** (riel ERP) habla de Proyectos, del diagnóstico y de la convocatoria; y `core/places.rs` compone los lugares desde el manual. Sale con el congelamiento.

## Resultado
