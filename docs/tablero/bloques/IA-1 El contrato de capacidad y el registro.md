---
tipo: bloque
id: IA-1
carril: ia
estado: hecho
prioridad: alta
caja: Registro común de capacidades; Contratos de capacidades
pedido_por:
depende_de:
rama: ia/ia-1-contrato-de-capacidad
adr: ADR-001
creado: 2026-10-20
cerrado: 2026-10-21
---

## Qué y por qué

Hoy no hay una forma común de decir qué ofrece un módulo: cada IA tiene sus propias herramientas escritas a mano, y las pantallas llaman directo a 134 comandos. Este bloque fija el **contrato de capacidad** y el **registro** que lo descubre. Todo lo demás de los planos se apoya en esto.

El contrato declara:
- nombre y módulo;
- entrada y salida, con su esquema;
- efecto: consulta, transformación, modificación, o acción sensible o externa;
- el permiso que pide;
- las clases de datos que toca;
- si pide aprobación;
- sus límites.

## Terminado cuando

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [x] El ADR-001 describe el contrato y el registro, y el líder lo aprueba. *(Aceptado el 2026-10-21, al validar el bloque.)*
- [x] Un módulo de prueba declara capacidades, y el registro las descubre sin que nadie las escriba en una lista. *(En la rama: `the_registry_finds_what_a_module_declares_without_a_list`.)*
- [x] La clase de datos es un campo obligatorio del contrato. Las clases de IA-1 son provisionales: el ADR del IA-3 fija las definitivas y su política sin cambiar la forma del contrato. *(En la rama.)*
- [x] El registro rechaza un contrato incompleto, como uno sin permiso, sin efecto o sin clase de datos, y una prueba lo demuestra. *(En la rama: sin permiso o sin efecto no compila, y lo prueban los doctests `compile_fail`; sin clase de datos y las demás reglas, `registry_tests`.)*
- [x] Una prueba de arquitectura impide que un módulo llame a la IA por su cuenta. Es estricta y no tiene excepciones: la nueva arquitectura manda, y el código viejo que la viole no se acomoda, se retira. *(En la rama: `no_module_nor_service_uses_the_ai` pasa, sin excepciones, después del ERP-4, el ERP-5 y la retirada.)*

## Plan

**Lo que propone el ADR-001** (el líder lo aprueba al validar):

1. **Dónde vive.** El contrato y el registro van en `src-tauri/src/capability/`, que no depende de la IA. Los módulos solo usan eso. La IA (`src-tauri/src/ai/`) pasa a ser la capa de la plataforma, y ningún módulo la puede usar.
2. **Cómo se declara.** Cada capacidad es un `static` en el código de su módulo, marcado para el registro. El registro las junta al enlazar (crate `linkme`), así que no hay ninguna lista. Un módulo nuevo es el módulo más sus capacidades: la plataforma no cambia.
3. **Qué declara:**
   - un id `módulo.nombre`, el módulo y una descripción;
   - la entrada y la salida, con un esquema JSON que sale del tipo de Rust (crate `schemars`), para que el esquema y el código no se separen;
   - el efecto: consulta, transformación, modificación, o acción sensible o externa;
   - el permiso que pide, del vocabulario de permisos de las cuentas, que es del riel ERP;
   - las clases de datos que toca. Son provisionales; el IA-3 fija las definitivas;
   - si pide aprobación;
   - sus límites: tiempo, reintentos y si es idempotente;
   - la función que la ejecuta, con la firma que usará el ejecutor del IA-2.
4. **Qué rechaza:**
   - **Al compilar:** un contrato al que le falte un campo, como el permiso o el efecto, no compila. Una prueba `compile_fail` lo demuestra.
   - **Al armar el registro:**
     - sin clases de datos;
     - un id repetido o mal formado;
     - una acción sensible sin aprobación;
     - reintentos en una operación con efectos que no es idempotente;
     - una entrada que no es un objeto.

     Una prueba recorre el registro real en `cargo test`, así que un contrato inválido no llega a `main`.
5. **La frontera.** En `architecture_tests.rs`, `ai` deja de ser base y pasa a ser la capa de la plataforma. Mi institución (`core`), los módulos y Proyectos no la pueden usar, sin lista de excepciones. Los permisos (`core::access::domain`) se clasifican como capa de control, para que el contrato los pueda usar sin depender de Mi institución.

**Los pasos:**

