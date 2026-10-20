---
adr: 48
estado: propuesta
fecha: 2026-10-20
tema: [ia, datos, seguridad, proceso]
reemplaza_en_parte: ["[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]", "[[ADR-045-tres-patrones-de-la-ia]]"]
tags: [adr]
---
# ADR-048 · El flujo de datos: un modelo de datos, la IA dirige y la casa se cuida hacia afuera

**Estado:** propuesta (2026-10-20). La pidió el líder al detener el desarrollo durante la prueba de IA-2. Mientras no se acepte, no se escribe código nuevo en ningún carril de la IA.

## Contexto

El ADR-043 ya decía «la IA decide dentro de la casa, con pocos límites que no crecen». El código siguió por otro camino, y en dos semanas cada falla se arregló con una pieza más del mismo tipo.

**Lo que pasó el 2026-10-20**, probando la bóveda (IA-2):

- **El Asistente mandaba la calle y la representante a otra ventana.** El mapa que recibe le decía que «Contacto y ubicación» solo lleva «Municipio» y «Estado».
  - El catálogo de campos marca la calle, el correo, el RFC y la representante como `AiUse::Never`: «nunca a la IA».
  - Desde el ADR-041 el Capturista sí los sugiere, y desde el ADR-044 le llegan como marca. El catálogo no se actualizó; se pusieron excepciones en el código de la IA.
  - Hoy cuatro piezas deciden qué campos ve la IA, cada una con su regla: el Capturista (`open_fields`, `told_fields`, `own`), el guardado de sugerencias (`told_own`), la ficha de la IA (`ai_sheet`) y el mapa del Asistente. La que se quedó atrás falló.
- **El escáner marcó «Calle Ana María Rodríguez» como un solo nombre.** El modelo leyó «Estamos en la [NOMBRE_1] 8» y no tenía cómo saber que era una calle. Además, «Rosa Méndez Ruiz» pasó sin marca: el escáner con listas no la reconoció.
- **Todo está diseñado para el plan gratis:**
  - el modelo más barato como regla;
  - 14 llamadas por minuto, una llamada por tarea y topes de tokens;
  - oficios que compensan con casos lo que el modelo barato no entiende.

  En producción no habrá plan gratis.

**El inventario del camino hacia la IA** (código del 2026-10-20) muestra el patrón:

| Qué | Dónde | Qué es en realidad |
|---|---|---|
| Nombres, títulos, santos, palabras de lugar y conectores | `scanner/names.rs` | Interpretación con listas (17 % de las personas cubiertas, ADR-046) |
| Qué campos ve la IA: `open_fields`, `told_fields`, `own`, `told_own`, `allowed` | `core/agents/capturist.rs`, `proposals.rs`, `ai_sheet.rs`, `assistant.rs` | Un dato del modelo de datos repetido en cuatro reglas |
| `Sensitivity::InstitutionalPrivate`: «nunca a la IA ni a una herramienta» | `common/forms.rs` | Una declaración que ya no dice la verdad |
| `BACK_TO_IT` («ese mismo», «con base en eso»…) | `core/agents/assistant.rs` | Interpretación con una lista de frases |
| `claims_done` («dejé», «guardé»…) y `mend_digits` / `KNOWN_WORDS` | `core/agents/assistant.rs`, `core/reader/quote.rs` | Remiendos de lo que escribe el modelo barato |
| Frases que reconocen cada tipo de documento (`KINDS`) | `core/reader/kinds.rs` | Interpretación con listas por tipo |
| Palabras por campo para buscar pasajes (`SEARCH`) y «el archivo habla del campo» (`SPEAKS_OF`) | `core/reader/fill.rs`, `upload.rs` | Interpretación con listas por campo |
| Reglas por tipo de documento (fechas, folios, vigencias) | `core/reader/rules.rs`, `dates.rs` | Lectura con regex; parte es formato fijo, parte es interpretación |
| Oficios con «si la persona hace X…» (Asistente v11) | `core/agents/assistant.v11.md` | Reglamento en el prompt (ADR-045 §3) |
| Modelo más barato, 14 por minuto, 4 herramientas, 40,000 tokens | `ai/settings.rs`, `ai/paced.rs`, cada agente | Supuestos del plan gratis |

El catálogo de opciones (los 32 estados, las figuras jurídicas, los regímenes fiscales) **no está en esta tabla**: no interpreta nada, es la forma del dato.

## Decisión

### 1. Primero el modelo de datos, después el código

Cada dato de SociAI se declara **una sola vez**, en el modelo de datos:
- qué es y qué acepta (texto, fecha, una opción…);
- **de quién es:** de la institución, que es una persona moral, o de una persona física;
- **cómo llega a la IA:** tal cual, como marca o nunca (§2).

