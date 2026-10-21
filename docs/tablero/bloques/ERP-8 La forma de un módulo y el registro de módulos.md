---
tipo: bloque
id: ERP-8
carril: erp
estado: backlog
prioridad: alta
caja: Módulos independientes; Registro de módulos (K12); Almacenamiento (K4)
pedido_por:
depende_de: ERP-7
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El [plano del ERP](../../plano-del-erp.md) (§3, §4 y D5) dice que el núcleo va debajo de todos los módulos, que todos los módulos tienen la misma forma y que el compilador hace cumplir las fronteras. Hoy todo es una sola *crate*, las fronteras las vigila una prueba que lee el texto, las migraciones están en una sola lista compartida y los módulos se apagan con una lista a mano (`HIDDEN`). Este bloque levanta el esqueleto donde se construye todo lo demás: el *workspace*, el núcleo como *crate*, la forma de un módulo, el registro de módulos y las migraciones por módulo, probados con un módulo de muestra.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] **Un *workspace* de Cargo** con el núcleo como *crate* propio y un módulo de muestra como otro *crate*. La app los usa, abre igual que hoy y una base que ya existe abre sin perder nada.
- [ ] **Las fronteras las hace cumplir el compilador:** el núcleo no puede usar un módulo, y un módulo solo puede usar de otro sus tipos publicados (`contract`). Una prueba lo demuestra.
- [ ] **El registro de módulos** junta los manifiestos sin ninguna lista, rechaza una dependencia que no existe o que forma un ciclo, y sabe qué módulos están encendidos.
- [ ] **Las migraciones son por módulo:** cada uno numera las suyas sin chocar con las de otro, y las 34 de hoy quedan como la historia de la app.
- [ ] **La prueba de conformidad** corre sobre todo módulo registrado (manifiesto, dependencias, migraciones sobre una base vacía, prefijo de sus tablas), y el módulo de muestra la pasa. Las partes del diccionario y de las capacidades se le agregan en ERP-2 y ERP-1.
- [ ] **El vocabulario de permisos vive en el núcleo** y cada módulo puede declarar los suyos en su manifiesto (K2), para que el IA-5 los use.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto. Se mide y se anota cuánto tarda la primera compilación.

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
