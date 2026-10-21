---
tipo: bloque
id: ERP-9
carril: erp
estado: en-curso
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

- [ ] **Un cambio del módulo de muestra se prepara** sin escribir nada y devuelve una propuesta: las diferencias campo por campo (antes y después), lo que no pasa la validación, la versión base y la huella.
- [ ] **Aplicar revisa la versión y la huella:** si el dato o la entrada cambiaron desde que se preparó, no se aplica y pide preparar de nuevo. Una prueba lo demuestra.
- [ ] **Al aplicar, una sola transacción** escribe el dato, su historial con su origen (persona, documento y página, cálculo o tarea con su id), su hecho en la bandeja y el registro del cambio. Si algo falla, no queda nada. Una prueba lo demuestra.
- [ ] **El historial sirve para cualquier entidad de cualquier módulo,** solo crece, y lo protegido dice que cambió sin guardar el valor.
- [ ] **El riel IA conoce y acepta** la forma de la propuesta y de la huella, anotada en el IA-2.
- [ ] **K5 dice si una huella ya se aplicó** (punto 4 del ERP-10), para no repetir a ciegas un cambio incierto.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto.

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

**El acuerdo con el riel IA (lo que necesita confirmar):**
- La salida de `prepare` de una capacidad de cambio es la `Proposal` del núcleo, con su `fingerprint`.
- El ejecutor ata la aprobación a esa `fingerprint`. Su huella propia, la de la capacidad y la entrada, le sigue sirviendo para reconocer un paso repetido.
- Al aplicar, el ejecutor pasa la huella aprobada a `apply`. Se propone un campo nuevo en `Call` (`fingerprint: Option<&str>`), que se agrega con el IA-5.
  - El núcleo vuelve a calcular la huella con la versión de hoy. Si no coincide, responde «prepara de nuevo», y el ejecutor pide otra aprobación.
  - Un cambio sin aprobación se aplica sin huella previa.
- Si un `apply` no contestó, antes de repetirlo el ejecutor pregunta `kernel::applied(huella)`.
- El origen de un cambio hecho por la plataforma es «tarea T, aceptada por la cuenta C».

**Lo que toca de otros rieles:** nada. `Cargo.toml` mueve `sha2`, `hex` y `ulid` a las dependencias del *workspace*, sin cambiar sus versiones, para que el núcleo use las mismas.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

## Resultado
