---
tipo: bloque
id: DIS-3
carril: diseno
estado: hecho
prioridad: alta
caja: Chat global; Interfaces de los módulos
pedido_por: IA-1
depende_de:
rama: diseno/dis-3-pantallas-sin-la-ia-vieja
adr:
creado: 2026-10-20
cerrado: 2026-10-20
---

## Qué y por qué

El IA-1 retira la IA vieja que vive dentro de Mi institución, porque ningún módulo puede llamar a la IA por su cuenta. Con ella desaparecen estos comandos:
- `assistant_ask`: el Asistente;
- `capture_ask`: el Capturista;
- `institution_fill_from_documents` e `institution_fill_all_from_documents`: llenar desde documentos;
- `proposals_reject`: las propuestas por campo;
- la lectura con IA que sigue a adjuntar un documento: el evento `record-suggested`;
- en la vista de un formulario, el dato `fill` (qué ventana llena el Capturista), que el ERP-5 quita.

Hoy los usan `src/core/assistant/Assistant.tsx`, `src/core/profile/FormWindow.tsx` y `src/core/profile/ProfilePage.tsx`, con sus pruebas. También desaparecen los comandos de IA de Proyectos que retire el ERP-4.

Se necesita que **las pantallas dejen de usarlos** antes de que se retiren. Lo que hacían lo sustituyen el chat global (DIS-1) y las capacidades nuevas; no se conserva.

**Alcance acordado con el líder (2026-10-20):**
- Este bloque cubre **Mi institución, el Asistente y la lectura al adjuntar**, que es la lista de arriba.
- Las pantallas de **Proyectos** pasan a [[DIS-4 Proyectos sin la IA vieja|DIS-4]], porque el ERP-4 todavía no dice qué comandos retira.
- La revisión de sugerencias por campo («Usar», «Descartar») **se conserva** hasta el DIS-2: la lectura con reglas, que no usa IA, sigue dejándolas al adjuntar un documento. Por eso `proposals_reject` no debe retirarse (ver «Hallazgos»).

## Terminado cuando

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [x] **Ninguna pantalla usa la IA vieja de Mi institución.** En `src/`, fuera de `lib/tauri.ts` y `lib/types.ts` (que cambia quien retira el comando), nada llama a `assistant_ask`, `capture_ask`, `institution_fill_from_documents` ni `institution_fill_all_from_documents`, nada escucha `record-suggested` y nada lee el dato `fill` de la vista de un formulario. Una búsqueda en `src/` lo demuestra.
- [x] **La app se usa sin el Asistente.** No hay botón ✨ en la barra, ni panel, ni «Preguntar más». La ayuda de cada campo sigue mostrando el manual, sin internet y sin gasto.
- [x] **Las ventanas de Mi institución y su página no ofrecen llenar con IA:** ni «Llenarlo por mí», ni «Llenar con mis documentos», ni «Llenar todo con mis documentos».
- [x] **Lo que no usa IA sigue como está:**
  - revisar sugerencias por campo («Usar», «Descartar», «Descartar todas») y el aviso «Hay N sugerencias» de Mi institución;
  - los avisos al adjuntar un documento (lo que se llenó y las sugerencias de la lectura con reglas);
  - «Ir a…» desde Inicio y la cuarentena de datos personales al guardar.
- [x] **No quedan textos ni componentes huérfanos:** lo que solo usaba el Asistente o la lectura con IA sale de `es-MX.ts` y de `src/`.
- [x] **Pruebas:** se retiran las del Asistente y de la lectura con IA; las de lo que se queda pasan; hay una prueba nueva por cada punto que cambia (barra, ayuda de campo, ventana, página). `pnpm test` y `pnpm build` pasan.
- [x] **Revisado en la app real (WebView2),** en tema claro y oscuro: Inicio, una ventana de Mi institución, adjuntar un archivo al Expediente y la ayuda de un campo.
- [x] **Se fusiona antes que el ERP-4, el ERP-5 y el IA-1.** Con los comandos aún en Rust, la app compila y funciona sin llamarlos.

## Plan

