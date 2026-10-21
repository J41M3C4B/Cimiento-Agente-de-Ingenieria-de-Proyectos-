---
tipo: bloque
id: ERP-9
carril: erp
estado: revision
prioridad: alta
caja: Unidad de trabajo (K4); Historial y origen (K5); Eventos (K8, la bandeja de hechos)
pedido_por:
depende_de: ERP-8
rama: erp/erp-9-preparar-y-aplicar
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El [plano del ERP](../../plano-del-erp.md) (§7) dice que todo cambio de datos se **prepara** y luego se **aplica**, venga de una persona o de la plataforma. Es el punto donde la IA entra de forma nativa sin que el ERP la conozca, y el IA-2 ata la aprobación a lo que este bloque entrega. Hoy no hay propuesta con diferencias, ni versión de cada registro, ni huella. Además, el historial y el origen solo existen para los campos de Mi institución (`core_field`, `core_change`), y cada tabla repite sus columnas de origen. Este bloque construye en el núcleo la propuesta, la huella, la versión y la unidad de trabajo, y el historial genérico.

**Lo fija el ERP y lo usa el IA-2:** la forma de la propuesta y de la huella. Este bloque va primero o a la par, y el formato se acuerda con el riel IA antes de cerrarlo.

**Del [[ERP-10 Lo que pide el plano de la IA al núcleo|ERP-10]] (punto 4):** que K5 sepa si una huella ya se aplicó, para que la plataforma no repita a ciegas un `apply` que no contestó.

## Terminado cuando

Aprobado por el líder el 2026-10-21. Las casillas se marcan cuando cada punto queda hecho.

- [x] **Un cambio del módulo de muestra se prepara** sin escribir nada y devuelve una propuesta: las diferencias campo por campo (antes y después), lo que no pasa la validación, la versión base y la huella.
- [x] **Aplicar revisa la versión y la huella:** si el dato o la entrada cambiaron desde que se preparó, no se aplica y pide preparar de nuevo. Una prueba lo demuestra.
- [x] **Al aplicar, una sola transacción** escribe el dato, su historial con su origen (persona, documento y página, cálculo o tarea con su id), su hecho en la bandeja y el registro del cambio. Si algo falla, no queda nada. Una prueba lo demuestra.
- [x] **El historial sirve para cualquier entidad de cualquier módulo,** solo crece, y lo protegido dice que cambió sin guardar el valor.
- [ ] **El riel IA conoce y acepta** la forma de la propuesta y de la huella, anotada en el IA-2.
- [x] **K5 dice si una huella ya se aplicó** (punto 4 del ERP-10), para no repetir a ciegas un cambio incierto.
- [x] `cargo test` y `pnpm test` pasan, sin red ni gasto.

## Plan

Todo va en el núcleo (`crates/kernel`). El módulo de muestra lo usa de punta a punta, como funciones de Rust: publicarlo como capacidad desde su *crate* es del IA-5.

1. **La referencia y la huella.**
   - `Ref { module, entity, id }`: cualquier entidad de cualquier módulo.
   - `kernel::fingerprint(&Ref, version, &input)`: SHA-256 en hexadecimal de `kernel.v1`, la referencia, la versión base y la entrada en JSON canónico (llaves ordenadas).
   - Una entidad nueva lleva su id desde la entrada: lo genera quien la pide (`kernel::new_id`). Así, preparar y aplicar ven el mismo id, y repetir la misma creación no crea dos.
2. **La propuesta.** `kernel::propose(...)` arma una `Proposal` y no escribe nada. La propuesta lleva:
   - la referencia;
   - la versión base (0 si es nueva);
   - las diferencias campo por campo (antes, después y si el campo es protegido);
   - lo que no pasa la validación (campo y código);
   - la huella.
3. **Aplicar: la unidad de trabajo (K4).** `kernel::apply(conn, Change, current_version, write)`, en una sola transacción:
   1. Si la huella ya se aplicó, no hace nada y lo dice.
   2. Lee la versión de hoy y vuelve a calcular la huella con la misma entrada. Si no coincide (cambió el dato o la entrada), no aplica: «prepara de nuevo».
   3. Si la validación no pasa, tampoco aplica.
   4. El módulo escribe su dato y sube la versión. El núcleo revisa que quedó en la versión base más uno.
   5. El núcleo escribe, en la misma transacción:
      - el registro del cambio (`kernel_change`, con la huella única);
      - el historial con su origen (`kernel_history`);
      - el hecho en la bandeja (`kernel_outbox`, solo ids y un código).
   
   Si algo falla, no queda nada.
4. **El historial (K5)** sirve para cualquier entidad. Lleva su origen:
   - una persona (su cuenta);
   - un documento, con su página;
   - un cálculo, con el código de su regla;
   - una tarea de la plataforma, con su id y la cuenta que aceptó.
   
   **Solo crece:** unos *triggers* impiden cambiarlo o borrarlo. **Lo protegido** dice que cambió, sin guardar el valor. `kernel::history(conn, &Ref)` lo devuelve.
5. **Saber si una huella se aplicó** (punto 4 del ERP-10): `kernel::applied(conn, huella)` dice el cambio, la entidad y la versión, o nada.
6. **El módulo de muestra.** Su nota gana versión, texto y un teléfono protegido (migración 2). `muestra::service` tiene `prepare` y `apply` sobre el núcleo, con una validación sencilla, porque el diccionario llega con el ERP-2. Su entrada (`NoteInput`) va en su `contract`.
7. **La suite completa y el PR.**

**El acuerdo con el riel IA: la forma de la propuesta y de la huella.** El riel IA la revisa y la acepta antes de que se cierre este bloque. Los puntos 3 y 4 son cambios del contrato de capacidad, y los agrega el riel IA cuando acepte.

