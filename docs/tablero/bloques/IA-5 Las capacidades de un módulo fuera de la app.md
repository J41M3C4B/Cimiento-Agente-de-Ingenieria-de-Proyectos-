---
tipo: bloque
id: IA-5
carril: ia
estado: backlog
prioridad: alta
caja: Contratos de capacidades; Registro común de capacidades
pedido_por: ERP-1
depende_de: IA-1, ERP-8
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

## Terminado cuando

(Lo escribe el riel IA con el líder.)

- [ ] 

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
