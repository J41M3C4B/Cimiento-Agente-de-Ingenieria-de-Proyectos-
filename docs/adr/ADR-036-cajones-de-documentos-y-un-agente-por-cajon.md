---
adr: 36
estado: aceptada
fecha: 2026-10-12
tema: [documentos, ia, nucleo]
reemplaza_en_parte: ["[[ADR-033-nucleo-expediente-y-formularios-universales]]", "[[ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento]]"]
tags: [adr]
---
# ADR-036 · Cajones de documentos con dueño, un agente por cajón y el Asistente como orquestador

**Estado:** aceptada (2026-10-12), por bloques (C1 a C4); primero solo el núcleo.

**Reemplaza en parte:**
- del ADR-033 §5, que «Documentos sigue mostrando todos los archivos» como un lugar propio;
- del ADR-034, el Lector único (§1), que el código elija siempre al agente (§1) y la herramienta del Asistente que busca en todos los documentos (precisiones de IA5);
- de las precisiones de IA6a, que el Lector corra sobre cualquier archivo.

Lo demás de esos ADR sigue igual. En particular: la IA solo propone y una persona acepta; las herramientas solo leen; un solo nivel de delegación; los módulos no saben que existe la IA.

## Contexto

Con IA5 e IA6a se vio que el diseño se revolvía:

- **Un solo montón de documentos.** Lo que se sube en Documentos y lo que se pone en el Expediente acaba en la misma tabla (`document`, `kind = 'internal'`). Nadie es dueño de un archivo: se sabe que existe, pero no a qué parte de la institución pertenece.
- **Un solo Lector que lee todo.** El Lector de IA6a corre sobre cualquier archivo, venga de donde venga, y deja sugerencias en «Mi institución». Si se sube la documentación de una doctora o una factura, el mismo Lector la tendría que entender.
- **Un Asistente que busca en todo sin saber qué hay.** Su herramienta `documents_search` busca en todos los documentos a la vez, y su contexto no le dice qué documentos existen. Probado por la institución (2026-10-12): a «según mi registro, ¿qué nombre legal pongo?» respondió con el manual; a «¿tienes acceso a registro_JAP.png?» respondió «no tengo acceso a imágenes», aunque la imagen ya estaba leída como texto.
- **Reglas por tipo de documento que no escalan.** Las reglas de IA6a sirven para formatos fijos (la constancia del SAT), pero un acta real de 14 páginas y la ficha de la Junta las rompieron hasta ajustarlas una por una. No se puede escribir una regla para cada documento posible del expediente.

La institución lo resumió así: «necesitamos alguien en cada cajón para organizar bien la información, para extraer la información y para compartir los datos».

## Decisión

### 1. Cada documento pertenece a un solo cajón

Un **cajón** es el lugar donde vive un documento, y tiene un solo dueño:

| Cajón | Dueño (ADR-032) | Qué guarda | Qué llena |
|---|---|---|---|
| **Expediente** | Núcleo | Los documentos de la institución: los 18 tipos del catálogo y los que la institución nombre («Otro documento») | «Mi institución» |
| **Personal** | Módulo `hr` | La documentación de cada persona que trabaja | La ficha de esa persona |
| **Beneficiarios** | Módulo `care` | La documentación de cada persona atendida | La ficha de esa persona |
| **Instalaciones** | Módulo `facilities` | Planos, dictámenes, permisos del inmueble | El inmueble y sus espacios |
| **Finanzas** | Módulo `finance` | Facturas y recibos, de ingresos o de egresos (decisión del 2026-10-12) | Ingresos y egresos |
| **Proyectos** | Proyectos | Las convocatorias | La lectura canónica (ADR-015) |
| **Donantes** | (cuando exista) | Reglas, indicadores y formatos de un donante | — |

