# El código frente a los planos

Revisión del 2026-10-20, caja por caja, del código de `main` contra [los planos de SociAI](planos-de-sociai.md). Dice qué tenemos, qué no tenemos y qué necesita refactor.

**Punto de partida.** Lo anterior no funcionaba bien. La metodología fue improvisada y estaba al revés: cada módulo tenía su propio equipo de IA, con sus oficios, su contexto, sus reglas de qué ve la IA y sus límites. Los planos invierten eso: **una sola plataforma de IA para todo el ERP**, y los módulos solo publican capacidades. Por eso aquí nada se conserva por estar hecho. Se conserva solo lo que encaja en una caja del plano sin cargar la forma vieja.

**Veredictos:**
- **Se conserva:** encaja en su caja tal como está, o con cambios menores.
- **Refactor:** la pieza sirve, pero cambia de lugar o de forma.
- **No existe:** hay que construirlo.
- **Se retira:** es la forma vieja; lo que hace se vuelve a construir sobre la plataforma.

---

## 1. El error de fondo: un equipo de IA por módulo

| Dónde | Su IA hoy | Cómo la arma |
|---|---|---|
| **Mi institución** (`core/`) | <ul><li>El **Asistente**: un agente con 4 herramientas de lectura, oficio en su versión 11.</li><li>El **Capturista**: un agente sin herramientas.</li><li>El **Lector**: reglas con frases y expresiones regulares, más dos agentes, uno para llenar y otro para responder.</li><li>La búsqueda por significado.</li></ul> | Cada uno con su oficio, su contexto (`assistant::context`, `capturist::context`, la hoja `ai_sheet.rs`), sus topes y sus parches. |
| **Proyectos** (`modules/projects/`) | 8 tareas de IA propias: conversación, diagnóstico, prioridades, tres de redacción, lectura de convocatorias y su resumen. | Cada una con su *prompt*, su contexto y su propia función `ask_ai`, mezcladas con el negocio del módulo. |

**Las consecuencias:**
- **La IA no se comparte.** El Asistente no sabe usar Proyectos, y la IA de Proyectos no usa las herramientas del Asistente.
- **Cada módulo nuevo traería su propia IA.**
- **Las pantallas llaman directo a 134 comandos.** Cuáles pasan por la IA se decide comando por comando.

---

## 2. Caja por caja

### Capa 1 · Interacción

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Chat global | El Asistente (`src/core/assistant/Assistant.tsx`, `core/agents/assistant.rs`):<ul><li>se abre desde cada pantalla, pero solo conoce Mi institución y el manual;</li><li>no ejecuta nada;</li><li>la conversación vive solo en memoria.</li></ul> | **Refactor.** La ventana se queda; detrás va el runtime. |
| Interfaces de los módulos | Las pantallas de React llaman a 134 comandos de Tauri, cada uno con su propio camino. | **Refactor.** La interfaz se queda. Lo que llaman deja de ser un comando suelto y pasa por el runtime como capacidad. |
| Eventos y tareas automáticas | Avisos sueltos (`record-read`, `record-suggested`, `reading-progress`) y trabajos en segundo plano que viven en memoria: la lectura de documentos y el índice de vectores. No hay tareas programadas ni calendario. | **No existe.** |

### Capa 2 · ERP

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Mi institución | El perfil, los formularios del catálogo, el Expediente y el historial de cada dato (`core/profile`, `core/institution`, `core/records`, `core/history.rs`). Es lo más completo, pero vive mezclado con la IA: `core/agents`, `core/reader` y `ai_sheet.rs` están dentro del núcleo. | **Refactor.** El negocio se queda; la IA sale a la plataforma. |
| Proyectos | Convocatorias, diagnóstico, conversación, redacción, presupuesto y calendario (`modules/projects`). Las tareas de IA están metidas en sus archivos de negocio: `calls.rs` (1,010 líneas), `drafting.rs` (743), `diagnosis.rs` (436). | **Refactor fuerte.** Se quedan el dominio y el almacenamiento; la IA sale a la plataforma. |
| Contabilidad | No hay. `modules/finance` es un balance simple (ingresos, egresos y si alcanza), y está oculto. | **No existe.** |
| Otros módulos | Personal, Beneficiarios e Instalaciones (`modules/hr`, `care`, `facilities`): existen, ocultos (`core/modules.rs`). La prueba de fronteras entre módulos (`architecture_tests.rs`) funciona. | **Se conservan** congelados. |
| Contratos de capacidades | No existen. Hay puertas en Rust entre módulos (`modules/*/api.rs`, `core/api.rs`, `core/bridge/`) y el permiso de cada comando (`COMMANDS`). Ninguna describe, de forma que un registro la lea:<ul><li>la entrada y la salida;</li><li>el efecto;</li><li>el permiso;</li><li>la clase de datos.</li></ul>Hay dos semillas:<ul><li>los formularios descritos como datos (`common/forms.rs`: campos, tipos, validación, sensibilidad);</li><li>el `Tool` del agente (nombre, descripción, esquema de argumentos).</li></ul> | **No existe.** Es la pieza central que falta. |

