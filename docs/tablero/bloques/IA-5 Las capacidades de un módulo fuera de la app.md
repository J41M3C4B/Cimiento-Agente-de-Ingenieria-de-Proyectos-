---
tipo: bloque
id: IA-5
carril: ia
estado: en-curso
prioridad: alta
caja: Contratos de capacidades; Registro común de capacidades; Runtime; Ejecutor; Auditoría de tareas
pedido_por: ERP-1
depende_de: IA-1, ERP-8, ERP-10
rama: ia/ia-5-contrato-y-plataforma-en-crates
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

**Pedido por el riel ERP.** El [plano del ERP](../../plano-del-erp.md) (D5, [ADR-002](../../adr/ADR-002-el-nucleo-del-erp-y-la-forma-de-un-modulo.md)) hace del núcleo y de cada módulo un *crate* de un *workspace* de Cargo, para que el compilador impida las dependencias prohibidas. El ERP-8 arma ese *workspace*, con el vocabulario de permisos dentro del núcleo.

Lo que se necesita:
- **Que un módulo, siendo un *crate* aparte, pueda declarar sus capacidades** con el contrato del IA-1, sin depender de la app. Hoy `capability/` vive dentro de la app y toma `Permission` de `core::access::domain`.
- **Que una capacidad pueda pedir los permisos que declara su propio módulo.** Con el plano, cada módulo declara los suyos en su manifiesto (K2), por ejemplo `contabilidad.polizas.registrar`, y los roles del núcleo los agrupan. Hoy el contrato solo conoce los cuatro permisos generales.

Para qué: el ERP-1 rehace Mi institución como el primer módulo sobre el núcleo, y publica sus capacidades desde su *crate*. Sin esto no puede.

**Con el plano de la IA (ADR-003) y lo que entregó el ERP-10 (2026-10-21), este bloque también:**
- **Pone al contrato y a la plataforma en sus *crates*** (A1 y A2): el contrato, en el *crate* más bajo, nombra el permiso por su id.
- **Conecta el runtime del IA-2 al núcleo:**
  - la autorización, con `kernel::authorize`, en lugar de que la conteste la sesión;
  - la auditoría de las tareas, con `kernel::record_task`, en el almacén del núcleo (K9).

## Terminado cuando

Aprobado por el líder el 2026-10-21, las dos partes juntas. Las casillas se marcan cuando cada punto queda hecho.

- [ ] **El runtime autoriza con el núcleo.** Cada invocación y cada entrega preguntan a `kernel::authorize` por la cuenta, la organización de la base y el id del permiso. Se deniegan, con el motivo de `Denial::code()`:
  - una cuenta desactivada;
  - una cuenta de otra organización;
  - un permiso que no existe.

  Una prueba lo demuestra. La organización deja de ser `"local"`.
- [ ] **La auditoría de las tareas va al núcleo.** Cada momento de una tarea (apertura, paso, denegación, aprobación y cierre) queda en `kernel_audit` con `kernel::record_task`, con el id de la tarea y solo códigos fijos. La tabla `ai_event` se retira: queda un solo almacén de auditoría.
- [ ] **El contrato vive en su *crate*, debajo del núcleo (A2),** y nombra el permiso por su id. Las capacidades de hoy siguen igual, con los permisos generales del núcleo (`kernel.usar`…).
- [ ] **Un módulo declara una capacidad desde su propio *crate*,** con un permiso que él define. El módulo de muestra lo hace, y el runtime la invoca de punta a punta. El registro rechaza una capacidad cuyo permiso no existe.
- [ ] **El runtime de la plataforma vive en su *crate* (A1).** El compilador impide que el núcleo o un módulo usen la plataforma, y una prueba lo demuestra.
  - Lo viejo de `ai/` (proveedores, pipeline, escáner, bóveda) se muda cuando se rehace: con el IA-3 y el IA-4.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto.

## Plan

La rama sale de `main` con el IA-2 y el ERP-10 ya fusionados.

1. **El *crate* del contrato** (`crates/capability`). Lleva `src/capability/` tal como está, salvo una cosa: el permiso deja de ser el `enum` de la app y pasa a ser un id (`permission: "kernel.usar"`). No depende del núcleo, de la plataforma ni de un módulo.
2. **El *crate* de la plataforma** (`crates/platform`). Lleva el runtime del IA-2 (`src/ai/runtime/`).
   - Se llama `platform` y no `ai`, porque la app todavía tiene su módulo `ai/` con lo viejo y los dos nombres chocarían. Se ajusta el plano, con su fila.
   - Depende del contrato y del núcleo.
3. **La autorización, por el núcleo:**
   - el runtime pregunta a `kernel::authorize`, con la cuenta como está ahora;
   - la app le da esa cuenta (`core::access::service::account`), solo si la sesión está abierta, sin bloquear y es de esa cuenta;
   - la organización es la institución de la base;
   - el motivo de una denegación es `Denial::code()`.
4. **La auditoría, en el núcleo:** cada momento de la tarea va a `kernel::record_task`. Una migración nueva de la plataforma retira `ai_event`; una migración nunca se edita.
5. **Los permisos de las capacidades se revisan al arrancar:**
   - una capacidad cuyo permiso no define el núcleo ni ningún módulo detiene la app;
   - una prueba lo revisa sobre el registro real.
6. **El módulo de muestra publica una capacidad** con su permiso `muestra.notas.escribir`, y una prueba de la app la recorre de punta a punta:
   - la Dirección no tiene ese permiso y se le niega;
   - la Administración sí lo tiene.
7. **Las fronteras:**
   - la prueba del *workspace* aprende los dos *crates* nuevos, con sus reglas (A1 y A2);
   - `architecture_tests.rs` deja de clasificar `capability` y `ai::runtime`, que ya no viven en la app.
8. **La suite completa,** medir la primera compilación, y el PR.

**Lo que toca de otros rieles:**
- `Cargo.toml`: los dos *crates* nuevos, como miembros y como dependencias del *workspace*;
- `src/workspace_tests.rs` (del ERP-8): las reglas de los dos *crates* nuevos;
- el módulo de muestra: su primera capacidad.

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