- **No hay documentos sin cajón** (decisión de la institución): «no pueden quedar volando sin clasificación precisa». Lo que es de la institución pero no está en el catálogo va al Expediente como «Otro documento», que ya existe.
- **Dentro del cajón, el documento dice qué es:** en el Expediente, su tipo (acta, constancia…); en Personal y Beneficiarios, de qué persona y qué papel es; en Finanzas, factura o recibo. Lo dice la persona al subirlo; una regla puede sugerirlo, nunca lo decide.
- **Se empieza de cero** (decisión de la institución): los documentos que ya existen no se migran; se vuelven a subir como se debe. Un documento anterior a este ADR no tiene cajón: ningún Lector lo lee y ningún agente lo busca.

### 2. Documentos deja de ser un cajón: es la bandeja

- **Ahí se sube cualquier archivo, y lo primero es decir a qué cajón va** y qué es dentro de él: Expediente y tipo, Personal y de quién, Finanzas y factura o recibo, Proyectos y convocatoria.
- **Después muestra todo ordenado por cajón,** pero no es dueño de nada: quien guarda, entiende y comparte el documento es el dueño del cajón.
- **Subir desde el cajón o desde la bandeja da lo mismo.** Un archivo puesto en el Expediente desde «Mi institución» y uno subido en la bandeja hacia el Expediente acaban en el mismo lugar, con el mismo Lector.
- Al recibirlo, la bandeja hace lo de siempre, igual para todos los cajones:
  - lo lee (texto, escaneos y fotos, ADR-035);
  - lo pasa por el escáner (cuarentena) y rechaza una lista de personas;
  - guarda solo el texto limpio, nunca el original (principio 6).

  Después se lo entrega al dueño.

### 3. Un solo almacén de texto, con la etiqueta del cajón

- **El texto limpio, sus fragmentos, el índice de palabras y los vectores se guardan en un solo lugar,** como infraestructura sin reglas de negocio. Cada documento y cada fragmento lleva la etiqueta de su cajón.
- **El dueño guarda la referencia:** el Expediente, qué documento del catálogo es; Personal, de qué persona.
- **Buscar siempre es buscar en un cajón:** la búsqueda de IA5 (palabras y vectores fusionados) recibe el cajón y nunca devuelve un fragmento de otro. No hay búsqueda «en todo».
- **Vectores solo donde la IA puede leer:** Expediente, Proyectos y, cuando existan, Finanzas y Donantes. Nunca Personal ni Beneficiarios (§5).

### 4. Un agente por cajón: el Lector de ese cajón

Cada cajón tiene su **Lector**, que solo ve su cajón y solo propone en los formularios de su dueño:

- **Lector del Expediente.** Es dueño de los formularios de «Mi institución»: sabe qué campos tienen, cuáles faltan y cuáles la IA puede ver.
  - **Reglas primero,** sin IA ni costo, para los formatos fijos (las de IA6a). Son las únicas que pueden proponer un dato protegido (RFC, folios, representante); leen el texto antes de que el escáner lo tape.
  - **Después, búsqueda por campo:** para cada campo, una consulta con sus palabras y las del manual (por ejemplo, «Nombre legal»: denominación, razón social). Busca en el Expediente los 2 o 3 pasajes que hablan de él; no lee los documentos completos.
  - **Una llamada a la IA por ventana** con esos pasajes. Propone un valor por campo y copia la frase exacta de donde lo sacó.
  - **El código revisa:**
    - que la frase esté en ese pasaje;
    - que el valor quepa en su campo;
    - que el campo no sea protegido.

    Lo que no pasa, se descarta.
  - Llena todo «Mi institución», no solo «Datos legales y fiscales». No espera a que se suba un documento en otro lado.
- **Lector de Finanzas.** Va con el trabajo del módulo de Finanzas: lee facturas y recibos y propone ingresos y egresos con las cuentas revisadas por el código.
- **Lector de Proyectos.** Es la lectura canónica de convocatorias (ADR-015), sin cambio.
- **Lectores de Personal y Beneficiarios.** **Solo reglas, en esta computadora, sin IA** (§5).
- **Lector de Instalaciones.** Se decide al llegar.

### 5. Lo que no se negocia