**Rama:** `diseno/dis-3-pantallas-sin-la-ia-vieja`, de `main` al día, en la carpeta `sociai-diseno`. Solo se toca `src/` y este bloque; nada de Rust, de `lib/tauri.ts`, de `lib/types.ts` ni de `docs/manual/`.

**Qué se retira y qué cambia** (lo que hoy usa los comandos, visto en el código):

1. **El Asistente** (`src/core/assistant/`): se borran `Assistant.tsx` y `Assistant.test.tsx`. No se conserva ningún cascarón: el DIS-1 dibuja el chat desde cero sobre el contrato común de resultados del IA-2.
   - `App.tsx`: sale `AssistantProvider`. `goTo` se queda, porque Inicio lo usa.
   - `components/Shell.tsx`: sale el botón ✨ y `useAssistant`.
   - `components/ManualHelp.tsx`: sale «Preguntar más»; el manual de cada campo sigue.
2. **`core/profile/FormWindow.tsx`:** salen `useAssistant`, `windowClosed`, los botones «Llenarlo por mí» y «Llenar con mis documentos», su estado (`searching`, `fromDocs`) y el bloque `form.data?.fill`. Se quedan «Usar», «Descartar» y «Descartar todas».
3. **`core/profile/ProfilePage.tsx`:** sale «Llenar todo con mis documentos» y su estado. Se quedan el aviso de sugerencias y «Revisarlas». Se borra `core/profile/fromDocuments.ts`.
4. **`core/profile/recordReading.tsx`:** sale la escucha de `record-suggested` y `suggestedText`; se borra `recordSuggested.test.ts`. Se queda `record-read` y los avisos de «lo que se llenó» y «sugerencias» al adjuntar (`attachedNotice`), que salen de la lectura con reglas.
5. **`i18n/es-MX.ts`:** sale el bloque `assistant` entero, `manual.askMore`, y de `suggestions` lo de llenar con documentos (`fillFromDocuments`, `fillAllFromDocuments`, `searchingDocuments`, `fromDocuments`, `fromDocumentsAll`, `nothingFromDocuments`); de `profile.records`, `suggestedFromFile` y `suggestFailed`. Antes de borrar cada llave se comprueba que nadie más la use.
6. **Pruebas:** `FormWindow.test.tsx` pierde el grupo «Llenar con mis documentos» y conserva el de las sugerencias; `ProfilePage.test.tsx` y `RecordWindow.test.tsx` se ajustan; se agregan las pruebas nuevas de la barra (sin ✨), la ayuda de campo (sin «Preguntar más»), la ventana (sin botones de llenar) y la página (sin «Llenar todo»).

**Los pasos:**
1. La rama, y la lista exacta de importaciones y llaves a retirar (búsqueda en `src/`).
2. Retirar el Asistente y lo que lo abre (1).
3. Retirar los botones y la lectura con IA (2, 3 y 4).
4. Limpiar textos (5) y pruebas (6).
5. `pnpm test`, `pnpm build` y la búsqueda del primer punto del «Terminado cuando».
6. Revisar en WebView2 real (CDP, con una carpeta de datos aparte), en claro y en oscuro.
7. `rebase` sobre `main`, revisar finales de línea, PR con «Cómo probarlo», y el bloque a «Revisión».

**Orden para fusionar:** DIS-3 primero (lo pide el IA-1). Después ERP-4 y ERP-5, y al final IA-1.

**Lo que no entra aquí:**
- Las pantallas de Proyectos: DIS-4.
- `lib/tauri.ts` y `lib/types.ts`: las líneas de los comandos retirados y sus tipos (`AssistantOutcome`, `CaptureOutcome`, `FillOutcome`, `RecordSuggested`, `Fill`, el campo `fill`) los quita el IA-1 y el ERP-5 con el comando. Se quedan sin uso, y TypeScript no se queja.
- La interfaz de revisión de sugerencias: la rediseña el DIS-2.
- El texto «asistente» de la bienvenida y del arranque (`onboarding`, `help`): habla del asistente de Proyectos, y va en DIS-4 o DIS-1.

