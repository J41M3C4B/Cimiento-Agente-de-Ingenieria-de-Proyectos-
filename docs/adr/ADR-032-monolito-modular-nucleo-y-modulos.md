# ADR-032 · Monolito modular: un núcleo y módulos independientes

**Estado:** aceptada (2026-10-08); se aplica por bloques (B0 a B6, abajo). Precisa la estructura del `01-arquitectura.md`; no reemplaza ningún ADR.

## Contexto

Cimiento nació como una herramienta para armar proyectos de donativo. Al construirla salieron funciones que sirven por sí mismas: personal (ADR-027), beneficiarios (ADR-029), instalaciones (ADR-030) y dinero (ADR-026). Tras revisarlas con las religiosas, la institución lo dijo con sus palabras: lo que necesita es un **ERP**. Quiere llevar al personal (y después sus recibos de pago), a los beneficiarios y las finanzas, y que los proyectos se alimenten de todo eso.

Personal, Beneficiarios e Instalaciones ya son módulos aparte: tablas con prefijo, error propio, una frontera `api.rs` que solo entrega agregados y una prueba que falla si el módulo usa algo de la app. El resto sigue unido:

- **15 archivos `*_service.rs` sueltos** en `src-tauri/src/`, sin dueño. Se mezclan el núcleo, Proyectos y la unión de los módulos con el perfil.
- **`domain/` mezcla** reglas del núcleo (perfil, dinero, acceso, primer inicio), de Proyectos (etapas, presupuesto, checklist, conversación) y los tableros que cruzan módulos.
- **`storage/` mezcla** la infraestructura (base, migraciones, llaves, respaldo) con las tablas de cada área.
- **Dependencias al revés:**
  - Instalaciones pasa sus textos por el escáner con una función de Proyectos (`diagnosis_service::guard_texts`).
  - El acceso borra personas llamando a los servicios de Personal y Beneficiarios.
  - El tipo de institución se lee con SQL directo en cinco lugares.
  - Las pruebas de los módulos usan una función de prueba de Proyectos.
- **Un solo error para todo** (`ServiceError`): mezcla errores del perfil, del acceso, de Proyectos y de los módulos.
- **El dinero vive dentro de las versiones del perfil** (`income_source` y `expense_item` cuelgan de `institution_profile`). No se puede hacer crecer como módulo sin tocar el perfil.
- **En la pantalla**, Personal, Beneficiarios e Instalaciones son pestañas de «Mi institución».

## Decisión

### 1. Un monolito modular

Un solo programa y un solo crate de Rust, partido en capas con reglas que revisa una prueba. No se usan microservicios ni un workspace de Cargo por ahora: las pruebas de fronteras dan el mismo aislamiento sin la complejidad de varios crates. El día que un módulo deba separarse (otro producto, otro equipo), su carpeta ya no depende de nada más que la base.

### 2. Cinco capas, cada una mira solo hacia abajo

| Capa | Dónde | Qué es | Puede usar |
|---|---|---|---|
| **Comandos** | `commands/` | Capa delgada: revisa el permiso, llama y convierte el error en un mensaje amigable. Lo único que conoce Tauri | todo |
| **Proyectos** | `modules/projects/` | Convocatoria, conversación, diagnóstico, redacción, revisión y guía en Word. Consume lo que el núcleo sabe de la institución | la base y `core::api` |
| **Núcleo** | `core/` | La institución: datos generales y cifras rápidas, primer inicio (abre el espacio de trabajo), cuentas, permisos y seguridad, archivo de documentos, tableros que cruzan módulos y la ficha que lee la IA | la base y `api`, `service` y tipos de `domain` de cada módulo; nunca su `storage` ni su `legacy` |
| **Módulos** | `modules/hr`, `care`, `facilities`, `finance` | Registran la operación: Personal, Beneficiarios, Instalaciones y Finanzas | solo la base. No se conocen entre sí ni conocen al núcleo |
| **Base** | `ai/`, `audit/`, `common/`, `documents/`, `scanner/`, `storage/` | Infraestructura sin reglas de negocio: proveedor de IA, bitácora, validadores, lectura de archivos, escáner, base cifrada, migraciones, llaves y respaldo | solo la base |

- **Excepción permanente:** `storage::migrations` puede llamar al `legacy` de los módulos. Los traslados de datos viejos a un módulo se hacen en una migración (como en ADR-027, 029 y 030).
- **Proyectos está arriba del núcleo y no a su lado** porque es quien consume: la IA de proyectos lee la ficha de la institución que arma el núcleo. Proyectos no pide nada directo a otro módulo; todo le llega por `core::api`. Un módulo futuro que también consuma (por ejemplo, Donantes) vive en el mismo piso.
- **Una sola prueba de fronteras** (`architecture_tests.rs`) clasifica cada archivo en su capa y falla si alguien mira hacia arriba o hacia los lados. Las violaciones de hoy están en una **lista de deuda** que solo puede achicarse: la prueba también falla si una excepción ya no hace falta y sigue en la lista. Las pruebas de frontera de cada módulo se quedan.