1. El borrador del ADR-001, para que el líder lo vea pronto.
2. Probar primero que `linkme` funciona en Windows, porque es el riesgo principal.
3. `capability/`: el contrato, el registro, la validación y un módulo de prueba que declara capacidades.
4. La prueba de arquitectura estricta.
5. **Retirar la IA que vive dentro de Mi institución** (riel IA):
   - `core/agents`: el Asistente, el Capturista y sus herramientas, con sus oficios y baterías;
   - la IA del Lector: `reader/ask`, `reader/fill` y la lectura con IA de `reader/upload`.

   Los comandos que solo envuelven eso salen con ellos: `assistant_ask`, `capture_ask`, `institution_fill_from_documents` e `institution_fill_all_from_documents`, de `commands/assistant.rs`. `proposals_reject` se queda en ese archivo, porque las propuestas no se retiran en este bloque (hallazgo del DIS-3). Salen también, junto con sus líneas en `src/lib/tauri.ts` y `src/lib/types.ts`. Los vectores (la búsqueda por significado) no se borran: salen de `core` hacia la plataforma, como herramienta de la capa 4. Lo que Mi institución usa de esas piezas lo quita antes el ERP-5.
6. **Sacar la IA de los servicios deterministas.** `documents/canonical/run.rs` llama a la IA: es la lectura de convocatorias, que es un operador de IA (capa 4, riel IA). Sale de `documents`, que se queda solo con lo determinista. La prueba estricta también impide que un servicio determinista use la IA: solo la plataforma y los comandos la usan.

**Cambios al hacerlo:**
- **Los vectores se retiraron, no se movieron** a la plataforma (2026-10-20). Ya nadie los usaba: el ERP-5 dejó la búsqueda solo por palabras. Además, moverlos tal como están rompería otra regla: la plataforma alcanza a los módulos por contratos, no leyendo sus tablas. Vuelven como herramienta de la plataforma sobre las capacidades.
- **La lectura de convocatorias queda detenida en la plataforma** (`ai::canonical`), por la decisión del líder en el ERP-6 (2026-10-21): no se elimina. `documents/canonical` conserva lo determinista, incluida la recuperación (`retrieve.rs`).

Las dos decisiones están en las «Consecuencias» del ADR-001.
7. La suite completa, y el PR.

**Lo que se pide a otros rieles:**
- **ERP-4:** Proyectos sin IA propia.
- **ERP-5:** Mi institución sin la IA vieja: la parte del ERP que usa el Capturista, la lectura con IA, la configuración de la IA y los vectores.
- **DIS-3:** las pantallas sin la IA vieja.
- **DIS-4:** las pantallas de Proyectos sin la IA vieja (lo abrió el riel de diseño, porque depende de lo que retire el ERP-4).

**El orden para fusionar:**
1. DIS-3: si va después, las pantallas no compilan sin los comandos.
2. ERP-5, y ERP-4 con DIS-4 (las pantallas de Proyectos primero).
3. IA-1: su prueba estricta necesita que ya no quede IA en los módulos.

Mientras tanto, el contrato, el registro y la prueba se construyen en la rama. La prueba estricta falla y enumera lo que falta retirar.

**Lo que no entra en este bloque:**
- Que Mi institución use el escáner de datos personales (`scanner::guard`, en unos 20 archivos) no es llamar a la IA. Es política de datos repartida, y la resuelve el IA-3.
- La configuración de la IA (proveedor, clave, tope) es de la plataforma y se queda.