### Capa 3 · Plataforma de inteligencia

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Runtime de ejecución | Nada. Cada comando de Tauri es su propia entrada. | **No existe.** |
| Enrutador de tareas | Nada. El Asistente decide solo dentro de su oficio; las demás IA se llaman con un botón. | **No existe.** |
| Planificador de flujos | Solo el ciclo de un agente (`ai/agent.rs`): el modelo pide una herramienta o contesta, con tope de pasos y de texto. | **Refactor.** Sirve de base para el «agente planificador», no para el planificador de la plataforma. |
| Registro común de capacidades | `toolbox()` (`core/agents/tools.rs`): 5 herramientas de lectura escritas a mano para el Asistente (`fill_state`, `sheet_section`, `manual_search`, `manual_entry`, `record_reader`). Cada agente lista los nombres que puede usar. | **No existe** como registro común. El contrato `Tool` es la semilla. |
| Gestor de contexto | Cada IA arma el suyo: `ai_sheet.rs` (1,294 líneas escritas a mano), el contexto y el mapa del Asistente, el del Capturista y el de cada tarea de Proyectos. Lo bueno:<ul><li>la búsqueda en documentos por palabras y por significado, con la página citada (`core/archive/search.rs`);</li><li>el mapa de lugares, que sale del manual (`core/places.rs`).</li></ul> | **Refactor.** Uno solo para todos, alimentado por los tres dominios y por la clasificación de datos. |
| Ejecutor de capacidades | Partes sueltas:<ul><li>`ai/pipeline.rs` es un solo camino para toda llamada al modelo: presupuesto, escáner, esquema y registro de uso;</li><li>el ciclo del agente revisa que la herramienta sea suya y que sus argumentos tengan forma.</li></ul>Pero las pantallas ejecutan sin pasar por ahí, y no hay ejecutor para lo determinista. | **Refactor.** El pipeline queda como la parte «llamar al modelo» del ejecutor. |
| Estado, límites y ciclo de vida | No hay tareas. Lo que se parece vive en memoria y se pierde al cerrar: los trabajos en curso de Proyectos (`projects/jobs.rs`) y las lecturas en segundo plano. Límites: solo el tope mensual del pipeline y los topes por agente. | **No existe.** |
| Entrega de resultados | Nada. Cada comando devuelve su propio tipo a su pantalla. | **No existe.** |

**Falla comprobada en el ciclo del agente** (`ai/agent.rs`): si el modelo contesta sin pedir herramienta y sin respuesta, el ciclo cuenta un paso y vuelve a empezar sin revisar si ya se acabaron. Cuando se agotan los pasos, un modelo que siga contestando así hace llamadas sin fin. Es justo lo que pide prevenir la §3.3 de los planos.

### Capa 4 · Capacidades ejecutables

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Servicios deterministas | Muchos y buenos:<ul><li>los servicios de cada módulo;</li><li>la validación de formularios, las fechas y los dígitos verificadores;</li><li>las cifras sin IA (`ai/figures.rs`) y la revisión de citas (`reader/quote.rs`);</li><li>la lectura de PDF, OCR, Word y Excel (`documents/`);</li><li>la exportación a Word.</li></ul> | **Se conservan.** Hay que publicarlos como capacidades. |
| Operadores de IA | Existen, pero cada uno amarrado a su módulo:<ul><li>las 8 tareas de Proyectos;</li><li>el Capturista y el Lector, que son agentes sin herramientas, o sea operadores;</li><li>la lectura de convocatorias por bloques con citas verificadas (`documents/canonical`).</li></ul> | **Refactor.** Se vuelven capacidades registradas. Sus *prompts* se revisan; no se copian. |
| Agentes especializados | El Asistente y el lector que responde. El Asistente carga parches sobre lo que escribe el modelo: cambiar dígitos por letras (`mend_digits`), detectar «ese mismo» (`back_to`), y la frase del botón la pone el código. | **Se retiran** como están. |
| Herramientas e integraciones | <ul><li>5 herramientas, todas de lectura: lo que la IA quiere cambiar queda como propuesta.</li><li>Proveedores intercambiables (Gemini, Anthropic).</li><li>OCR y Pdfium locales y vectores.</li></ul> | **Refactor.** Las herramientas salen de los contratos. Los proveedores y las integraciones locales se conservan. |

