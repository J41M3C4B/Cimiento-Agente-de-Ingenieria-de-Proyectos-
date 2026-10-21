---
tipo: bloque
id: IA-2
carril: ia
estado: revision
prioridad: alta
caja: Runtime; Ejecutor; Estado, límites y ciclo de vida; Entrega de resultados
pedido_por:
depende_de: IA-1
rama: ia/ia-2-esqueleto-de-la-plataforma
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

La columna vertebral del plano operativo, **todavía sin IA**:
- el runtime que recibe toda petición;
- la tarea con su estado, sus límites y su id de correlación;
- un solo ejecutor que aplica los controles antes y después de cada invocación;
- la entrega común de resultados.

Se arma primero sin modelo para probar el camino con un flujo determinista, sin gastar nada. Así el ciclo sin fin que hoy tiene el agente queda imposible por diseño.

**Acuerdos con el riel ERP (2026-10-21):**
1. **La aprobación.** El IA-2 ata la aprobación a la entrada desde ahora, como dice el ADR-001. La versión del dato y la huella del [[ERP-9 Preparar y aplicar, con historial y origen|ERP-9]] se agregan después. Su formato lo acuerdan el riel IA y el riel ERP antes de cerrar el ERP-9; está escrito en su tarjeta.
2. **Los archivos de compilación.** El ERP-8 cambia `src-tauri/Cargo.toml` y `Cargo.lock` para armar el *workspace*. Cuando el ERP-8 entre a `main`, el IA-2 hace `rebase` antes de su PR.
3. **`Permission`.** Se muda al núcleo, pero `core::access::domain` lo sigue exponiendo, así que `capability/` y `ai/` no cambian por el ERP-8. Lo que el IA-5 pide aparte son los permisos que declara cada módulo.

**Propuesta para empezar:** primero, limpiar `ai/`. Se conserva lo que dice `codigo-frente-a-los-planos.md` (proveedores, configuración, la bóveda y el ritmo) y la lectura de convocatorias detenida, con lo que usa. Se retiran las tareas viejas de Proyectos, y las pruebas del proveedor apagadas (`dry_run_tests.rs`) se portan a un impulsor propio. Después, el esqueleto.

## Terminado cuando

(Aprobado por el líder el 2026-10-21. Ajustado ese día con [el plano de la IA](../../plano-de-la-ia.md): piezas P1, P2, P7, P8, P9 y P12, y decisiones A3 y A4.)

- [ ] Una petición de pantalla que nombra una capacidad recorre runtime → tarea → ejecutor → entrega, como flujo determinista, sin enrutador ni modelo y con todos los controles. El resultado llega en el contrato común de resultados, con el id de la tarea.
- [ ] Toda tarea registra su cierre: terminada, pausada, denegada, por límite o con error, con su motivo y sin datos personales.
- [ ] Los límites de pasos, reintentos, gasto y tiempo cortan una tarea que no avanza, y la misma capacidad con la misma entrada dos veces la cierra por límite. Una prueba lo demuestra.
- [ ] Una modificación se prepara, espera aprobación atada a sus parámetros, y se vuelve a pedir si los parámetros cambian. La huella del ERP-9 se agrega cuando exista.
- [ ] Las tareas se guardan en la base: una tarea pausada o esperando aprobación sigue ahí al volver a abrir la app, y se reanuda.

## Plan

La rama sale de `main` con el ERP-8 ya fusionado, así que no hace falta `rebase`. Todo lo nuevo va en `src-tauri/src/ai/runtime/`, sin depender del código viejo de `ai/`, para que pasarlo al *crate* de la plataforma (A1, IA-5) sea mover archivos.

1. **Las tablas de la plataforma** (A3), con `kernel::migrate_owner(conn, "ai", …)`:
   - `ai_task`: la tarea, su estado, la razón de su pausa o de su cierre, y sus contadores;
   - `ai_step`: cada invocación, con su capacidad, su fase y la huella de su entrada;
   - `ai_approval`: la aprobación pendiente, atada a la huella de la capacidad y su entrada;
   - `ai_event`: la auditoría de la tarea (P12), solo ids y códigos fijos.

   Las migra la app al abrir la base. El almacén de auditoría del núcleo (K9) llega con el ERP-10; mientras, la plataforma escribe en su propia tabla.
2. **La petición y el contrato común de resultados** (P1, P9):
   - **la petición:** canal, tarea por reanudar y contenido (una capacidad con su entrada, un texto, una respuesta o una decisión);
   - **el resultado:** respuesta, datos, aclaración, propuesta, estado y error, cada uno con el id de la tarea, el paso y la hora.

   Su tipo va en `src/lib/types.ts`, para el DIS-1.
3. **La tarea y sus límites** (P2), por tipo de tarea (pantalla, chat o evento):
   - pasos;
   - correcciones;
   - gasto con reserva;
   - tiempo de cada turno;
   - plazo de una pausa;
   - la misma capacidad con la misma entrada dos veces.
