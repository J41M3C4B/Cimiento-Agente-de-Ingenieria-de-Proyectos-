# ADR-028 · Perfiles de acceso: cuentas, permisos por comando y borrados que esperan al administrador

**Estado:** aceptada (2026-10-07). Retira el PIN de pantalla del ADR-019.

## Contexto

Cimiento no sabía quién lo estaba usando: cualquiera frente a la computadora podía hacer todo, incluido borrar por completo personas, proyectos o documentos, cambiar la llave de la IA o restaurar un respaldo que reemplaza toda la base. Las usuarias y los usuarios no son técnicos, y un error así es grave. El responsable técnico (administrador) pidió:

- Ser el **administrador universal**.
- Un **panel de administración** donde dar usuario y contraseña al personal y otorgar permisos.
- Por ahora, un solo perfil para la **directora y el contador**: pueden usar todo como él, pero **sin el panel** y **sin acciones peligrosas**.
- Que un borrado hecho por ellos se vuelva **solicitud**, y que el registro **desaparezca para ellos al momento**, para que no sientan que no se borró.
- Más adelante, un módulo remoto para aprobar esas solicitudes a distancia.

## Decisión

1. **Dos roles, por permisos** (`domain/access.rs`). Un rol es una lista de permisos: `use` (trabajar con la app), `delete` (borrar por completo), `settings` (configuración técnica) y `administer` (el panel). El administrador (`admin`) los tiene todos; dirección y contaduría (`manager`) solo `use`. Agregar un rol después es agregar una lista.
2. **Cada comando declara lo que necesita** en `COMMANDS`: abierto (`Open`), sesión (`Session`), un permiso, o «borrar o pedir» (`DeleteOrRequest`).
   - **Revisión en Rust:** cada comando llama primero `guard(&session, "<su nombre>")`, que revisa que haya una sesión abierta y no bloqueada, que no haya contraseña temporal pendiente y que tenga el permiso.
   - **Ninguno sin permiso:** una prueba recorre los comandos registrados y falla si alguno no está en la lista o no llama `guard`.
   - **Negativas:** un intento sin permiso responde «Esto lo hace la persona administradora» y queda en la bitácora (`access.denied`).
   - **La pantalla** solo esconde lo que no se puede usar; la que decide es Rust.
3. **Clasificación de lo existente**:
   - **Solo administrador:**
     - Configuración técnica (`settings`): proveedor, modelos, llave y tope de la IA, la comprobación de la llave, restaurar un respaldo y cargar los ejemplos de desarrollo.
     - El panel (`administer`).
   - **Borrar o pedir (`DeleteOrRequest`):** borrado de emergencia de un documento, borrar a una persona del personal, a un beneficiario o un proyecto, y quitar un dato propio de los formularios (de beneficiarios o de personal).
   - **El resto es `use`**, incluidos crear respaldos y consultar los datos tapados (esa consulta ya queda en la bitácora). Editar partidas o actividades, archivar puestos y cambiar ingresos son trabajo normal, no peligroso.
4. **Borrados que esperan** (tabla `access_request`):
   - **Al pedir:** si quien borra no tiene `delete`, el comando crea una solicitud y **oculta** el registro para todos. Para eso hay una columna `hidden` en `document`, `project`, `roster_entry`, `roster_field`, `hr_person` y `hr_custom_field`.
   - **Mientras espera:** el registro sale de las listas, de los totales y de lo que lee la IA. El comando devuelve lo mismo que un borrado, así que para quien lo pidió quedó borrado.
   - **Decisión del administrador:** si aprueba, se borra de verdad; si rechaza, vuelve.
   - **Datos de un formulario:** un dato oculto conserva sus valores aunque se edite la ficha mientras tanto, así que rechazar no pierde nada.
   - **Duplicados:** no puede haber dos solicitudes pendientes para lo mismo.
   - Esta tabla es la que sincronizará el módulo remoto.
