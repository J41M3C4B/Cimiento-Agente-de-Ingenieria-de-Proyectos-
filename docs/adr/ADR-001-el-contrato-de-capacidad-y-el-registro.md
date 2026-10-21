---
adr: 1
estado: propuesta
fecha: 2026-10-20
tema: [ia, plataforma, contratos, arquitectura]
bloque: IA-1
tags: [adr]
---
# ADR-001 · El contrato de capacidad y el registro

**Estado:** propuesta (2026-10-20), dentro del bloque IA-1. El líder la aprueba al validar el bloque.

## Contexto

Los [planos](../planos-de-sociai.md) dicen que hay **una sola plataforma de IA** para todo el ERP: los módulos publican **contratos de capacidades**, y el **registro común** descubre lo que ofrecen (regla 2). Todo lo demás se apoya en eso:
- el enrutador propone capacidades del registro;
- la autorización se revisa sobre la capacidad ya resuelta;
- el ejecutor invoca «mediante contrato»;
- la política de datos se aplica según las clases de datos que el contrato declara.

Hoy no existe ninguna forma común de decir qué ofrece un módulo. Este ADR la fija. Se diseña desde los planos; el código actual no le da forma.

## Decisión

### 1. Tres capas nuevas en el código, y quién usa a quién

| Capa | Dónde | Qué es | Quién la puede usar |
|---|---|---|---|
| **Contrato** | `src-tauri/src/capability/` | El contrato de capacidad, la macro para declararlo y el registro. No sabe que existe la IA | Todos: los módulos para publicar, la plataforma para descubrir |
| **Plataforma** | `src-tauri/src/ai/` (y lo que se agregue de la capa 3) | La IA: proveedores, llamadas al modelo, y después el runtime, el enrutador y el ejecutor | Solo los comandos y la propia plataforma |
| **Control** | `core::access::domain` | El vocabulario de permisos (riel ERP). Hoy vive en `core`, pero es del plano transversal | Todos |

**Ningún módulo usa la plataforma.** Tampoco Mi institución (`core`), Proyectos ni los servicios deterministas (`documents`, `storage` y los demás de la base). Un módulo no llama a la IA: publica capacidades, y la plataforma las usa.

### 2. Qué declara un contrato

| Campo | Qué es | Regla |
|---|---|---|
| `id` | `módulo.nombre`, por ejemplo `institution.missing_fields` | Minúsculas, dígitos y `_`, con un solo punto. Único en el registro. El módulo es lo que va antes del punto |
| `description` | Qué hace, en español, en una o dos líneas. La lee el enrutador para proponerla y la puede leer una persona | No vacía |
| entrada y salida | Tipos de Rust. Su esquema JSON sale del tipo (crate `schemars`), así que el esquema y el código no se pueden separar | La entrada es siempre un objeto |
| `effect` | `Query` (consulta), `Transform` (transformación), `Modify` (modificación) o `Sensitive` (acción sensible, externa o irreversible) | — |
| `permission` | El permiso de las cuentas que pide (`Permission`, del riel ERP) | — |
| `data` | Las clases de datos que toca. **Provisionales:** `Public`, `Internal`, `Confidential`, `Personal` y `SensitivePersonal`. El ADR del IA-3 fija las definitivas y su política | Al menos una |
| `approval` | `Never` o `Required`: si pide la aprobación de la persona | `Sensitive` exige `Required` |
| `limits` | `timeout` (tiempo máximo), `retries` (reintentos permitidos) e `idempotent` (si repetirla da lo mismo) | `timeout` mayor que cero. Una modificación o acción sensible que no es idempotente no se reintenta: `retries = 0` |
| cómo se ejecuta | `read(f)` para consultas y transformaciones; `change(prepare, apply)` para modificaciones y acciones sensibles | La forma debe corresponder al efecto |

### 3. Ejecutar no es lo mismo para todos los efectos

Los planos dicen que una modificación **primero se prepara** y que la aprobación va **atada a sus parámetros** (§3.5). Por eso el contrato tiene dos formas, y la correcta la exige el registro:
- **`read(f)`**: una consulta o transformación. `f` recibe la entrada y devuelve la salida, sin escribir nada.
- **`change(prepare, apply)`**: una modificación o acción sensible.
  - `prepare` recibe la entrada y devuelve **qué va a cambiar**, sin escribir.
  - `apply` recibe **la misma entrada** y hace el cambio.

  La plataforma muestra lo que devuelve `prepare`, pide la aprobación cuando el contrato la exige, y solo llama a `apply` con la entrada aprobada. Si la entrada cambia, la aprobación ya no vale. Eso lo construye el IA-2; el contrato ya trae las dos piezas que necesita.

### 4. La llamada y los errores

- **La llamada (`Call`)** la arma solo la plataforma y se la pasa a la capacidad. Lleva:
  - la conexión a la base;
  - quién la pide;
  - el id de la tarea.

  Solo la plataforma la arma, con `Call::new`; los módulos solo la leen. Por eso la plataforma le puede agregar campos sin tocar el código de ningún módulo.
- **Los errores** dicen qué hacer, según la §3.2 de los planos:
  - `Invalid { field, reason }`: un argumento corregible. Se puede reintentar con la entrada corregida.
  - `NeedsPerson { what }`: falta información de la persona. La tarea se pausa y se le pide.
  - `Failed { reason }`: un error terminal.

  Sus textos son `&'static str`, así que un dato de la persona no puede terminar dentro de un error, ni de ahí en la auditoría.

