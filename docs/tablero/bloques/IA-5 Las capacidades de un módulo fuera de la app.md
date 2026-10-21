---
tipo: bloque
id: IA-5
carril: ia
estado: backlog
prioridad: alta
caja: Contratos de capacidades; Registro común de capacidades
pedido_por: ERP-1
depende_de: IA-1, ERP-8, ERP-10
rama:
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

(Propuesta del riel IA; la aprueba el líder.)

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
