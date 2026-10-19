---
adr: 36
estado: aceptada
fecha: 2026-10-12
tema: [documentos, ia, nucleo]
reemplaza_en_parte: ["[[ADR-033-nucleo-expediente-y-formularios-universales]]", "[[ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento]]"]
reemplazado_en_parte_por: ["[[ADR-038-politica-unica-de-llenado-desde-documentos]]"]
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
  - lo pasa por el escáner (cuarentena) y rechaza una lista de personas; *en el Expediente, desde el 2026-10-13, el escáner solo tapa: ni rechaza ni pregunta (ver «Precisión del 2026-10-13»)*;
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

**Los módulos se tratan como si no existieran** (nota de la institución, 2026-10-12, `notas/C1_NOTAS`): el proyecto, por ahora, es Inicio, Mi institución y Documentos. Nada del núcleo se adapta a lo que hoy tienen Personal, Beneficiarios, Instalaciones, Finanzas o Proyectos, ni se arregla nada de ellos: cuando la metodología funcione en el núcleo, cada módulo se rehace a su medida. La columna «De los donantes» de Documentos sale de la pantalla mientras ese cajón no exista; las convocatorias siguen en Proyectos, sin tocarlas.

Cada bloque se prueba por la institución antes del siguiente.

| Bloque | Qué | Terminado cuando |
|---|---|---|
| **C1** | El cajón de cada documento: etiqueta en el almacén y referencia del dueño. Documentos como bandeja que pregunta a qué cajón va (hoy solo el Expediente, con su tipo). La búsqueda siempre por cajón | Ningún documento nuevo queda sin cajón (prueba). Subir al Expediente desde la bandeja o desde «Mi institución» da lo mismo. La búsqueda de un cajón nunca devuelve otro |
| **C2** | El Lector del Expediente: reglas primero, búsqueda por campo, una llamada por ventana con citas verificadas, «Llenar con mis documentos» en cada ventana de «Mi institución» | Batería de referencia con el expediente ficticio (valor y página esperados por campo) aprobada con el modelo más barato; costo por ventana anotado |
| **C3** | El Asistente como orquestador del núcleo: el Lector del Expediente como herramienta, el mapa en el contexto, sin `documents_search` general. Se mide herramientas nativas contra el protocolo JSON | Batería de preguntas por pantalla («según mi registro…», «¿qué documentos tengo?») aprobada con el modelo más barato: agente correcto, respuesta con cita |
| **C4** | Revisión completa del núcleo: cada agente del núcleo (Asistente, Capturista, Lector del Expediente) con su batería aprobada con el modelo más barato, y cada pantalla del núcleo probada por la institución | El núcleo sin pendientes conocidos; la metodología escrita para los módulos |
| Después, módulo por módulo | Su cajón, su Lector y su batería, con la misma metodología: Finanzas (facturas), Personal y Beneficiarios (Lector de reglas, sin IA), Instalaciones, Donantes; el patronato desde un documento | Con su propio bloque |

### Precisiones al aplicar C1 (2026-10-12)

- **El cajón es una columna del almacén** (`document.drawer`, migración 0032): `record` o `projects`. Dar de alta un documento exige su cajón (`Drawer` en `core/archive/storage.rs`); no hay otra forma de guardarlo.
- **La bandeja no guarda nada por su cuenta.** «Subir documento» pregunta el cajón y qué documento del Expediente es, y abre la misma ventana del Expediente de «Mi institución» (`RecordWindow`), que pone el archivo con su lectura, su escáner, su Lector y su cuarentena. Se quitaron la subida propia de Documentos, el texto pegado y el lápiz de «qué documento es»: en el Expediente lo dice el documento donde se pone.
- **Un documento del Expediente tiene un solo archivo:** al reemplazarlo o quitarlo, el texto anterior sale del cajón con sus fragmentos y sus vectores.
- **Lo aceptado se queda:** «Borrar esto y todo lo que salió de ahí» ya no borra el origen (`core_field`) ni el historial (`core_change`, que solo crece) de un dato que la persona aceptó de ese documento. Antes ese borrado fallaba.

### Precisión del 2026-10-13: en el Expediente el escáner tapa, no rechaza ni pregunta

**Decisión de la institución.** Al probar con documentos reales, todo el Expediente salía rechazado con «Ese archivo es una lista de personas»: las actas constitutivas y hasta una constancia de situación fiscal. Los papeles de la institución traen, por fuerza, datos de personas: socios, patronato, representante legal, notario. Cuántos traen depende de cada institución y de cada documento. Ningún tope de datos personales por archivo le atina, así que la restricción no es viable.

