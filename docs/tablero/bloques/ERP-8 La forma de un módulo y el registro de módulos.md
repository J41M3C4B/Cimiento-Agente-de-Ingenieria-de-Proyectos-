---
tipo: bloque
id: ERP-8
carril: erp
estado: en-curso
prioridad: alta
caja: Módulos independientes; Registro de módulos (K12); Almacenamiento (K4)
pedido_por:
depende_de: ERP-7
rama: erp/erp-8-forma-de-un-modulo
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El [plano del ERP](../../plano-del-erp.md) (§3, §4 y D5) dice que el núcleo va debajo de todos los módulos, que todos los módulos tienen la misma forma y que el compilador hace cumplir las fronteras. Hoy todo es una sola *crate*, las fronteras las vigila una prueba que lee el texto, las migraciones están en una sola lista compartida y los módulos se apagan con una lista a mano (`HIDDEN`). Este bloque levanta el esqueleto donde se construye todo lo demás: el *workspace*, el núcleo como *crate*, la forma de un módulo, el registro de módulos y las migraciones por módulo, probados con un módulo de muestra.

## Terminado cuando

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [ ] **Un *workspace* de Cargo** con el núcleo como *crate* propio y un módulo de muestra como otro *crate*. La app los usa, abre igual que hoy y una base que ya existe abre sin perder nada.
- [ ] **Las fronteras las hace cumplir el compilador:** el núcleo no puede usar un módulo, y un módulo solo puede usar de otro sus tipos publicados (`contract`). Una prueba lo demuestra.
- [ ] **El registro de módulos** junta los manifiestos sin ninguna lista, rechaza una dependencia que no existe o que forma un ciclo, y sabe qué módulos están encendidos.
- [ ] **Las migraciones son por módulo:** cada uno numera las suyas sin chocar con las de otro, y las 34 de hoy quedan como la historia de la app.
- [ ] **La prueba de conformidad** corre sobre todo módulo registrado (manifiesto, dependencias, migraciones sobre una base vacía, prefijo de sus tablas), y el módulo de muestra la pasa. Las partes del diccionario y de las capacidades se le agregan en ERP-2 y ERP-1.
- [ ] **El vocabulario de permisos vive en el núcleo** y cada módulo puede declarar los suyos en su manifiesto (K2), para que el IA-5 los use.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto. Se mide y se anota cuánto tarda la primera compilación.

## Plan

1. **El *workspace*.** `src-tauri/Cargo.toml` pasa a ser también la raíz de un *workspace*. La app (`sociai`) se queda donde está, con todo el código de hoy, y los *crates* nuevos van en `src-tauri/crates/`. Las versiones de las dependencias comunes (como `rusqlite` con SQLCipher) se fijan una sola vez en el *workspace*, para que se compilen una sola vez.
2. **El núcleo, `crates/kernel`.** En este bloque lleva solo lo que el esqueleto necesita:
   - **el manifiesto de un módulo:** id, nombre, versión, de qué módulos depende y sus permisos;
   - **el registro de módulos:** los junta al enlazar, sin lista, con `linkme`, igual que el registro de capacidades del IA-1. Revisa ids, dependencias que no existen y ciclos, y sabe cuáles están encendidos;
   - **las migraciones por módulo:** una tabla nueva lleva la versión de cada módulo, y las 34 de hoy se quedan en `schema_migrations` como la historia de la app;
   - **el vocabulario de permisos:** `Permission` pasa al núcleo. `core::access::domain` lo reexporta, así que `capability/` (del riel IA) no cambia. Además, cada módulo declara los suyos en su manifiesto, con su id como prefijo.
3. **Un módulo son dos *crates*:** el módulo y su `contract`. Así, otro módulo solo puede depender del `contract`. El módulo de muestra (`crates/modules/muestra`) tiene las dos partes, con una tabla y una migración, y la app solo lo enlaza en las pruebas.
4. **Las fronteras.** Una prueba lee el grafo de dependencias de Cargo (`cargo metadata`, sin red). Falla si el núcleo depende de un módulo, o si un módulo depende de otro que no sea su `contract`. `architecture_tests.rs` sigue vigilando el código viejo de la app hasta que se retire.
5. **La prueba de conformidad** corre en la app sobre todo módulo registrado: manifiesto, dependencias, migraciones sobre una base cifrada vacía, el prefijo de sus tablas y el de sus permisos.
6. **La suite completa,** abrir la app con una base que ya existe, medir la primera compilación, y el PR.

**Lo que toca de otros rieles:** solo `Cargo.toml` y `Cargo.lock` (se dice en el PR), y una línea en `core::access::domain` para reexportar `Permission`. No toca `capability/` ni `ai/`.

**Riesgos:**
- **El enlazador puede descartar un *crate* al que nadie nombra,** y con él sus `static` de `linkme`. Por eso la app nombra cada *crate* de módulo una vez (`use muestra as _;`), y una prueba falla si un módulo de las dependencias no aparece en el registro.
- **La compilación de las tres carpetas cambia.** El *workspace* comparte `target/`, y no debería recompilar SQLCipher. Se mide antes de abrir el PR.

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