### 3. Cómo es un módulo por dentro

Igual que Personal, Beneficiarios e Instalaciones:

```
modules/<módulo>/
  mod.rs       error propio (thiserror) y su prueba de frontera
  domain/      reglas puras, sin E/S
  storage.rs   sus tablas, con prefijo propio (hr_, care_, fac_, fin_)
  service.rs   casos de uso de sus pantallas
  api.rs       lo único que el núcleo le puede pedir: agregados, indicadores y borrar a petición del acceso
  legacy.rs    traslado de datos anteriores (si los hay)
```

- **Tablas de Proyectos:** conservan sus nombres (`project`, `grant_call`, `need`, `budget_item`…). Renombrarlas no aporta nada y arriesga los datos. El dueño de cada tabla queda escrito en `05-modelo-datos.md`.
- **Errores:** cada módulo tiene el suyo. `ServiceError` queda como el error del núcleo y Proyectos tiene `ProjectsError`. `error.rs` (capa de comandos) los convierte todos en el mensaje amigable y el código interno de siempre.

### 4. Cómo se conectan sin tocarse

- **Tipo de institución:** el núcleo lo dice en un solo lugar (`core::institution::kind`). Cada módulo convierte ese texto en su `Flavor` (`Flavor::from_kind`). Así nadie vuelve a leer `institution` con SQL.
- **Después de guardar en un módulo**, el núcleo pasa los textos por el escáner y actualiza lo que se deriva: las líneas anónimas del perfil (`profile_sync`) y el balance.
- **Borrados que aprobó el administrador** (ADR-028): el acceso llama a `api::delete_person` del módulo y luego vuelve a derivar. Ya no llama a servicios de pantalla.
- **IA:** ningún módulo arma prompts sobre otro. Todo lo que la IA sabe de la institución sale de la ficha del núcleo (`core::ai_sheet`), que lee el `api::ai_summary` de cada módulo. Las reglas de qué llega a la IA (ADR-020, 026, 027, 029 y 030) no cambian.
- **Permisos:** solo en `commands/` (ADR-028). Los módulos no saben quién está usando la app.
- **Sin registro dinámico ni bus de eventos.** El núcleo nombra a cada módulo en un `match` que el compilador revisa. Con cinco módulos, la indirección no paga.

### 5. Finanzas pasa a ser un módulo (`modules/finance/`)

- **Qué se lleva:** ingresos por tipo, egresos (la lista o el aproximado exprés) y el balance, con las reglas de `domain/finances.rs` (ADR-026).
- **Tablas propias** (`fin_income`, `fin_expense` y `fin_settings` para el aproximado de egresos). El dinero **deja de ser parte de las versiones del perfil**: cada línea conserva `origin`, `source_ref`, `confirmed_at` y `confirmed_by`.
- **Migración:** copia las líneas del perfil vigente a `fin_*`. Las tablas viejas no se borran: quedan como historia de las versiones anteriores del perfil y ya no se escriben.
- **Nómina y cuotas** se siguen calculando en Personal y Beneficiarios. El núcleo se las pasa a Finanzas como líneas anónimas para el balance; Finanzas nunca ve personas.
- **Primer inicio:** el paso «Dinero» guarda en Finanzas a través del núcleo, y la ficha de la IA lee `finance::api`. El paso sigue siendo obligatorio.
- **Comandos:** la pantalla tiene comandos nuevos `finance_*`, y `profile_save` deja de llevar ingresos y egresos.
- **Después, cada módulo con su ADR:** movimientos, presupuestos y reportes en Finanzas; recibos de pago en Personal (nómina), que nunca salen hacia la IA.

### 6. Proyectos se separa sin lógica nueva

Su desarrollo queda **en pausa**. Se mueve tal cual a `modules/projects/`:

- llamadas, conversación, diagnóstico, redacción, guía, revisión y procesos de IA (`jobs`);
- sus reglas (etapas, presupuesto, cronograma, checklist, prioridades, requisitos y secciones);
- su almacenamiento (`projects`, `calls`, `drafting`).

Los prompts siguen en `ai/prompts/`. Lo que hoy pide directo al perfil o a Instalaciones pasa a pedirlo a `core::api`.

### 7. En la pantalla: el núcleo arriba, los módulos en el riel