- **Cuántos, nunca quiénes (principio 5).** Los documentos de Personal y Beneficiarios son datos de una persona: nada de ellos llega a la IA, ni pasajes ni vectores. Su Lector solo usa reglas aquí (una CURP, una fecha, el nombre de un título). Hacia fuera, su cajón solo comparte conteos y agregados, como sus módulos.
- **Los módulos no saben que existe la IA** (ADR-032, ADR-034 §2):
  - el cajón y sus datos son del módulo;
  - los agentes que usan IA (el Lector del Expediente, el de Finanzas) los registra el núcleo con lo que el módulo ofrece por su `api`;
  - las reglas sin IA de un módulo pueden vivir en el módulo.
- **La IA solo propone; una persona acepta.** Todo lo que un Lector encuentra es una sugerencia (`ai_proposal`) con su documento y su página. Nada se guarda sin «Usar» y «Guardar».

### 6. El Asistente como orquestador

- **El Asistente no lee documentos: le pregunta al agente del cajón.** Sus herramientas incluyen a los agentes de cada cajón (**agente como herramienta**). Ante «según mi registro, ¿qué nombre legal pongo?», decide preguntarle al Lector del Expediente, que busca en su cajón y le devuelve el valor con su cita. Ante una duda de una factura, al de Finanzas.
- **Decide la IA, dentro de lo que el código permite.** El Asistente elige a qué agente preguntar, pero solo entre los que su lista tiene. Cada agente tiene su tope de pasos y de texto. Se conserva **un solo nivel de delegación**: un agente de cajón no pregunta a otro.
- **La herramienta `documents_search` de IA5 desaparece** y la reemplaza una por cajón. Ningún agente busca en todo.
- **Un mapa en el contexto de cada agente,** para que sepa qué existe antes de decidir:
  - qué cajones hay;
  - qué documentos tiene cada cajón al que puede preguntar (nombre y tipo, nunca contenido);
  - que los escaneos y las fotos ya están leídos como texto;
  - qué formularios le tocan, qué falta y qué sugerencias esperan.
- **«Llenar con mis documentos»** en cada ventana de «Mi institución» corre el Lector del Expediente para esa ventana. El Asistente también puede pedirlo.

### 7. Nuestra arquitectura de agentes, en Rust

- **Se queda el bucle propio** (`ai/agent.rs`), no LangChain ni otro marco:
  - LangChain es Python o JavaScript, y llevarlo a la app obligaría a meter Python en el instalador o a correr el agente en el WebView, donde la llave nunca puede estar;
  - los marcos en Rust todavía son jóvenes;
  - lo difícil no es el bucle sino lo que lo rodea, y eso ningún marco lo trae: escáner en cada paso, cuántos y nunca quiénes, tope mensual, `ai_usage`, permisos, cadena de modelos de respaldo, sugerencias en lugar de guardados.
- **Se mide, con preguntas de referencia por agente:** herramientas nativas de cada proveedor contra nuestro protocolo JSON (ADR-034 §2 lo dejó abierto). Se cambia solo si gana.

### 8. Cada agente se prueba, y con el modelo más barato

Decisión de la institución: «si el barato lo puede hacer, entonces un modelo más caro lo hará igual sin problema».

- **Cada agente tiene su batería de referencia** antes de darse por terminado:
  - tareas con el resultado esperado: el cajón y la herramienta correctos, el valor correcto y la cita correcta;
  - con datos ficticios de `fixtures/`, como las preguntas de referencia de IA5;
  - primero en seco, con el modelo simulado y sin gasto;
  - después, una corrida real acotada.
- **Se valida con el modelo más barato disponible** (hoy `gemini-3.5-flash-lite`). La meta es que el barato cumpla:
  - si falla, primero se mejora lo que le damos (el mapa, las herramientas, las descripciones, el oficio);
  - solo si ya no se puede mejorar, se mide un modelo más fuerte para ese agente y se anota por qué.
