# Roles, tablero y ramas

Cómo se organiza el trabajo desde 2026-10-18. El tablero es [[Tablero]] (`docs/tablero/Tablero.md`); cada bloque es una nota en `docs/tablero/bloques/` hecha con la plantilla `docs/plantillas/bloque.md`.

## Roles

| Rol | Quién | Decide | Hace |
|---|---|---|---|
| **Líder de producto, arquitectura y validador** | La persona dueña del proyecto | Qué entra y en qué orden; los ADR; qué cuenta como terminado; si un bloque pasa a «Hecho» | Aprueba «Terminado cuando», prueba en la app lo que está en «Revisión» y deja observaciones en `docs/notas/` |
| **CTO del ERP** | Claude, en `D:\Agente Proyectos\cimiento` | Cómo se construye el ERP determinista y su arquitectura, dentro de los ADR | Los bloques del carril `erp`: propone, planea, implementa, prueba, documenta, abre el PR y mueve el bloque a «Revisión». Avisa antes de lo que no se puede deshacer |
| **CTO de la IA** | Claude, en `D:\Agente Proyectos\cimiento-ia` | Cómo se construye la IA: operadores, agentes, oficios, modelos y baterías, dentro de los ADR | Lo mismo, con los bloques del carril `ia` |

Ningún CTO mueve un bloque a «Hecho» ni fusiona a `main` sin la validación del líder. Una decisión de arquitectura la propone el CTO de su carril y la aprueba el líder en un ADR.

## Dos espacios: el ERP y la IA

Cada CTO trabaja solo en su carril, en su carpeta y con su rama. **Lo que es del otro carril no se hace aquí: se le pide al otro.** Así cada uno conoce a fondo su parte y ninguno cambia lo del otro sin que lo sepa.

**La frontera son los patrones** ([patrones de la IA](patrones-de-ia.md), ADR-045):

| | CTO del ERP | CTO de la IA |
|---|---|---|
| **Patrón** | 1 · Determinista: reglas y código | 2 · IA acotada (operadores) y 3 · Agentes |
| **La prueba** | Su trabajo se prueba **sin ningún modelo**, ni simulado | Su trabajo necesita un modelo (real o simulado), un oficio o una batería |
| **Es suyo** | Datos y migraciones; formularios y su validación; reglas de negocio (`domain/`); cálculos (avance, cifras, presupuesto); etapas y flujos; permisos y acceso; fronteras del monolito (ADR-032); comandos Tauri; pantallas; el interruptor de módulos; el manual y el mapa de lugares; leer archivos (Pdfium, OCR, Word, Excel); el tipo de un documento por sus palabras y las reglas por tipo; la política de llenado y el guardado (`proposals`); el escáner; respaldo y seguridad | Los operadores (Capturista, Lector, lectura de convocatorias, diagnóstico, redacción) y el Asistente; sus oficios (`*.vN.md`); qué recibe cada uno (contexto, la ficha de la IA, la bóveda); sus herramientas; la revisión de citas y cifras; modelos y proveedores; `ai_usage` y costos; sus baterías y corridas reales |
| **Sus documentos** | `principios-de-ingenieria.md` (lo general), `05-modelo-datos.md`, `docs/manual/` | `patrones-de-ia.md`, `metodologia-lectores.md` §3 y §3b, `07-ia-y-costos.md` |

**Las piezas que se tocan** tienen un dueño y un contrato:
- **El ERP ofrece y la IA usa.** El catálogo de campos (qué acepta cada uno), el mapa de lugares, el interruptor de módulos, las funciones que sirven de herramienta y el único punto de guardado (`proposals::propose`) son del ERP. La IA los llama, pero no los cambia.
- **Lo que la IA propone lo guarda el código del ERP**, con su validación, el escáner y el historial (ADR-034, principio 1).
- Un cambio del ERP que cambia lo que ve la IA, como un campo nuevo, una página oculta o una opción con otro nombre, lo hace el ERP. **Lo que la IA hace con eso le toca a la IA:** el oficio, el contexto o una pregunta nueva en la batería.

**Cómo se pide algo al otro carril:**
1. Se escribe un bloque en su carril, en «Backlog», o una nota «Anotado desde ERP-n» en un bloque suyo que ya exista. Lleva qué se necesita y por qué, sin decirle cómo hacerlo.
2. Si el bloque propio no cumple su «Terminado cuando» sin eso, se anota la dependencia en los dos bloques y se avisa al líder. El bloque propio espera; no se hace el trabajo del otro.
3. En duda de a qué carril va algo, decide el líder.

**Lo compartido:**
- **El tablero:** cada CTO mueve solo los bloques de su carril.
- **El mapa del proyecto:** cada uno escribe solo lo de su carril en «Hoy».
- **El roadmap y `docs/agents/`:** cambios cortos, en el PR de su bloque; si cambian lo del otro, se le avisa en su tablero.
- **`cargo test` completo** lo corren los dos antes de abrir un PR: la suite es una sola.

**Ejemplo, ERP-1 (2026-10-19):** al ocultar los módulos, el Asistente seguía mandando «los ingresos» a otra ventana. Con esta regla:
- el ERP habría dado la lista de lo oculto (`core::modules::not_yet`);
- el oficio v11 y las preguntas nuevas de la batería se habrían pedido como un bloque `ia`.

Esa vez se hizo todo en este espacio. Desde ahora, no.

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