### 5. Cómo se declara una capacidad

Con la macro `capability!`, en el código del módulo, junto a la función que hace el trabajo:

```rust
capability! {
    static MISSING_FIELDS = "institution.missing_fields" {
        description: "Qué datos le faltan a la institución, por ventana.",
        effect: Query,
        permission: Use,
        data: [Internal],
        approval: Never,
        limits: Limits::READ,
        run: read(missing_fields),
    }
}

fn missing_fields(call: &Call, input: MissingInput) -> Result<MissingFields, CapabilityError> { … }
```

- **La macro pide todos los campos.** Un contrato sin permiso, sin efecto o sin cualquier otro campo no compila.
- **Los tipos de entrada y salida salen de la función.** Nadie escribe un esquema a mano.
- **La capacidad queda registrada al enlazar el programa** (crate `linkme`). No hay ninguna lista: ni de capacidades ni de módulos.

**Probado en Windows (2026-10-20):** con la misma forma del crate de SociAI (`staticlib`, `cdylib` y `rlib`, más el binario), `linkme` junta las capacidades de dos módulos en desarrollo, en las pruebas (también una declarada solo en una prueba) y con el perfil de *release* real (LTO, `codegen-units = 1`, `strip`).

### 6. El registro

- **Descubre** todas las capacidades enlazadas y revisa cada contrato con las reglas de la §2: un id bien formado y único, al menos una clase de datos, la aprobación que exige una acción sensible, los reintentos y la idempotencia, una entrada que es objeto, y la forma de ejecutar que corresponde al efecto.
- **Si un contrato no cumple, no hay registro.** La app no arranca con un contrato inválido.
- **Una prueba revisa el registro real en `cargo test`.** Así, un contrato inválido se detecta antes de llegar a `main`, no al abrir la app.
- **Lo que ofrece a la plataforma:** buscar por id, listar todo, y listar por módulo y por efecto. La app lo arma al arrancar y lo guarda para toda la sesión. Filtrar por permiso, por finalidad y por política es del ejecutor y del enrutador (IA-2 al IA-4), no del registro.

### 7. La frontera, comprobada por una prueba

`architecture_tests.rs` clasifica `ai` como la capa de la plataforma y `capability` como la del contrato. La prueba falla si cualquier archivo que no sea un comando ni de la propia plataforma nombra algo de `crate::ai`. **No tiene lista de excepciones:** el código viejo que viole la regla no se acomoda, se retira o se mueve a la plataforma.

## Lo que este ADR no decide

- **Las clases de datos definitivas y su política:** el IA-3. La forma del contrato no cambia; cambian los valores.
- **El runtime, la tarea, el ejecutor, la aprobación atada a los parámetros y la entrega:** el IA-2.
- **Qué capacidades publica cada módulo:** cada bloque del riel ERP, empezando por el ERP-1.
- **Cómo se autoriza una invocación** sobre el recurso, la institución y los datos concretos: el ejecutor (IA-2), con los permisos del riel ERP.

## Consecuencias

- **Un módulo nuevo es el módulo más sus capacidades.** La plataforma no cambia, y no hay lista que actualizar.
- **La IA vieja que vive dentro de los módulos se va:**
  - el Asistente, el Capturista y la IA del Lector, en el IA-1;
  - lo que Mi institución usaba de eso, en el ERP-5;
  - las tareas de IA de Proyectos, en el ERP-4;
  - sus pantallas, en el DIS-3 y el DIS-4.

  Hasta que la plataforma las sustituya, esas funciones no están en la app.
- **La búsqueda por significado se retira; no se mueve a la plataforma.** Moverla tal como está rompería otra regla: la plataforma no lee las tablas de Mi institución, las alcanza por contratos. Además, ya nadie la usaba: el ERP-5 dejó la búsqueda del Expediente solo por palabras. Vuelve como herramienta de la plataforma, sobre las capacidades de los módulos. Las tablas de los vectores (`document_chunk_vec`, `vec_model`) se quedan en la base; qué hacer con ellas lo decide el riel ERP.
- **La lectura de convocatorias queda detenida en la plataforma** (`ai::canonical`). Por decisión del líder (2026-10-21), no se elimina: espera a conectarse a un núcleo sólido. Como llama al modelo, sale de `documents`, y lo determinista (el contrato, el paquete, la recuperación y la verificación de las citas) se queda allí. Nada del programa la llama.
- **Dos dependencias nuevas:**
  - `linkme`, para el registro sin lista;
  - `schemars`, para el esquema que sale del tipo.
- **Un contrato inválido no puede llegar a `main`:** si le falta un campo, no compila; si rompe una regla, la prueba del registro falla.

## Alternativas descartadas

- **Una lista de capacidades escrita a mano**, o una lista de módulos que cada uno llena: es la forma que los planos quieren evitar. Cada módulo nuevo cambiaría la plataforma.
- **El crate `inventory`:** registra al arrancar el programa, con constructores estáticos. `linkme` lo resuelve al enlazar, sin código que corra antes de `main`.
- **Esquemas JSON escritos a mano:** se separan del tipo de Rust con el tiempo.
- **Una sola forma de ejecutar para todos los efectos:** dejaría que una modificación se escriba sin prepararse antes, y la aprobación atada a los parámetros no tendría de dónde salir.