- **Las métricas de cada corrida se anotan** en el roadmap y en `07-ia-y-costos.md`:
  - aciertos por tarea, herramienta bien elegida y cita verificada;
  - llamadas, tokens, costo y tiempo.

## Orden de trabajo

**Primero el núcleo, solo el núcleo** (decisión de la institución, 2026-10-12): «necesitamos dejar el core sin problemas y probar todo para que lo demás se haga a partir de la metodología». Los módulos esperan.

El núcleo se usa para fijar la forma de trabajar: el cajón, el agente del cajón, su batería de referencia con el modelo más barato y la prueba de la institución. Cada módulo la repite después tal cual. Mientras tanto, la bandeja solo ofrece el cajón del Expediente; los demás aparecen con su módulo.

Cada bloque se prueba por la institución antes del siguiente.

| Bloque | Qué | Terminado cuando |
|---|---|---|
| **C1** | El cajón de cada documento: etiqueta en el almacén y referencia del dueño. Documentos como bandeja que pregunta a qué cajón va (hoy solo el Expediente, con su tipo). La búsqueda siempre por cajón | Ningún documento nuevo queda sin cajón (prueba). Subir al Expediente desde la bandeja o desde «Mi institución» da lo mismo. La búsqueda de un cajón nunca devuelve otro |
| **C2** | El Lector del Expediente: reglas primero, búsqueda por campo, una llamada por ventana con citas verificadas, «Llenar con mis documentos» en cada ventana de «Mi institución» | Batería de referencia con el expediente ficticio (valor y página esperados por campo) aprobada con el modelo más barato; costo por ventana anotado |
| **C3** | El Asistente como orquestador del núcleo: el Lector del Expediente como herramienta, el mapa en el contexto, sin `documents_search` general. Se mide herramientas nativas contra el protocolo JSON | Batería de preguntas por pantalla («según mi registro…», «¿qué documentos tengo?») aprobada con el modelo más barato: agente correcto, respuesta con cita |
| **C4** | Revisión completa del núcleo: cada agente del núcleo (Asistente, Capturista, Lector del Expediente) con su batería aprobada con el modelo más barato, y cada pantalla del núcleo probada por la institución | El núcleo sin pendientes conocidos; la metodología escrita para los módulos |
| Después, módulo por módulo | Su cajón, su Lector y su batería, con la misma metodología: Finanzas (facturas), Personal y Beneficiarios (Lector de reglas, sin IA), Instalaciones, Donantes; el patronato desde un documento | Con su propio bloque |

## Consecuencias

- **Documentos cambia de función:** de cajón propio a bandeja; su pantalla se rediseña alrededor de «¿a qué cajón va?».
- **El Lector de IA6a pasa a ser el Lector del Expediente.** Sus reglas se quedan como su primera capa y solo corren sobre documentos del Expediente.
- **La búsqueda de IA5 y sus vectores se quedan;** cambia que siempre se busca en un cajón.
- **El Asistente cambia de herramientas** (agentes de cajón en lugar de `documents_search`) y de contexto (el mapa). Su prompt sube de versión.
- **Más pruebas por agente,** y cada una con el modelo más barato primero.
- **Los documentos subidos antes de este ADR no se usan;** se vuelven a subir.

## Alternativas descartadas

- **Un Lector general que lee todo** (como en IA6a): mezcla contextos, no puede respetar «cuántos, nunca quiénes» por cajón y crece sin dueño.
- **Reglas por tipo de documento para llenar las fichas:** no escalan a la variedad real de los expedientes; se quedan solo para los formatos fijos y los datos protegidos.
- **Que cada módulo guarde su propio texto y sus vectores:** repetiría la lectura, el escáner y la búsqueda en cada módulo. El almacén es infraestructura; el dueño es de quien es la referencia.
- **LangChain u otro marco de agentes:** no encaja en una app de escritorio en Rust con la llave fuera del frontend, y no trae la gobernanza que necesitamos (§7).
- **Validar con el modelo más fuerte y bajar después:** esconde los problemas de contexto y herramientas detrás de un modelo caro. Se valida con el barato.