1. **El esquema de la propuesta.** La salida de `prepare` de una capacidad de cambio es la `kernel::Proposal`, en JSON:

   ```json
   {
     "entity": { "module": "muestra", "entity": "note", "id": "nota_01J…" },
     "base_version": 3,
     "changes": [
       { "field": "title", "before": "Viejo", "after": "Nuevo", "protected": false },
       { "field": "phone", "before": "55…", "after": "55…", "protected": true }
     ],
     "invalid": [ { "field": "title", "reason": "required" } ],
     "fingerprint": "9f2c…(64 hex)"
   }
   ```

   - `base_version` es 0 cuando la entidad es nueva.
   - `changes` lleva solo los campos que cambian, ordenados por nombre.
   - `invalid` usa códigos fijos (`required`, `too_long`…). Si no viene vacío, `apply` no aplica.
   - Un campo `protected` trae sus valores para la persona que aprueba; la política decide si llegan al modelo. En el historial solo queda que cambió.
2. **La huella es fija.** Es `kernel::fingerprint(&entity, base_version, &input)`: SHA-256 en hexadecimal (64 caracteres) del texto `kernel.v1\n{module}.{entity}#{id}@{base_version}\n`, seguido de la entrada en JSON canónico, con las llaves ordenadas en todos los niveles y sin espacios.
   - La calcula siempre el núcleo. La plataforma no la calcula; la toma de `proposal.fingerprint`.
   - Una entidad nueva lleva su id en la entrada (`kernel::new_id`), para que preparar y aplicar vean la misma entidad.
   - El prefijo `kernel.v1` permite cambiar la forma algún día sin confundir huellas viejas con nuevas.
3. **El paso de la huella en la `Call`** (cambio del contrato).
   - `Call` gana `fingerprint: Option<&str>`.
   - Al aplicar después de una aprobación, el ejecutor pone ahí `proposal.fingerprint`, la que aprobó la persona.
   - Un cambio sin aprobación (`approval: Never`) va con `None`, y el núcleo lo aplica como se prepara en ese momento.
   - La aprobación de la plataforma se ata a esa huella. La huella propia del ejecutor, la de la capacidad y la entrada, le sigue sirviendo para reconocer un paso repetido.
4. **El error de desincronización** (cambio del contrato). Si el dato o la entrada cambiaron después de aprobar, `apply` responde con el código `prepare_again` (`kernel::ApplyError::Stale`).
   - Se propone que el contrato lo dé como un `CapabilityError` propio, por ejemplo `Outdated`.
   - Con él, el ejecutor vuelve a preparar y pide otra aprobación, que es lo que ya hace con `Again::Outdated`.
   - Los otros códigos de `apply`: `already_applied`, `invalid`, `nothing_to_change`, `bad_origin`, `bad_fact`, `version_not_raised` y `database`.
5. **La consulta a K5.**
   - Antes de repetir un `apply` que no contestó (el paso quedó «started»), el ejecutor pregunta `kernel::applied(conn, huella)`. Si contesta `Some(Applied { change, entity, version })`, el cambio ya está hecho y no se repite.
   - Además, `apply` con una huella ya aplicada no hace nada y responde `already_applied`, con el mismo `Applied`.
6. **El origen viaja por la `Call`.** El `apply` de la capacidad arma el origen con lo que trae la `Call`:
   - si la invoca una tarea: `Origin::Task { task: call.task, account: call.by }`, donde `call.by` es la cuenta que aprobó o pidió;
   - si viene de una pantalla, por el flujo determinista: el mismo origen de tarea, porque toda invocación pasa por el runtime;
   - lo que viene de un documento: `Origin::Document { document, page, account }`.
   
   Los ids deben tener forma de id (prefijo y ULID); si no, el núcleo responde `bad_origin`. Por eso, el id de la tarea y el de la cuenta deben seguir esa forma.

**Lo que toca de otros rieles:** nada. `Cargo.toml` mueve `sha2`, `hex` y `ulid` a las dependencias del *workspace*, sin cambiar sus versiones, para que el núcleo use las mismas.

## Cómo probarlo

PR [#27](https://github.com/J41M3C4B/SociAI/pull/27).

**Cómo correrlo:**
- Carpeta y rama: `D:\Agente Proyectos\sociai-erp`, rama `erp/erp-9-preparar-y-aplicar`.
- Antes: cerrar la app del otro riel si está abierta. No hace falta `pnpm install`. La base gana tres tablas vacías del núcleo (`kernel_change`, `kernel_history`, `kernel_outbox`). El código de antes las ignora.
- Arrancar: `pnpm tauri dev`. Recompila unos minutos.
- Al terminar: cerrar la app. No queda nada que limpiar.

**Qué hacer y qué debe pasar:**

- [ ] La app abre con la base de siempre y todo funciona como antes. Este bloque no cambia nada visible.
- [ ] El riel IA confirma la forma de la propuesta y de la huella escrita en el plan.

## Hallazgos

- **Alguien marcó todas las casillas del «Terminado cuando» en la copia de esta tarjeta en la carpeta `sociai-erp`.** Las tarjetas se escriben en `sociai`, así que no lo subí. Lo guardé como parche (`erp9-casillas-ajenas.patch` en el *scratchpad* del agente ERP) y regresé el archivo a como estaba en git.
- **El historial nuevo convive con el de Mi institución** (`core_field` y `core_change`). Este se retira cuando Mi institución pase a la forma de módulo (ERP-1).
- **La bandeja de hechos (`kernel_outbox`) solo se llena.** Entregar los hechos y reaccionar a ellos es del ERP-3, con el actor del sistema.
- **La validación de la muestra es suya por ahora** (título obligatorio, teléfono con forma de teléfono). Con el ERP-2, el diccionario la hace para cualquier entidad.

## Resultado