### Capa 5 · Dominios de información

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Contexto institucional | El perfil, el catálogo de campos y el origen e historial de cada dato. | **Se conserva.** La hoja escrita a mano (`ai_sheet.rs`) se retira; la sustituye el gestor de contexto. |
| Conocimiento documental | El Expediente, el texto extraído, la búsqueda con página citada y el manual del programa. El tipo de cada documento y sus datos se reconocen con listas de frases y expresiones regulares: `reader/kinds.rs` y `reader/rules.rs`, 885 líneas. | **Refactor.** Se conservan el almacén y la búsqueda. El reconocimiento por frases no es la base. |
| Datos operativos del ERP | Las tablas de cada módulo, cifradas con SQLCipher, y el respaldo. | **Se conserva.** |

### Capa 6 · Control transversal

| Caja | Qué hay hoy | Veredicto |
|---|---|---|
| Autenticación e identidad | Cuentas, sesión, bloqueo y recuperación (`core/access`). | **Se conserva.** |
| Autorización y permisos | Un permiso por comando (`COMMANDS`: abierto, sesión, usar, administrar, borrar), revisado por `guard()`. Las herramientas revisan el rol. Es por comando, no por capacidad, recurso ni dato. | **Refactor.** El permiso pasa al contrato de cada capacidad. |
| Clasificación de datos | Cada campo tiene su sensibilidad y su uso por la IA (`Sensitivity`, `AiUse` en `common/forms.rs`). Pero qué ve la IA se decide también en otros lugares:<ul><li>la hoja (`ai_sheet.rs`);</li><li>cada herramienta;</li><li>las frases seguras del escáner;</li><li>el oficio de cada agente.</li></ul> | **Refactor.** Una sola clasificación. |
| Políticas de datos y PII | El escáner (formatos con dígito verificador y nombres), la bóveda de marcas y la segunda barrera en el pipeline. No hay política por proveedor: nada impide mandar datos personales al plan gratis de Gemini, cuyos términos lo prohíben ([investigación externa](investigacion/investigacion-externa.md), §4.2). | **Refactor.** |
| Validación de contratos y reglas | El esquema JSON de cada respuesta (con un reintento), los argumentos de las herramientas, los campos de formulario, las citas y las cifras. Todo disperso. | **Refactor.** La validación va en el contrato. |
| Confirmaciones humanas | Las propuestas (`ai_proposal`): la IA sugiere un valor por campo, y la persona lo usa y guarda. Es por campo, no por acción con parámetros, y no se invalida si algo cambia. | **Refactor.** |
| Auditoría y trazabilidad | <ul><li>La bitácora de eventos sin contenido (`audit/mod.rs`, unos 40 tipos).</li><li>El uso de IA (`ai_usage`).</li><li>El resumen de cada encargo de agente.</li></ul>No hay id de correlación ni cierre de tarea. | **Refactor.** Se amplía. |

---

## 3. Lo que se retira

- **Los equipos de IA por módulo** tal como están:
  - el Asistente, el Capturista y el Lector, con sus oficios versionados (asistente v1 a v11, capturista v1 a v5);
  - las tareas de IA metidas en Proyectos.

  Lo que hacen se vuelve a construir como capacidades de la plataforma.
- **La hoja de la IA escrita a mano** (`ai_sheet.rs`).
- **Los parches sobre lo que escribe el modelo** y las listas de frases para interpretar.
- **Las reglas de qué ve la IA repartidas** en herramientas, oficios y escáner.

---

## 4. Por dónde empezar (propuesta)

Primero la columna vertebral, probada de punta a punta con una sola capacidad. Después se migra el resto.

1. **El contrato de capacidad y el registro** (ADR-001). Qué declara cada capacidad:
   - nombre y módulo;
   - entrada y salida;
   - efecto (consulta, transformación, modificación, sensible o externa);
   - permiso;
   - clases de datos;
   - si pide aprobación;
   - sus límites.
2. **El esqueleto de la plataforma, sin IA todavía:** runtime, tarea con estado y límites, ejecutor único con los controles, y entrega común. Se prueba con un flujo determinista de Mi institución, de punta a punta; por ejemplo, «qué me falta por llenar».
3. **La clasificación y la política en un solo lugar,** incluida la regla del proveedor gratuito.
4. **El enrutador y un operador:** la primera petición del chat que recorre todo el plano operativo.
5. **Migrar las capacidades:** primero Mi institución y después Proyectos. Cada equipo viejo se retira cuando lo suyo ya corre en la plataforma.

Cada paso con su ADR y sus pruebas.
