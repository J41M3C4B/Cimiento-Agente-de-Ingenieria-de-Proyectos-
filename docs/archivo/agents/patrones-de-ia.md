# Los tres patrones: determinista, IA acotada y agentes

Decisión de origen: [ADR-045](../adr/ADR-045-tres-patrones-de-la-ia.md). Lee este documento antes de diseñar cualquier pieza que use IA, o que podría usarla.

SociAI construye todo con **tres patrones**. **No hay chats:** ningún modelo contesta texto libre sin un operador o un agente detrás, con su esquema y la revisión del código. La caja de conversación de la pantalla es solo una puerta: lo que la persona escribe entra a un operador que decide a dónde va.

| Patrón | Qué es | Quién decide los pasos | Ejemplos de hoy |
|---|---|---|---|
| **1 · Determinista** | Reglas y código. La capa más robusta y el núcleo de todo | El código | Escáner, cálculos del presupuesto, permisos, el tipo de un documento por sus palabras, las reglas por tipo, la política de llenado, la revisión de una cita |
| **2 · IA acotada (operadores)** | Una tarea con principio y fin conocidos: la misma entrada, los mismos pasos y la forma del resultado definida, aunque el valor no se sepa | El código. La IA hace el juicio en un punto fijo | El Capturista, el Lector (al subir, al llenar, al contestar), la lectura de una convocatoria, el turno del diagnóstico, la redacción |
| **3 · Agentes** | Autonomía acotada por permisos: los pasos cambian según lo que decide el modelo y lo que encuentra | El modelo, dentro de sus herramientas y topes. El código pasa a segundo plano | El Asistente (el único) |

**Operador** es la pieza del patrón 2 y **agente** la del patrón 3. Una función determinista no lleva nombre propio.

**Decidir no es lo mismo que ser agente.** Que la IA decida qué es un dato, a qué campo va o si conviene ofrecer un botón es un juicio, y cabe en un operador. Ser agente quiere decir que el modelo controla el camino: qué hace después depende de lo que encontró antes.

## 1. Cómo se decide el patrón: las cuatro preguntas

Se contestan en orden, por escrito, en el «Plan» del bloque, antes de programar. La respuesta y su por qué se anotan.

**Pregunta 1. ¿Hay que interpretar lenguaje o juzgar?** Por ejemplo, entender lo que escribió una persona o lo que dice un documento, decidir qué es algo o a dónde va, o elegir cómo decirlo.
- **No → patrón 1, determinista.** Formatos fijos (un RFC, una fecha con formato, la estructura de un PDF), catálogos cerrados, cálculos, permisos, guardar y validar.
- **La prueba de la lista:** si hacerlo con código pide una lista de palabras o frases que **crece con cada caso nuevo**, la respuesta es «sí». Es interpretación disfrazada de regla (ADR-043 §5). Una lista que crece por tipo, y no por caso (las frases que reconocen un acta), puede seguir siendo determinista.

**Pregunta 2. ¿Se conocen los pasos desde el principio?** Se cumple si las tres cosas son ciertas:
- el código puede juntar todo lo que hace falta **antes** de llamar a la IA;
- basta una llamada, o una secuencia fija de llamadas;
- la salida cabe en un esquema cerrado que el código puede revisar.

**Sí → patrón 2, operador.**

**Pregunta 3. ¿Lo que sigue depende de lo que se encontró?** Por ejemplo, qué leer después según lo que dijo lo anterior, o una pregunta abierta que mezcla fuentes que no se pueden juntar antes sin mandarlo todo.
- **Antes de elegir agente:** si los caminos son pocos y se conocen, **no es agente**. Es un operador que enruta (clasifica la intención en un esquema cerrado) más un operador o una función por camino.
- **Solo si los caminos no se pueden enumerar → patrón 3, agente.**

**Pregunta 4. ¿Es el nivel más bajo que lo resuelve?** Siempre se elige el más bajo.
- **Subir** de nivel pide una medición en seco y real que muestre que el de abajo no alcanza.
- **Bajar** cuando la batería lo muestra (ver §3).

## 2. Lo que vale en los tres