5. **Cuentas** (`app_user`):
   - **Datos:** usuario único (en minúsculas), nombre, enlace opcional a la ficha del personal, rol y si está activa.
   - **Contraseña:** guardada como **Argon2id** (crate `argon2` 0.6, puro Rust; el SHA-256 iterado del PIN no basta para contraseñas). Mínimo 8 caracteres y distinta del usuario.
   - **Contraseña temporal:** la que pone el administrador obliga a cambiarla al entrar.
   - **Intentos:** 5 intentos malos hacen esperar 30 segundos, y la espera se duplica hasta 15 minutos. Queda en la base, así que cerrar la app no la salta. La respuesta nunca dice si falló el usuario o la contraseña.
   - **Siempre un administrador:** debe quedar al menos uno activo; nadie se desactiva a sí mismo.
   - **Por ahora** solo se dan cuentas de dirección y contaduría.
   - **Bajas:** una persona marcada «Ya no trabaja aquí» o borrada pierde su cuenta.
6. **Primera vez**:
   - Sin cuentas, la app pide crear la del administrador. Si la computadora tenía PIN, lo pide antes, para que no cualquiera se quede con la administración, y después el PIN se retira.
   - Se muestra **una sola vez** un **código de recuperación** (16 caracteres; se guarda solo su hash). Con él, el administrador pone una contraseña nueva si olvida la suya, y recibe un código nuevo. Desde el panel también se puede renovar.
7. **Sesión**:
   - Vive solo en memoria de Rust.
   - **Bloqueo por inactividad:** tras 15 minutos sin usar el ratón o el teclado (los cuenta la pantalla) la app pide `access_lock`. Rust mantiene el bloqueo, y solo la contraseña de quien estaba dentro lo abre. El trabajo no se pierde, y también se puede bloquear a mano.
   - **Cerrar sesión** borra de la pantalla lo que veía la persona anterior.
   - **Al restaurar un respaldo** todas las personas vuelven a entrar, porque el respaldo trae sus propias cuentas.
8. **Bitácora con autor**: `audit_log.actor_id`.
   - **Cómo se llena:** al entrar se anota la persona en una tabla temporal de la conexión, y `audit::record` la toma sola. Así no se tocaron los más de 30 lugares que escriben en la bitácora.
   - **Eventos nuevos:** `access.setup`, `auth.login`, `auth.login_failed`, `auth.locked`, `auth.logout`, `auth.recovered`, `auth.password_changed`, `user.created`, `user.updated`, `user.password_reset`, `access.denied`, `request.created`, `request.approved` y `request.rejected`.
   - **Nunca** guardan contraseñas, códigos ni datos de una persona.
9. **Panel de administración** (solo `administer`):
   - Personal y cuentas: dar acceso, contraseña temporal, activar y desactivar.
   - Solicitudes: aprobar o devolver.
   - Bitácora con filtros.
   - Renovar el código de recuperación.

## Consecuencias

- **Base de datos:** migración 0016. Los comandos del PIN desaparecen (`pin_status`, `pin_set`, `pin_clear`, `pin_verify`); de esas funciones solo queda en Rust la que lee un PIN viejo para crear la primera cuenta. `project_job` ahora puede responder error (sesión).
- **Primera ejecución tras actualizar:** la app pide crear la cuenta del administrador antes de mostrar nada.
- **Qué protege y qué no:** las cuentas protegen contra errores y usos indebidos de personas sin perfil técnico. **No sustituyen el cifrado.** La base sigue cifrada con la llave del llavero de Windows, y alguien con acceso técnico a la computadora podría saltarse la pantalla de entrada (el mismo límite que el ADR-019 reconocía para el PIN).
- **Frontend:** hay un frontend que funciona (entrada, primera cuenta, recuperación, cambio de contraseña, bloqueo, panel y menú según permisos); el diseño fino queda para la sesión de diseño.
- **Pendiente:**
  - El módulo remoto de solicitudes.
  - Más roles (capturista, solo lectura).
  - Que el personal en general use la app.
  - Revisar si la dirección debe ver los datos tapados.

## Alternativas descartadas

- **Solo esconder los botones:** la pantalla no es una barrera. Cualquier comando se puede llamar, así que la regla tiene que vivir en Rust.
- **Bloquear sin solicitud:** obligaba a pedir los borrados por fuera y no deja rastro.
- **Mantener el PIN además de la contraseña:** dos secretos por persona, sin ganar nada sobre una contraseña con bloqueo por inactividad.