- **En el Expediente, el escáner tapa todo lo que encuentra y el archivo se guarda.**
  - Ya no hay rechazo por «lista de personas».
  - Ya no hay ventana de cuarentena al poner un archivo: la persona ya no elige «Tapar y seguir».
  - La pantalla dice cuántos datos se taparon («Tapamos 3 datos personales en su texto…»). Cada tapado queda en la bitácora con sus conteos, nunca con los valores.
- **El principio 5 se cumple igual:**
  - solo se guarda el texto tapado;
  - la IA nunca recibe un dato de persona del Expediente;
  - el original no se toca.

  El Lector lee antes de tapar, como ya hacía (IA6): el RFC o el nombre legal de la institución se siguen proponiendo con su cita.
- **Se mantiene igual:**
  - el nombre y las notas que la persona escribe a mano en un documento del Expediente siguen con su cuarentena;
  - las convocatorias de Proyectos conservan el rechazo de una lista de personas (no son papeles de la institución).
- En el código:
  - `core::archive::service::cover_pages` reemplaza a `guard_pages`;
  - `AttachOutcome` ya no tiene `Quarantine` y trae `covered`;
  - `Unreadable::Roster` se quitó.
  - Prueba: `an_act_full_of_partners_is_kept_with_their_data_covered` (un acta ficticia con seis socios, su CURP y RFC, y un renglón «NOMBRE, EDAD, CURP»).

### Precisiones al aplicar C2 (2026-10-13)

- **Disparo (decisión de la institución):** «Llenar con mis documentos» en cada ventana de «Mi institución» (1 llamada) y «Llenar todo con mis documentos» en la tarjeta del Expediente (4 llamadas). Al subir un archivo siguen corriendo solo las reglas, sin costo.
- **Búsqueda por campo** (`core/reader/fill.rs`):
  - la consulta es el título del campo en el manual más sus palabras en `SEARCH` (cómo lo escriben los documentos);
  - 3 pasajes por campo, por palabras y por significado cuando hay vectores, sin repetir fragmentos;
  - solo los campos **vacíos** que la IA puede ver: lo lleno no se pide (ADR-038) y lo protegido solo lo leen las reglas.
  - Sin documentos con texto, sin campos vacíos o sin pasajes, no se llama a la IA.
- **Una llamada por ventana** con el modelo ligero (`core/agents/reader.v1.md`). Recibe los campos vacíos, con cada opción y su nombre en la pantalla, y los pasajes numerados con su documento y página.
- **Lo que revisa el código:**
  - que el campo sea uno de los pedidos;
  - que el pasaje se haya mostrado;
  - que la cita esté en ese pasaje (sin acentos ni puntuación);
  - que una cifra esté en la cita, con cifras o con palabras: la IA no calcula;
  - que el valor quepa en su campo.

  Lo que pasa queda como sugerencia del Lector, con origen `document`, documento, página, cita, autoridad y huella, bajo la política del ADR-038.
- **Medición** (`reader_live`, `fixtures/expediente/lector.json`, `gemini-3.5-flash-lite`):
  - **primera corrida:** 16 bien, 0 inventados, 17 de 17 páginas. Faltó el rubro de donataria porque la IA veía los códigos de las opciones solo en inglés. Se corrigió en la causa: cada código va con su nombre de la pantalla (`option_label`, leído de `es-MX.ts`), para el Lector y para el Capturista;
  - **segunda:** 18 de 18 bien, 0 mal, 0 faltantes, 0 inventados, 18 de 18 páginas; 4 llamadas, 11,428 tokens de entrada y 1,660 de salida, **$0.14 MXN las cuatro ventanas**, de 1.6 a 3.6 s cada una.
  - En la ventana de capacidad, la IA intentó una cifra que no estaba en el texto (la suma de residentes y estancia de día) y el código la descartó.
- **Revisado en el WebView real:** con dos PDF ficticios en el Expediente, «Llenar todo» dejó el personal (31) y el voluntariado (18), cada uno con «Lo leímos en «informe-anual-2025.pdf», página 1».

### Precisiones al aplicar C3 (2026-10-13)