- **Frontend:** `src/core/` (Inicio, Mi institución, primer inicio, acceso, documentos y ajustes) y `src/modules/` (personal, beneficiarios, instalaciones, finanzas y proyectos con convocatorias). `components/`, `lib/` e `i18n/` siguen compartidos.
- **El riel** muestra primero el núcleo (Inicio, Mi institución) y después un renglón por módulo, cada uno con su color de marca (`docs/13-sistema-visual.md`). Ajustes y Ayuda quedan abajo.
- **«Mi institución»** se queda con lo que es de la institución: datos generales, legales, contacto, capacidad y cifras rápidas. De cada módulo muestra un resumen corto («12 personas en el personal») que lleva al módulo. Las pestañas de Personal, Beneficiarios e Instalaciones salen de ahí.
- **Reparto del trabajo:** la estructura y la navegación funcional se hacen en esta sesión; el diseño fino, en la sesión de diseño.

### 8. Lo que no cambia

- Un solo crate, una sola base cifrada y una sola lista de migraciones numeradas (nunca editadas).
- Los nombres de las tablas existentes y de los comandos, salvo los nuevos de Finanzas.
- El perfil sigue guardando las líneas anónimas de personal y beneficiarios. Cambiarlo es un cambio de datos que necesita su propio ADR.
- Todos los módulos están siempre activos. Encenderlos y apagarlos por institución queda para cuando haga falta.
- Los principios de siempre: determinista primero, la IA nunca calcula, todo dato lleva origen, cuántos y nunca quiénes.

## Orden de trabajo

Cada bloque es un commit propio y termina con `cargo test`, `pnpm test` y `pnpm build` en verde. Salvo B3 y B6, **ningún bloque cambia el comportamiento.**

| Bloque | Qué | Terminado cuando |
|---|---|---|
| **B0** | Este ADR, la documentación y la prueba de fronteras con la lista de deuda | La prueba pasa y la lista nombra cada violación de hoy |
| **B1** | Ayudantes del escáner a la base; `core::institution::kind` y `Flavor::from_kind`; `api::delete_person` en los módulos | Salen de la lista las violaciones del escáner, del tipo de institución y de los borrados |
| **B2** | `hr`, `care` y `facilities` a `modules/`. Mecánico | Las pruebas de frontera de cada módulo pasan desde su nueva carpeta |
| **B3** | Módulo de Finanzas: tablas `fin_*`, migración, comandos, el paso «Dinero» y la ficha de la IA | Las pruebas de `finances` pasan desde el módulo; el balance y la ficha dicen lo mismo que antes con los ejemplos |
| **B4** | `core/`: institución, primer inicio, acceso, seguridad, documentos, tableros, ficha de la IA y la unión con los módulos; `core::api` | En la lista no queda nada del núcleo |
| **B5** | Proyectos a `modules/projects/` con `ProjectsError`, consumiendo solo `core::api` | La lista de deuda queda vacía |
| **B6** | Frontend en `src/core/` y `src/modules/`, el riel con los módulos y «Mi institución» con los resúmenes. Notas «Para cloud» | `pnpm test` y `pnpm build` en verde; se navega a cada módulo desde el riel |

## Consecuencias

- **Un módulo nuevo** (Donantes, Nómina, Inventario) se arma con la misma forma y entra al núcleo por su `api`. Su lógica se construye desde cero sin tocar lo que ya funciona, como se hizo con Personal.
- **La IA no cambia lo que lee.** Las pruebas de `institution_context` cuidan que la ficha diga lo mismo antes y después de mover el código.
- **Los movimientos de B2, B4 y B5 cambian rutas, no comportamiento.** Por eso se hacen en commits aparte de cualquier cambio de lógica, para revisarlos fácilmente.
- **La sesión de diseño** recibe en B3 los comandos nuevos de Finanzas y en B6 la nueva estructura de `src/`. Hasta entonces, sus pantallas no notan nada.

## Alternativas descartadas

- **Microservicios o un proceso por módulo:** la app es de escritorio, con una sola base cifrada en la computadora de la institución. Solo sumaría fallas y despliegue.
- **Un workspace de Cargo con un crate por módulo desde ya:** el aislamiento sería el mismo que dan las pruebas de fronteras, pero costaría compilaciones más lentas (SQLCipher ya tarda) y mucho movimiento. Queda abierto para cuando un módulo se separe de verdad.
- **Registro dinámico de módulos con traits o un bus de eventos:** con cinco módulos que se conocen en tiempo de compilación, esconde el flujo sin ganar nada. Un `match` explícito es más fácil de seguir y de probar.
- **Dejar el dinero en el núcleo:** Finanzas crecería atada a las versiones del perfil, y separarla después costaría más.
- **Que los módulos lean del núcleo:** los volvería dependientes del perfil. Es mejor que el núcleo les pase lo poco que necesitan (el tipo de institución).