**Riesgos:**
- Hasta el DIS-1 y el IA-4, la app no tiene chat. El líder ya lo aceptó (ver IA-1, «Riesgos»).
- Sugerencias que ya estén guardadas en la base con origen de la IA (`ai_assumption`) siguen apareciendo en la ventana con la revisión de siempre; es lo correcto, y las retira el ERP con las propuestas.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-diseno`, rama `diseno/dis-3-pantallas-sin-la-ia-vieja` (PR #16).
- Antes: nada nuevo (`pnpm install` ya está hecho). Strawberry Perl antes que el `perl` de Git en el `PATH`. El `target` de esa carpeta ya está compilado, así que arranca en segundos.
- Arrancar: `pnpm tauri dev` desde esa carpeta. Si tu propio `pnpm tauri dev` usa el puerto 1430, ciérralo antes o dime y lo corro aparte.
- Al terminar: cerrar la app. Cambiar de rama en esa carpeta solo después de validar.

**Qué hacer y qué debe pasar:**

- [x] Entrar, ir a Inicio: la barra tiene tema, configuración y tu cuenta; **no hay botón ✨**.
- [x] Mi institución → «Editar» de una tarjeta: la ventana abre **sin** «Llenarlo por mí» ni «Llenar con mis documentos».
- [x] En esa ventana, pulsar el «?» de un campo: se despliega el manual y **no** aparece «Preguntar más».
- [x] En Mi institución, sección del Expediente con un documento que ya tiene archivo: **no** aparece «Llenar todo con mis documentos».
- [x] Subir un archivo a un documento (por ejemplo la constancia de situación fiscal de `fixtures/expediente`): cierra la ventana y llega el aviso «Se guardó el archivo…»; si la lectura con reglas dejó sugerencias, en su ventana siguen «Usar», «Descartar» y «Descartar todas».
- [x] Repetir en tema oscuro; nada roto ni en blanco.

## Hallazgos

- **`proposals_reject` no debería retirarse con el Asistente** (para el IA-1 y el ERP-5). El bloque lo lista entre los comandos que desaparecen, pero vive en `commands/assistant.rs`, que el IA-1 borra entero. Las propuestas por campo no llaman a la IA (hallazgo del IA-1) y la lectura con reglas del Lector sigue produciéndolas al adjuntar un documento. Sin ese comando no hay forma de descartarlas. Se necesita que el comando se mueva a un archivo que se quede (el de Mi institución o el de Expediente) antes de que el IA-1 borre `commands/assistant.rs`. Es del riel ERP; el líder decide si va en el ERP-5 o en un bloque aparte.
- **`docs/manual/` habla del Asistente** (`programa.md`, `README.md`) y se compila dentro de la app. Es del riel ERP: queda para cuando el chat nuevo exista, o antes si el líder prefiere.
- **El IA-1 debería depender también del DIS-4** (hoy depende del DIS-3, el ERP-4 y el ERP-5). Lo ajusta su riel.

## Resultado

Fusionado a `main` con squash (PR #16, `52ece77`), validado por el líder el 2026-10-20.

- Las pantallas ya no usan `assistant_ask`, `capture_ask`, `institution_fill_from_documents` ni `institution_fill_all_from_documents`, ni escuchan `record-suggested`. Una prueba (`src/lib/retiredAi.test.ts`) falla si algo de `src/` vuelve a nombrarlos.
- No hay Asistente: sin botón ✨, sin panel y sin «Preguntar más». La ayuda de cada campo sigue mostrando el manual.
- Las ventanas y la página de Mi institución ya no ofrecen llenar con IA.
- Se conservó la revisión de sugerencias por campo hasta el DIS-2.
- Revisado en WebView2 real, en claro y oscuro. `cargo test` no se corrió: no se tocó Rust.

**Para los otros rieles:**
- **ERP-5 y ERP-4** ya pueden retirar sus comandos: ninguna pantalla los llama. Hacen `rebase` sobre `main` antes de fusionar.
- **IA-1:** `lib/tauri.ts` y `lib/types.ts` conservan los wrappers y tipos de los comandos retirados (`AssistantOutcome`, `CaptureOutcome`, `FillOutcome`, `RecordSuggested`, `Fill`, el campo `fill`). Quien retira cada comando los quita.
- **`proposals_reject` debe quedarse** (ver «Hallazgos»).