- **La IA ve el tipo del documento, nunca el nombre de su archivo** (decisión de la institución: «el nombre del archivo es simplemente la referencia de la cita»).
  - Un documento llega a la IA como «Constancia de situación fiscal» (su tipo del catálogo, con su nombre de `es-MX.ts`) o con el nombre que la institución le dio a un «Otro documento».
  - El nombre del archivo solo aparece en la pantalla, junto a la cita, y lo pone el código.
  - Vale para todos los agentes: el mapa del Asistente, los pasajes del Lector que contesta y los pasajes de «Llenar con mis documentos» (C2).
  - Si la persona nombra un archivo en su pregunta («¿qué dice registro_JAP.png?»), el código lo reconoce (sin acentos, mayúsculas, guiones ni extensión) y le dice al Asistente qué documento del mapa es, para que pregunte por ese documento.
  - En el código: `core::records::ai_name`, `records::map` y `records::named_in`.
- **El Lector también contesta preguntas** (`core/reader/ask.rs`, oficio `reader_answer.v1.md`):
  - busca en su cajón los 4 pasajes que mejor responden, por palabras y por significado;
  - si la pregunta es de un documento, busca solo en él, y si ninguna palabra coincide, lee sus primeras páginas;
  - hace una llamada con el modelo más barato.
  - El código se queda solo con citas que están en un pasaje mostrado y con cifras que están en lo citado, con números o con palabras («dos mil dieciséis»).
  - Si ninguna cita pasa, la respuesta es «no lo dice». Sin documentos o sin pasajes, no llama a la IA.
- **El Asistente como orquestador** (oficio v7):
  - `documents_search` desaparece y la reemplaza `record_reader`, el Lector como herramienta: un solo nivel de delegación, porque el Lector no pregunta a nadie;
  - el Asistente recibe la respuesta revisada del Lector, nunca pasajes crudos, y solo puede citar los pasajes que el Lector le entregó.
- **El mapa en cada pregunta:**
  - los documentos del Expediente con una clave (`D1`…), su tipo y su estado (con su texto, leído de una imagen, sin archivo, falta, vigencia);
  - que los escaneos y las fotos ya están leídos como texto;
  - lo que falta y las sugerencias que esperan en cada ventana de «Mi institución».

  «¿Qué documentos tengo?» se contesta con el mapa, sin herramientas.
- **«Llenar con mis documentos» desde el Asistente** (decisión de la institución: un botón que la persona pulsa). La respuesta trae `from_documents` con una ventana o «todas», y el panel ofrece el botón, que corre el Lector de C2 en el clic.
- **Batería** (`fixtures/expediente/asistente.json`, 13 preguntas desde Inicio, Mi institución y Documentos). Prueba en seco: la página esperada está entre los pasajes del Lector en 7 de 7. Corridas reales con `gemini-3.5-flash-lite`:

  | Oficio | Resultado | Qué falló y cómo se corrigió |
  |---|---|---|
  | v5 | 11 de 13 | No nombró los documentos del mapa; adivinó un `id` del manual. El v6 pide nombrarlos y prohíbe inventar un `id` |
  | v6 | 11 de 13 | Para los extintores pidió la ficha en lugar del Lector. El v7 pide preguntarle al Lector antes de decir que no lo sabe |
  | v7, dos rondas | **26 de 26** | Herramienta correcta en todas, 16 de 16 citas con la página esperada, 0 inventados |

  - Costo del v7: unos $0.04 MXN por pregunta (2 llamadas del Asistente y 1 del Lector). Las 26 preguntas costaron $1.03 MXN, vectores incluidos.
  - Tiempo: de 1 a 7 s por pregunta. Algunas tardaron unos 45 s porque el programa espera para no pasar de 15 llamadas por minuto, lo que permite el plan gratuito de ese modelo; con una persona preguntando, no pasa.
  - Como el modelo barato no decide igual cada vez, la corrida real se repite en rondas (`CIMIENTO_ROUNDS`) y muestra cada paso: qué pidió el Asistente y qué preguntó y contestó el Lector.