**Todas las piezas leen esa declaración y ninguna decide por su cuenta:** el Asistente, el Capturista, los Lectores, el guardado, la ficha de la IA y el mapa. Un campo nuevo, o uno que cambia de clase, llega solo a todas.

- **Sí se escribe a mano:** el modelo de datos y los dominios cerrados (los estados, las figuras jurídicas). Son la forma del dato, se escriben al diseñarlo y no crecen con los errores.
- **No se escribe a mano, nunca:** una lista, un regex o un diccionario que intenta entender lenguaje o contexto.

### 2. Tres clases de dato hacia la IA, por necesidad y no por miedo

| Clase | Qué es | Cómo llega a la IA | Ejemplos |
|---|---|---|---|
| **Tal cual** | Lo que la IA necesita para trabajar y no identifica a una persona física | Con su valor | Lo que la institución es y hace, sus cifras, su nombre legal, el RFC de la institución, su domicilio, su teléfono y su correo institucionales |
| **Marca** | Lo que identifica a una persona física | `[NOMBRE_1]`, `[CURP_1]`: el valor se queda en el equipo (ADR-044) | Nombres de personas (también la representante), CURP, el RFC o el teléfono de una persona |
| **No sale** | Lo que la IA nunca necesita | Ni el valor ni la marca: la IA solo sabe si está lleno | Cuentas bancarias y CLABE, contraseñas y llaves, la salud y la ficha de una persona |

La regla para decidir la clase de un dato: **¿la IA lo necesita para hacer su trabajo, y es de una persona física?** No se decide por cómo se ve el dato ni por lo que podría pasar en un caso.

### 3. El flujo, siempre el mismo

1. **Entra** lo que la persona escribe o sube: el chat, un archivo, un formulario.
2. **El escáner inteligente marca** lo que es de personas físicas: el modelo local (ADR-046) y los formatos con dígito verificador (CURP, RFC, CLABE). Marca, pero no decide si «Ana María Rodríguez» es una calle o una persona: eso lo decide la IA con el contexto. Marcar de más se acepta (ADR-046).
3. **La IA recibe todo el contexto y el modelo de datos completo:** cada campo con su título, lo que acepta y su clase, y lo guardado según la clase. Decide qué es cada cosa, a dónde va y qué hace.
4. **Actúa con sus herramientas.** El código ejecuta y revisa lo que se puede revisar sin adivinar:
   - la marca se resuelve en el equipo;
   - el valor cabe en su campo;
   - la cita está en el texto y la cifra en la cita;
   - el permiso existe.
5. **El resultado llega a la persona con sus valores reales.** Nada se guarda sin ella.

La casa se cuida **hacia afuera**, en el paso 2 y en la clase de cada dato. Adentro, la IA no es el enemigo: dirige.

### 4. Interpretar es de la IA o de un modelo local

Todo lo que pide entender lenguaje o contexto lo hace la IA o un modelo local: qué es un dato, de qué documento se trata, a qué se refiere «ese mismo», qué dice un archivo.

El código se queda con lo que no se interpreta:
- formatos con dígito verificador;
- revisiones sin adivinar (§3, paso 4);
- cálculos (la IA no calcula);
- permisos;
- guardar con origen e historial.

**Un error no se arregla agregando una frase, una palabra, un regex ni un caso al código.** Se busca su causa: el modelo de datos, el contexto que recibió la IA, el escáner o el proveedor.

### 5. Se diseña para producción, no para el plan gratis

- **El modelo se elige por la calidad medida** en las baterías, con un presupuesto mensual por institución. El más barato ya no es la regla de diseño; deja de valer «se valida con el modelo más barato» del ADR-043 §5.
- **Los límites del plan gratis son de desarrollo:**
  - viven en la configuración de pruebas (las llamadas por minuto, el tope diario);
  - nunca en la forma de una pieza (una sola llamada, pocas herramientas, oficios con casos).
- **Un agente usa las llamadas y herramientas que su tarea necesite** dentro del presupuesto. Los topes cuidan el gasto, no compensan al modelo.

### 6. Las pruebas salen del modelo de datos

- **Una prueba recorre el modelo de datos** y exige que cada pieza vea cada campo según su clase. La falla del mapa de hoy habría salido ahí, sin abrir la app.
- **Las baterías se generan del modelo de datos:** por cada campo que la IA puede tomar, alguien que cuenta ese dato. Se suman los casos reales del líder. Una batería mide; no se aprueba agregando frases al código.
- **Lo que solo pasa en la app** se ve con el rastro de desarrollo (`CIMIENTO_AI_TRACE`, en `comandos-y-pruebas.md`): qué recibió el modelo y qué contestó, ya con las marcas.