**Riesgos:**
- ~~`linkme` en Windows.~~ Probado el 2026-10-20 en un proyecto aparte con la misma forma de crate (`staticlib`, `cdylib` y `rlib`, más el binario). Junta las capacidades de dos módulos sin lista, en desarrollo, en pruebas y con el perfil de *release* real (LTO, `codegen-units = 1`, `strip`).
- Hasta el IA-4 y la migración de capacidades, la app queda sin el Asistente, sin la lectura con IA y sin la IA de Proyectos. Lo viejo no se conserva por existir.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/20

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-ia`, rama `ia/ia-1-contrato-de-capacidad`.
- Antes: nada; `pnpm install` y el OCR ya están. Strawberry Perl antes que el `perl` de Git en el `PATH`.
- Arrancar: `pnpm tauri dev` (la primera vez compila unos minutos).
- Al terminar: cerrar la app.

**Qué hacer y qué debe pasar:**

- [ ] La app abre y se entra normal. El registro de capacidades se arma al arrancar; si un contrato estuviera mal, no abriría.
- [ ] Mi institución → subir un archivo a un documento del Expediente (por ejemplo, la constancia fiscal de `fixtures/expediente`): llega el aviso de lo que se guardó, con las sugerencias de la lectura con reglas.
- [ ] En esa ventana, «Descartar» una sugerencia: desaparece y no vuelve.
- [ ] Documentos → buscar una palabra que esté en un documento: sale el pasaje con su página, sin internet.
- [ ] Ayuda automática → guardar o cambiar la llave: se guarda y «Comprobar» funciona. Ya no se arma el índice de vectores en segundo plano.

**Pruebas:**
- `cargo test`: 583 pasan, 15 ignoradas (las corridas reales) y 0 fallan, sobre `main` con el ERP-6.
- `pnpm test`: 224 pasan. `pnpm build` pasa.

## Hallazgos

- **La hoja de la IA (`core/ai_sheet.rs`) no llama a la IA:** solo la usa en sus pruebas. Por eso no rompe la regla del IA-1, y se quitó del Plan. Se retira cuando el gestor de contexto la sustituya. Hoy también la usan `core::api` y Proyectos (la guía exportada).
- **Las propuestas por campo (`core/agents/proposals.rs`) tampoco llaman a la IA.** Son la forma vieja de la confirmación, y se retiran cuando el IA-2 traiga la aprobación atada a los parámetros. Las usan los formularios y la lectura con reglas.
- **`main` estuvo en rojo después del DIS-4.** El ERP-4 había dejado Proyectos en `HIDDEN`, y la prueba de `core/places.rs` exigía que un módulo congelado conservara su página en `Shell.tsx`, que el DIS-4 quitó. Lo resolvió el **ERP-6** (#21), sacando Proyectos del programa.
- **La retirada dejó código sin uso en Mi institución y en los módulos** (unos 100 avisos del compilador). Es código que solo servía a la IA vieja:
  - casi toda la hoja de la IA (`core/ai_sheet.rs`);
  - `core/reader/quote.rs`;
  - lo del Asistente en `core/places.rs`, `core/records/mod.rs` y `core/institution/forms.rs`;
  - `ai_summary` de Beneficiarios y `lines` de Finanzas.

  También hay comentarios que todavía hablan del Asistente (`core/modules.rs`, `core/places.rs`). Es del riel ERP. No lo toqué, para no hacer trabajo de otro riel; el líder decide si va en un bloque aparte.
- **La plataforma (`ai/`) conserva lo de las tareas viejas de Proyectos:** sus tareas en `AiTask` (conversación, diagnóstico, prioridades, redacción y el resumen de la convocatoria), sus oficios, `figures.rs`, y `dry_run_tests.rs`, que el ERP-4 apagó. Propongo que el IA-2 empiece por ahí:
  - dejar `ai/` con lo que se conserva según `codigo-frente-a-los-planos.md` (proveedores, configuración, la bóveda y el ritmo);
  - conservar la lectura de convocatorias detenida, con lo que usa (`CallCanonical`, su oficio y `golden`), como decidió el líder en el ERP-6;
  - portar las pruebas del proveedor a un impulsor propio.

## Resultado

Fusionado a `main` con *squash* en el PR #20 (`11b1c85`), validado por el líder el 2026-10-21. El ADR-001 queda aceptado.

**Qué quedó:**
- **El contrato de capacidad, la macro `capability!` y el registro sin lista** (`src-tauri/src/capability/`, con `linkme` y `schemars`). La app arma el registro al arrancar, y no abre si un contrato está mal.
- **Qué se rechaza:**
  - al compilar, un contrato al que le falta un campo, lo que prueban 3 doctests `compile_fail`;
  - al armar el registro, las demás reglas: 12 pruebas, que también recorren el registro real.
- **La frontera de la IA:** `no_module_nor_service_uses_the_ai`, sin lista de excepciones. Ningún módulo, ni Mi institución ni un servicio determinista, usa la IA.
- **Lo que salió:**
  - el Asistente, el Capturista y sus herramientas;
  - la IA del Lector;
  - los 4 comandos que los envolvían, con sus permisos y sus líneas en el front;
  - la búsqueda por significado.
- **La lectura de convocatorias quedó detenida** en la plataforma (`ai::canonical`), por la decisión del líder en el ERP-6.
- **Pruebas:**
  - `cargo test`: 583 pasan, 0 fallan y 15 se ignoran (las corridas reales);
  - `pnpm test`: 224 pasan; `pnpm build` pasa.

**Lo que sigue:** los hallazgos de arriba. El código sin uso de Mi institución lo purga el ERP con el diseño del core, y el IA-2 empieza limpiando `ai/`.