- **Herramientas nativas contra nuestro protocolo JSON (§7), medido: se queda el protocolo JSON.**
  - Con `assistant_native_vs_json_live` (solo medición, `ai/gemini_native.rs`, no está en la app), la misma batería, el mismo agente, las mismas herramientas, el mismo tope y el escáner sobre lo que devuelve cada herramienta, con el mismo ritmo de llamadas por minuto, dos rondas y `gemini-3.5-flash-lite`.
  - En Gemini con `generateContent`, las funciones nativas van como `functionDeclarations` y la respuesta final como salida estructurada, que Gemini 3 permite junto con funciones. Los turnos del modelo se devuelven tal como llegaron, con su firma de pensamiento.

  | | Bien | Citas | Inventados | Llamadas del Asistente | Tokens de entrada | Costo del Asistente | Tiempo del modelo |
  |---|---|---|---|---|---|---|---|
  | **Protocolo JSON** | **26 de 26** | **16 de 16** | 0 | 48 | 89,397 | **$0.72 MXN** | 50.4 s |
  | Funciones nativas | 25 de 26 | 15 de 16 | 0 | 53 | 175,000 | $1.10 MXN | 48.9 s |

  - Las nativas eligen bien la herramienta igual (26 de 26), pero una vez no citaron el documento nombrado y gastan casi el doble de entrada, porque cada paso reenvía toda la conversación y las declaraciones de las funciones. Cuestan 52 % más y tardan lo mismo.
  - Nuestro protocolo además funciona igual con cualquier proveedor (ADR-034 §2). Se vuelve a medir si cambia el modelo o el proveedor; la prueba queda para eso.
  - Cambiar a un modelo más caro sería cambiar el costo por proceso, no la arquitectura ni el código (decisión de la institución).

### Precisiones al aplicar C4 · el Capturista (2026-10-15)

- **Su batería** (`fixtures/capturista.json`, `capturist_battery_live`): 12 casos en las cuatro ventanas. Cada caso trae lo que cuenta la persona y lo que debe quedar en cada campo, o «nada» cuando no lo dijo.
  - Hay trampas: una suma, «personas mayores» sin edad, un párrafo sin datos, cifras escritas con palabras, una conversación previa («Sí, por favor»), domicilio, teléfono y RFC.
  - En seco, el código tira las 10 respuestas tramposas de la batería.
- **El código revisa la cifra** (hallazgo en seco): el Capturista aceptaba una cifra con cualquier frase que dijo la persona; «60 años» pasaba con «personas mayores». Ahora, como el Lector, una cifra solo pasa si está en las palabras copiadas (`reader::fill::says_number`). En la corrida real atrapó ese mismo «60» y la suma de camas (20 + 15 = 35), que el modelo propuso en 4 de 4 rondas.
- **Las palabras para la persona son del programa, no de la IA** (hallazgo de la primera corrida). La IA escribía la nota antes de que el código revisara, y prometía lo que el código tiró: «le dejé sugerida la capacidad total sumando las camas», o «edad mínima y máxima» cuando solo propuso la mínima.
  - Con el oficio v2, la IA ya no escribe nota.
  - El código dice qué dejó, por los títulos del manual, y que lo que habría tenido que calcular o suponer no lo dejó. También dice qué puede contar todavía: los campos que aplican y están vacíos, sin sugerencia.
  - Es lo mismo que ya hacía «Llenar con mis documentos».
- **Medición** (`gemini-3.5-flash-lite`, dos rondas cada una):

  | Oficio | Resultado | Notas |
  |---|---|---|
  | v1 | 23 de 24 casos, 137 de 138 campos, 0 inventados | Copió mal una cita («sua casa») y el código la tiró; la nota prometió lo tirado |
  | v2 | 24 de 24, 138 de 138, 0 inventados | Tres veces dejó `answer` vacío cuando no había nada que sugerir, y el bucle volvió a preguntar |
  | v2 final | 23 de 24 casos, 136 de 138 campos, 0 inventados, 24 llamadas para 24 casos | Una vez no propuso «¿A quién atienden?»; el programa se lo pidió a la persona |

  - Con el v2 final salieron bien 68 de los 69 casos que el modelo contestó (una corrida perdió 3 por la cuota por minuto; ahora la batería espera 4 s entre casos).
  - Cuesta unos $0.017 MXN por caso, de 1.6 a 6 s.

### Precisiones al aplicar C4 · el Asistente y la prueba de la institución (2026-10-15)

La fase 2 atendió las tres preguntas que salían 1 de 2 y dos hallazgos de la prueba de la institución. Primero se midió una línea base con el oficio v8 (24 de 32, los dos casos nuevos 0 de 2) y cada causa se tomó de lo que contestó el modelo:

| Caso | Causa | Arreglo |
|---|---|---|
| El régimen fiscal sin cita | El Lector lo encontró, pero copió «Reg1menes» por «Regímenes» y la revisión exacta tiró la cita | Una sola revisión de citas (`core::reader::quote::find`) para C2, C3 y el Capturista: perdona un desliz de copia en citas de 12 letras o más (una letra de cada 20, a lo más 3) y las letras escritas como HTML; **nunca una cifra distinta**; guarda las palabras del texto, no la copia del modelo |
| «Con base en ese mismo» contestado de memoria | El modelo no siempre le pregunta al Lector | El código reconoce que la persona vuelve al documento de la respuesta anterior (`assistant::back_to`) y le pregunta al Lector antes que la IA |
| «Tenemos 35 espacios» se fue al poder | La frase de C3 que empujaba cada pregunta al documento de la respuesta anterior | Se quitó: volver a un documento lo decide el código por las palabras de la persona |
| «¿Qué documentos tengo?» con cifras y sin nombres | La IA resumía la lista | La lista la escribe el programa (`records`): tiene, faltan, en trámite, por vencer y vencidos; la IA solo pide la lista y dice una frase de entrada |
| «30 espacios y viven 12 niñas» sin «Llenarlo por mí» | El oficio ponía primero las preguntas y el mapa no decía qué lleva cada ventana | Oficio v9: primero ver qué hace la persona en su último mensaje; el mapa dice los campos de cada ventana |
| «¿Cuántos extintores?»: «Voy a revisar…» | El modelo pidió la herramienta y escribió una respuesta en el mismo paso, y el bucle tomaba la respuesta | El bucle de agentes corre la herramienta primero si es suya y le quedan pasos (todos los agentes) |
| El Capturista no tomaba «viven 12 niñas» ni «30 abuelitos» | La persona no usa las palabras de los campos | Capturista v3: pensar a qué campo va cada cosa dicha. Darle el «Qué poner» del manual se midió y empeoró (24 de 28) |

- **Medición** (`gemini-3.5-flash-lite`, dos rondas): Asistente v9 **32 de 32**, 18 de 18 citas, $1.21 MXN; Capturista v3 **28 de 28**, 154 de 154 campos, $0.52 MXN; «Llenar con mis documentos» **18 de 18** tras el cambio de la revisión de citas, $0.14 MXN. Nada inventado.
- La regla del Asistente sobre nombres sigue el ADR-040: no pide nombres de personas, pero sí dice el de alguien de la institución que el Lector encontró en un documento.
- Una fecha sola («2027-01-18») se mostraba un día antes en México (`formatDate` la leía como medianoche en Londres); se corrigió para toda la app.

### Aprobación de los agentes del núcleo (C4, fase 3, 2026-10-15)

Corrida final de las tres baterías con el modelo más barato (`gemini-3.5-flash-lite`), dos rondas cada una, con el código y los oficios con los que se queda el núcleo. Criterio: **nada inventado y al menos 90 % bien en dos rondas**.

| Agente | Batería | Resultado (2 rondas) | Inventado | Llamadas | Costo | ¿Aprobado? |
|---|---|---|---|---|---|---|
| Asistente (oficio v9) con el Lector como herramienta | `fixtures/expediente/asistente.json`, 16 preguntas desde varias pantallas | 32 de 32 (100 %); herramienta bien 32; citas 18 de 18 | 0 | 50 del Asistente, 18 del Lector, 19 de vectores | $1.17 MXN | **Sí** |
| Capturista (oficio v4) | `fixtures/capturista.json`, 15 casos en las cuatro ventanas | 28 de 30 casos (93 %); 186 de 188 campos | 0 | 30 | $0.55 MXN | **Sí** |
| Lector del Expediente, «Llenar con mis documentos» | `fixtures/expediente/lector.json`, las 4 ventanas | 35 de 36 campos (97 %); páginas 35 de 35 | 0 | 9 y los vectores de los fragmentos | $0.29 MXN | **Sí** |

- **Lo que faltó** (nunca un dato distinto):
  - el Capturista no tomó «desde dos mil diez» como año de fundación (1 de 2, igual que en la corrida anterior);
  - de «comida, tareas dirigidas y apoyo psicológico» dejó los apoyos sin la comida (1 de 2);
  - el Lector no sugirió la edad mínima en una ronda.
- **Un hueco del código encontrado en la primera corrida:** con «vienen voluntarios de una parroquia», el modelo propuso 1 voluntario y la revisión de cifras lo dejó pasar, porque leía «una» como 1. Ahora «un», «una» y «primer» solos no son una cifra (`reader::fill::says_number`, la misma para C2, C3 y el Capturista); «uno», «treinta y una» y «1» sí. La trampa quedó en la batería en seco. La corrida de la tabla es posterior al arreglo.
- **Ritmo:** las tres baterías nunca mandan más de 14 solicitudes por minuto a cada modelo (`ai::paced`; el plan gratuito permite 15). Toda la corrida: 107 llamadas al modelo de texto, unos 12 minutos y **$2.01 MXN**.

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