### 7. Se trabaja en orden

Las reglas están en [flujo de trabajo](../agents/flujo-de-trabajo.md) y [roles y tablero](../agents/roles-y-tablero.md):
- lo abierto se termina antes de empezar algo nuevo;
- lo que se descubre a medio bloque no crece el tablero;
- el líder decide qué entra.

### 8. El nombre es SociAI en todo

El repositorio, las carpetas, el código, la base y el llavero todavía dicen `cimiento`. Se renombran a SociAI en una fase propia, con una migración que **no pierda la base ni la llave** de quien ya usa el programa.

## Fases

Una a la vez. Cada una termina cuando el líder la prueba en la app.

| Fase | Qué | Cómo se sabe que terminó | Lo que se retira |
|---|---|---|---|
| **F0** | Este ADR y las reglas de trabajo en orden | El líder lo acepta; el tablero queda con estas fases | — |
| **F1 · El modelo de datos** | Cada campo declara su clase (§2) y todas las piezas la leen | Una prueba recorre todos los campos y cada pieza ve lo mismo; la secuencia del líder (correo, representante, calle) va a su ventana | `open_fields`, `told_fields`, `own`, `told_own`, `allowed`; la declaración vieja de `InstitutionalPrivate` |
| **F2 · El escáner inteligente** | El modelo local de nombres en Rust (ADR-046, antes IA-8) | El examen de 708 frases: ≥ 98 % de las personas; la calle llega como calle | Las listas de `scanner/names.rs` |
| **F3 · La IA dirige el chat** | Contar datos da una sola respuesta con lo que el código confirmó (antes IA-4); el Asistente recibe el modelo de datos completo | Un dato ya guardado se dice en la misma respuesta; las baterías generadas pasan | `BACK_TO_IT`, `claims_done`, `mend_digits`, `KNOWN_WORDS`; los casos del oficio |
| **F4 · Los Lectores entienden** | El tipo de documento y lo que dice cada archivo los decide la IA, con la cita revisada por el código | La prueba completa del Expediente sin listas por tipo ni por campo | `KINDS` (frases), `SEARCH`, `SPEAKS_OF`; las reglas por tipo que interpretan |
| **F5 · Producción** | Elegir el modelo por calidad y fijar el presupuesto; los límites gratis solo en pruebas | Las baterías con el modelo elegido; el costo por institución anotado | Los supuestos del plan gratis en la forma de cada pieza |
| **F6 · SociAI** | Renombrar `cimiento` sin perder datos | La app instalada abre la base y la llave de antes | El nombre `cimiento` |

**Lo que pasa con lo que ya está en el tablero:**
- **IA-2 (la bóveda):** se queda, porque es el paso 2 del flujo. Sus pruebas pendientes de la app se cierran con F1 y F2.
- **IA-8:** pasa a ser F2.
- **IA-4:** pasa a ser F3.
- **IA-3, IA-6 e IA-7:** se juntan con la fase que les toca o se cierran. Lo decide el líder.
- **El catálogo de campos es del ERP.** F1 cambia su declaración (`common/forms.rs`, `core/institution/forms.rs`), así que se hace con el ERP o en un solo bloque, como decida el líder.

## Consecuencias

- **Se mantienen los límites del ADR-043:**
  - el valor de un dato personal no sale;
  - nada se guarda sin una persona;
  - la IA no calcula;
  - permisos.

  Lo que cambia es dónde vive cada uno: en el modelo de datos y en el borde de la casa, no en reglas repartidas.
- **Se mantienen los tres patrones del ADR-045** como forma de construir. Cambia el criterio para subir de nivel: la medición se hace con el modelo de producción, no con el más barato.
- **Al principio cuesta más por llamada.** A cambio, se deja de pagar con semanas de remiendos.
- **Lo que hoy funciona con reglas no se borra antes de tener con qué reemplazarlo.** Cada fase retira lo suyo cuando su reemplazo pasa sus pruebas.

## Lo que decide el líder

1. **La clase de cada dato de «Mi institución»:** sobre todo el teléfono y el correo institucionales, el domicilio y el RFC de la institución (propuesta: tal cual), y el nombre de la representante (propuesta: marca).
2. **El presupuesto mensual de IA por institución,** para elegir el modelo de F5.
3. **Si F1 se hace con el ERP o en un solo bloque.**
4. **Qué pasa con IA-3, IA-6 e IA-7.**
5. **Si IA-2 se fusiona ya**, con sus pendientes anotados para F1 y F2, **o espera** a esas fases.