Los cuatro límites del ADR-043 no dependen del patrón:
- el valor de un dato personal no sale del equipo;
- nada se guarda sin una persona;
- la IA no calcula;
- permisos: cada pieza tiene sus herramientas y cada campo dice qué acepta.

Además, toda llamada pasa por el mismo camino (`ai::pipeline`): presupuesto → escáner → ritmo → proveedor → esquema → `ai_usage`. No hay otro. En un encargo con bóveda (el Asistente y el Capturista, ADR-044), lo que halla el escáner va como marca (`[NOMBRE_1]`) y el código pone el valor en el equipo.

## 3. Señales de un patrón equivocado

| Lo que se ve | Qué quiere decir | Qué hacer |
|---|---|---|
| El código tiene una lista de frases que crece con cada falla | Es interpretación, no regla | Pasarla a un operador o a un límite (la bóveda, un permiso) |
| El oficio de un operador o agente tiene una lista de «si la persona hace X…» que crece con cada versión | El prompt se volvió reglamento | Partir: un operador que enruta y uno por camino |
| Dos llamadas para una sola decisión, unidas por un botón de la pantalla | El código no encadenó lo que ya sabía | El código las encadena, o la segunda es herramienta de la primera |
| Un agente pide siempre las mismas herramientas en el mismo orden en su batería | No necesitaba decidir el camino | Bajarlo a operador: el código junta antes lo que pedía |
| El código remienda lo que escribió el modelo (una letra por un número, una frase prohibida) | Falta una revisión de fondo, o el modelo no alcanza | Que el código confirme lo que se afirma; medir un modelo más fuerte |
| Un operador necesita «ir a buscar» algo según lo que leyó | Su camino no es fijo | Medir si lo resuelve un mejor contexto; si no, es agente |

## 4. La ficha de cada patrón

Se escribe en el «Plan» del bloque antes de programar. Es lo que hace replicable cada pieza: un operador nuevo copia la ficha de uno que ya existe.

### Patrón 1 · Determinista

| Qué dice | |
|---|---|
| **Regla** | Qué entra, qué sale y qué casos cubre |
| **Dónde vive** | El `domain/` de su módulo o del núcleo, en Rust |
| **Pruebas** | Positivos y negativos de cada regla; obligatorias para el escáner, cálculos, etapas, Excel y Word |
| **Uso por la IA** | Puede ser herramienta de un agente: solo lectura, revisa el permiso de quien pregunta y devuelve solo lo que puede llegar a la IA |

Nunca llama a la IA.

### Patrón 2 · Operador (IA acotada)

| Qué dice | |
|---|---|
| **Tarea** | Una frase: de qué entrada a qué salida |
| **Quién lo dispara** | La persona (un botón), un evento (subir un archivo), el operador que enruta o un agente |
| **Entrada** | Lo que junta el código y lo que **nunca** va (datos protegidos, nombres de archivo, fichas de personas) |
| **Salida** | Un esquema JSON cerrado |
| **Revisión del código** | Lo que se puede revisar sin adivinar: la cita está en el texto, la cifra en la cita, el valor cabe, el campo está permitido |
| **Qué hace el código con lo que pasa** | Una sugerencia (`ai_proposal`), un texto en pantalla o un dato con su origen |
| **Modelo y topes** | El más barato que pase su batería; número de llamadas; tokens |
| **Oficio** | Un archivo versionado. Describe la tarea y su criterio, no una lista de casos |
| **Batería** | Casos con lo esperado, también «nada»; primero en seco, después real con el modelo más barato; aciertos, inventados, llamadas, costo y tiempo |

No tiene herramientas y no llama a nadie. Puede ser herramienta de un agente: así trabaja hoy el Lector con el Asistente (`record_reader`).

### Patrón 3 · Agente

Lleva la ficha del operador y además:

