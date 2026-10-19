# Roles, tablero y ramas

Cómo se organiza el trabajo desde 2026-10-18. El tablero es [[Tablero]] (`docs/tablero/Tablero.md`); cada bloque es una nota en `docs/tablero/bloques/` hecha con la plantilla `docs/plantillas/bloque.md`.

## Roles

| Rol | Quién | Decide | Hace |
|---|---|---|---|
| **Líder de producto, arquitectura y validador** | La persona dueña del proyecto | Qué entra y en qué orden; los ADR; qué cuenta como terminado; si un bloque pasa a «Hecho» | Aprueba «Terminado cuando», prueba en la app lo que está en «Revisión» y deja observaciones en `docs/notas/` |
| **CTO** | Claude | Cómo se construye: el plan técnico, las ramas, las pruebas y las mediciones, dentro de los ADR | Propone bloques y prioridad, escribe el plan, implementa, prueba, mide, documenta, abre el PR y mueve el bloque a «Revisión». Avisa antes de lo que no se puede deshacer |

El CTO nunca mueve un bloque a «Hecho» ni fusiona a `main` sin la validación del líder. Una decisión de arquitectura la propone el CTO y la aprueba el líder en un ADR.

## Carriles

- **`erp`:** el producto y su arquitectura (pantallas, datos, formularios, flujos, Proyectos).
- **`ia`:** la arquitectura de la IA (agentes, bóveda, oficios, herramientas, modelos, baterías).

Un bloque que toca los dos va al carril de su decisión principal. Si es grande, se parte en uno de cada carril, y el de `ia` va primero cuando el de `erp` depende de él.

## Columnas

| Columna | Entra cuando | Quién lo mueve |
|---|---|---|
| **Backlog** | Hay una idea, un hallazgo o un pendiente («Qué y por qué») | Cualquiera |
| **Listo** | «Terminado cuando» está escrito y aprobado | El líder |
| **En curso** | El CTO lo toma, crea su rama y escribe el «Plan». **A lo más un bloque por carril** | El CTO |
| **Revisión** | Las pruebas pasan, el PR está abierto y «Cómo probarlo» está escrito | El CTO |
| **Hecho** | El líder lo probó, se fusionó a `main` y el «Resultado» está escrito | El líder valida; el CTO fusiona y cierra |

- **Lo que se descubre a medio bloque no se mete en él:** va a un bloque nuevo en el Backlog. Solo entra si sin eso el bloque no cumple su «Terminado cuando».
- **Un bloque que vuelve de «Revisión»** regresa a «En curso» con lo que faltó anotado en «Cómo probarlo».
- **El roadmap** (`09-roadmap.md`) dice las fases y sus criterios; **el tablero** dice qué se hace ahora. Al cerrar un bloque se marca su avance en el roadmap y en el mapa.
- **Las notas** (`docs/notas/`) siguen siendo las observaciones del líder. Una nota que pide trabajo se vuelve un bloque y se enlaza desde él.

## Ramas y versiones

- **`main` es lo validado.** Siempre compila y sus pruebas pasan, y nadie trabaja directo sobre ella.
- **Una rama por bloque,** nacida de `main` y con el carril y el id en el nombre:
  - `erp/erp-1-ocultar-modulos` o `ia/ia-1-boveda` para los bloques;
  - `fix/<lo-que-arregla>` para un arreglo urgente fuera del tablero.
- **Commits** cortos, en inglés, que empiezan con el id del bloque: `ERP-1: hide the frozen modules from the rail`.
- **Del bloque a `main`:** un PR por bloque con «Terminado cuando» y «Cómo probarlo». Cuando el líder lo valida, se fusiona con *squash* (un commit por bloque en `main`) y la rama se borra.
- **Versiones** (SemVer, la misma en `package.json`, `Cargo.toml` y `tauri.conf.json`): cada fase del MVP validada sube la versión menor con su etiqueta (`v0.2.0`, `v0.3.0`…), y `v1.0.0` es el MVP validado con las dos instituciones.
- **Una carpeta por sesión de trabajo.** Dos sesiones en la misma carpeta se cambian la rama una a la otra, así que cada bloque en curso trabaja en su propio `git worktree` (por ejemplo, `D:\Agente Proyectos\cimiento-ia` para el carril `ia`). Cada carpeta compila aparte: unos 4 GB por carpeta.