4. **El ejecutor** (P7). Antes de cada invocación revisa la tarea, los límites, la repetición, el permiso (por una pregunta de autorización que hoy contesta la sesión y mañana K2) y el esquema de la entrada. Después revisa el esquema de la salida. Una consulta entrega los datos; un cambio prepara y, si el contrato lo pide, espera aprobación. Un `apply` que no terminó queda «incierto» y nunca se repite.
5. **La aprobación** (P8): atada a la huella de la capacidad y su entrada. Si la entrada cambia, la aprobación anterior queda vieja y se pide otra vez.
6. **La entrada desde la app:** dos comandos, `runtime_submit` y `runtime_task` (consultar una tarea por su id). La entrega también se avisa a la ventana como evento.
7. **Las pruebas,** con capacidades de prueba y sin modelo:
   - un recorrido completo, que se puede leer en la terminal;
   - cada cierre;
   - cada límite;
   - la aprobación y su cambio;
   - reabrir la base con una tarea en pausa.

**Lo que no entra aquí:**
- **La política de datos:** IA-3, en los cuatro puntos fijos que deja el ejecutor.
- **Retirar las tareas viejas de Proyectos y traer las pruebas del proveedor apagadas:** van con el IA-4, cuando el modelo entre al ejecutor. Las pruebas del pipeline usan esas tareas como ejemplo, y el pipeline se rehace ahí; hacerlo ahora sería hacerlo dos veces.

## Cómo probarlo

PR: https://github.com/J41M3C4B/SociAI/pull/25

**Pruebas:** `cargo test`: 635 pasan y 0 fallan; `tsc --noEmit`: sin errores; `pnpm test`: 224 pasan.

**Cómo correrlo:**
- **Carpeta y rama:** `D:\Agente Proyectos\sociai-ia`, rama `ia/ia-2-esqueleto-de-la-plataforma`.
- **Antes:**
  - compilar desde **PowerShell**, no desde Git Bash. La primera compilación tarda más, porque entran los *crates* del ERP-8; SQLCipher no se vuelve a compilar.
  - Al abrirse, la base gana las tablas `ai_` de la plataforma. No toca nada de lo que ya tiene.
- **Arrancar:**
  1. Las pruebas, donde se ve el recorrido completo (todavía ninguna pantalla usa el runtime; eso es del DIS-1):
     ```
     cd src-tauri
     cargo test --lib ai::runtime -- --nocapture --test-threads=1
     ```
  2. La app: `pnpm tauri dev`, desde `D:\Agente Proyectos\sociai-ia`.
- **Al terminar:** cerrar la app. No queda nada que limpiar.

**Qué hacer y qué debe pasar:**

- [ ] **Las 24 pruebas del runtime pasan.** La del recorrido (`a_change_from_a_screen_walks_the_whole_path`) cuenta en español lo que pasa:
  - la propuesta;
  - la aprobación;
  - el cambio, con el id de su tarea;
  - las siete líneas que dejó en la auditoría.
- [ ] **La app abre igual que antes,** y se entra con la cuenta de siempre.
- [ ] **La puerta contesta desde la app real.** Con la sesión abierta, abre las herramientas de desarrollo (clic derecho → «Inspeccionar», pestaña «Console») y pega:
  ```js
  await window.__TAURI_INTERNALS__.invoke("runtime_submit", { request: { channel: { kind: "chat" }, content: { kind: "text", text: "¿Qué me falta?" } } })
  ```
  Debe regresar:
  - un `task` que empieza con `task_`;
  - `state: "error"`;
  - `body: { kind: "error", code: "without_ai", next: "use_screen" }`.

  El chat todavía no tiene modelo, y lo dice con un camino para seguir.
- [ ] **Con ese id, la tarea se consulta después:**
  ```js
  await window.__TAURI_INTERNALS__.invoke("runtime_task", { task: "task_…" })
  ```
  Regresa el mismo resultado.
- [ ] **Una capacidad que no existe se cierra con su motivo:**
  ```js
  await window.__TAURI_INTERNALS__.invoke("runtime_submit", { request: { channel: { kind: "screen", place: "inicio" }, content: { kind: "invoke", capability: "nadie.nada", input: {} } } })
  ```
  Regresa `code: "unknown_capability"`.

## Hallazgos

- **El tiempo de cada capacidad (`limits.timeout`) todavía no se corta.**
  - Por qué: una capacidad corre con la conexión de la base y no se puede interrumpir a la mitad. Para cortarla hay que correrla aparte, con su propia conexión.
  - Lo que hay por ahora: el tiempo se revisa por turno, antes de cada invocación.
  - Cuándo se resuelve: cuando el ejecutor llame al modelo (IA-4), que sí es asíncrono, o antes si una capacidad lenta lo pide.
- **Encontrado en la revisión propia y corregido aquí:** si un `apply` ya aprobado pedía algo a la persona, su respuesta se aplicaba con la aprobación anterior. Ahora una pausa nunca se reanuda en `apply`: lo que la persona agrega se prepara y se aprueba otra vez. Hay una prueba para eso.
- **Lo que espera de otros bloques, ya con su lugar:**
  - la pregunta de autorización la contesta la sesión hasta que K2 la tenga (ERP-10);
  - la auditoría de las tareas vive en `ai_event` hasta que K9 reciba el evento de tarea (ERP-10);
  - la organización es «local» hasta que exista K1;
  - la huella todavía no lleva la versión del dato (ERP-9).

## Resultado