| Qué dice | |
|---|---|
| **Por qué es agente** | Qué camino no se puede enumerar (pregunta 3) |
| **Herramientas** | Lista cerrada: funciones deterministas de solo lectura y operadores. Cada una revisa el permiso; la frontera es la herramienta, no el oficio |
| **Delegación** | Un nivel: un agente llama operadores, nunca a otro agente |
| **Topes** | Pedidos de herramientas y tokens por encargo |
| **Lo que decide** | Qué pedir, qué preguntar, cómo decirlo y qué ofrecer |
| **Lo que nunca** | Guardar, calcular, ni decir que algo pasó («quedó guardado», «le dejé sugerencias») si el código no lo confirma |
| **Revisión de su respuesta** | Las fuentes son citas que entregaron sus herramientas, los lugares salen del mapa del programa y lo que afirma lo confirmó el código |
| **Batería** | Rondas reales, porque un modelo no decide igual cada vez; por caso, los pasos y las herramientas que pidió, para ver si podría bajar a operador |

## 5. Cómo nace una pieza nueva

1. **El bloque dice la tarea** y contesta las cuatro preguntas (§1). El patrón va en la propiedad `patron` del bloque.
2. **La ficha** de su patrón (§4) va en el «Plan».
3. **La batería se escribe antes que el código:** los casos con lo esperado, también los que no deben dar nada.
4. **El código:**
   - lo determinista va en `domain/`;
   - un operador o un agente va en la carpeta de su módulo o del núcleo, sobre `ai::agent` y el pipeline.
5. **Se mide** primero en seco y después una corrida real con el modelo más barato. Los números van a `07-ia-y-costos.md` y al roadmap.
6. **Al cerrar,** la pieza entra en la tabla de §6.

## 6. Lo que tenemos hoy

| Pieza | Patrón | Código | Nota |
|---|---|---|---|
| Escáner, tipo de documento, reglas por tipo, política de llenado, revisión de citas y cifras, mapa de lugares, cálculos de Proyectos | 1 | `scanner/`, `core/reader/`, `core/agents/proposals.rs`, `core/places.rs`, `modules/projects/domain/` | |
| Capturista | 2 | `core/agents/capturist.rs` | Una llamada, sin herramientas |
| Lector al subir un archivo | 2 | `core/reader/upload.rs` | Una llamada por archivo |
| Lector «Llenar con mis documentos» | 2 | `core/reader/fill.rs` | Una llamada por ventana |
| Lector que contesta | 2 | `core/reader/ask.rs` | Herramienta del Asistente |
| Lectura de una convocatoria y su ficha | 2 | `documents/canonical/`, `modules/projects/calls.rs` | Secuencia fija de llamadas |
| Turno y resumen del diagnóstico, necesidades | 2 | `modules/projects/conversation.rs`, `diagnosis.rs` | El código decide la etapa; la IA la redacta |
| Redacción (plan, sección, todo) | 2 | `modules/projects/drafting.rs` | |
| Asistente | 3 | `core/agents/assistant.rs` | Hasta 4 herramientas; hoy enruta en su oficio (ver deuda) |

**Deuda conocida.** Lo que hoy está en el patrón equivocado, con el bloque que lo resuelve:

| Qué | Hoy | Debe ser | Bloque |
|---|---|---|---|
| El Asistente decide en su oficio entre cinco caminos («Primero vea qué hace la persona») | Reglamento en el prompt (v11) | Un operador que enruta y despacha | IA-4 |
| «Llenarlo por mí»: el Asistente ofrece el botón y el Capturista llama otra vez | Dos llamadas unidas por un botón | El Capturista como herramienta; un dato ya guardado se dice en la misma respuesta | IA-4 |
| `BACK_TO_IT` («ese mismo», «con base en eso») | Lista de frases | Lo decide el operador que enruta | IA-4 |
| `claims_done` (borra un «guardé» del modelo) | Remiendo | El código confirma lo que se afirma | IA-4 |
| `mend_digits` («dej3») | Remiendo del modelo más barato | Se queda medido; se revisa con IA-4 | IA-4 |
| Palabras del escáner para nombres (`scanner/names.rs`) | Lista que crece por caso | Un modelo local de nombres (ADR-046); la bóveda (ADR-044, IA-2) ya hace que marcar de más no cueste | IA-8 |
| El Capturista, los Lectores y el Asistente se llaman todos «agent» en el código (`ai::agent::run`, `fn agent()`) | Nombres que no dicen el patrón | Operador y agente separados en el código | IA-6 |
