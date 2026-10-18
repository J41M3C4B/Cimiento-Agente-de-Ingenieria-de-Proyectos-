---
adr: 42
estado: aceptada
fecha: 2026-10-16
tema: [nucleo, documentos, ia]
reemplaza_en_parte: ["[[ADR-031-primer-inicio-y-datos-de-la-institucion]]", "[[ADR-038-politica-unica-de-llenado-desde-documentos]]", "[[ADR-036-cajones-de-documentos-y-un-agente-por-cajon]]"]
sobre: "[[Observaciones Pruebas]]"
tags: [adr]
---
# ADR-042 · Un solo camino para llenar la institución: el primer inicio pide solo lo indispensable

**Estado:** aceptada (2026-10-16, la institución revisó la tabla de §2). Fases 1 y 2 aplicadas (2026-10-16); fase 3, el 2026-10-17. Sale de la prueba del núcleo de la institución ([[Observaciones Pruebas]], problemas 1, 2, 4, 6, 7 y 8).

## Contexto

La prueba del núcleo salió bien en lo que se lee y se llena: 33 de 39 datos quedaron como debían. Lo que falló es **el camino**: hay seis formas de empezar y nada las ordena.

1. **El primer inicio pide datos que no viven en «Mi institución».** Sus 6 pasos (ADR-031) están en una lista aparte (`onboarding/domain.rs`) que pide dinero (Finanzas) e inmueble (Instalaciones). La persona llenó las 4 ventanas de «Mi institución» y el aviso de Inicio seguía diciendo «Quedaron 2 pasos»: eran esos dos, y «Llenarlos ahora» la llevó al primer inicio.
2. **Hay dos medidas del avance.** El aviso cuenta pasos del primer inicio; la tarjeta de «Mi institución» cuenta datos de las ventanas, más las fichas de Personal y Beneficiarios. Dicen cosas distintas de lo mismo.
3. **El primer inicio pide mucho de una vez:** unos 19 datos en 6 pasos, varios que los documentos ya traen (año de fundación, contacto) o que no hacen falta para usar la app (gasto, pisos del inmueble).
4. **Un archivo en el lugar equivocado se guarda** y el aviso sale en verde (ADR-038 §3).
5. **Lo que los documentos dicen no llega solo.** Al subir, las reglas leen los formatos fijos; lo demás (a qué se dedica, servicios, requisitos) solo llega si la persona pulsa «Llenar con mis documentos» en cada ventana.
6. **El chat se mezcla con los formularios:** ofrece «Llenar con mis documentos», manda los datos que la persona cuenta a una sola ventana elegida por el modelo (el nombre de la representante fue a «Contacto»), y el Asistente dijo «dejé 3 sugerencias» cuando no puede dejarlas.

## Decisión

### 1. Indispensable es lo que el programa necesita para funcionar, y lo decide el código

Un dato es **indispensable** si cumple al menos una de estas tres:

| Criterio | Por qué | Cómo se revisa |
|---|---|---|
| **A. Cambia lo que el programa pide o muestra** | Sin él, el Expediente pide los documentos equivocados, Beneficiarios no sabe qué ficha usar y Personal no sabe qué puestos ofrecer | Todo campo que aparece en una condición (`Condition` de un formulario, `Applies` del Expediente o `kind_of`) es indispensable. Una prueba lo exige |
| **B. Sin él la IA no sabe quién es la institución** | Es el encabezado de la ficha de la IA y el primer filtro de toda convocatoria | Lista corta y fija |
| **C. Se ve en Inicio** | Inicio no puede mostrar la ocupación sin él | Lista corta y fija |

- En el código es una marca nueva en el catálogo de campos (`FieldSpec::essential()`), aparte de `required`. `required` ya significa «cuenta en lo que falta», y eso no cambia.
- **El primer inicio sale de los formularios:** pide los campos marcados como indispensables y nada más. La lista aparte de `onboarding/domain.rs` desaparece. Así, todo lo que pide el primer inicio vive en una ventana de «Mi institución».
- **Un solo avance:** el primer inicio está completo cuando no falta ningún dato indispensable, y lo calcula `overview::completion`, lo mismo que lee la tarjeta de «Mi institución». El aviso de Inicio y la tarjeta salen de la misma cuenta.
- **El primer inicio se cierra solo** con el guardado que deja completo el último dato indispensable, venga del primer inicio o de una ventana de «Mi institución»: confirma el perfil y pone `onboarded_at`, como «Terminar y entrar». Así el aviso de Inicio nunca dice «faltan 0 datos».
- **El porcentaje de «Mi institución» cuenta solo datos de la institución** (sus ventanas y el patronato). Las fichas de Personal, Beneficiarios y los espacios se quedan en «Siguientes pasos» de Inicio, que es donde se piden.

### 2. Los campos, uno por uno

Revisa la última columna: «Sí», o lo que cambiarías.

| Ventana              | Campo                                            | ¿Indispensable?            | Criterio o quién lo llena después                                                                                       | ¿De acuerdo? |
| -------------------- | ------------------------------------------------ | -------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------ |
| Su institución       | Nombre                                           | **Sí**                     | B: todo lo usa                                                                                                          | Si           |
| Su institución       | ¿A qué se dedica?                                | **Sí**                     | B: proyectos e IA. Una frase basta                                                                                      | Si           |
| Su institución       | ¿A quién atienden?                               | **Sí**                     | A: decide el tipo (asilo, casa hogar), la ficha de Beneficiarios, los puestos de Personal y los documentos para menores | Si           |
| Su institución       | ¿Cómo los atienden?                              | **Sí**                     | A: decide si el Expediente pide aviso de funcionamiento y protección civil                                              | Si           |
| Su institución       | Mujeres, hombres o ambos                         | Después                    | Reglamento, con «Llenar con mis documentos»                                                                             | Si           |
| Su institución       | ¿En qué los apoyan?                              | Después                    | Acta e informe                                                                                                          | Si           |
| Su institución       | Servicios o programas                            | Después                    | Informe                                                                                                                 | Si           |
| Su institución       | Edad mínima y máxima                             | Después                    | Reglamento y acta                                                                                                       | Si           |
| Su institución       | Requisitos de ingreso                            | Después                    | Reglamento                                                                                                              | Si           |
| Contacto y ubicación | Estado                                           | **Sí**                     | B: primer filtro de las convocatorias                                                                                   | Si           |
| Contacto y ubicación | Municipio o alcaldía                             | **Sí**                     | B: igual                                                                                                                | Si           |
| Contacto y ubicación | Teléfono y correo                                | Después                    | Solo los usan los documentos de Word; «Llenarlo por mí»                                                                 | Si           |
| Contacto y ubicación | Calle, números, colonia, código postal           | Después                    | Constancia del SAT o «Llenarlo por mí»                                                                                  | Si           |
| Datos legales        | Figura jurídica                                  | **Sí**                     | A: decide si el Expediente pide el registro ante la Junta                                                               | Si           |
| Datos legales        | ¿Es donataria autorizada?                        | **Sí**                     | A: decide si pide el oficio y los datos de donataria                                                                    | Si           |
| Datos legales        | ¿Tiene CLUNI?                                    | **Sí**                     | A: decide si pide la clave CLUNI                                                                                        | Si           |
| Datos legales        | Nombre legal, RFC, régimen, código postal fiscal | Después                    | Constancia del SAT (regla, al subirla)                                                                                  | Si           |
| Datos legales        | Año de fundación                                 | **Sí**                     | B: muchas convocatorias piden «al menos N años operando» (decisión de la institución, 2026-10-16)                        | Si           |
| Datos legales        | Objeto social                                    | Después                    | Acta (regla, al subirla)                                                                                                | Si           |
| Datos legales        | Folio ante la Junta                              | Después                    | Registro ante la Junta                                                                                                  | Si           |
| Datos legales        | Rubro, número y fecha del oficio                 | Después                    | Oficio de donataria                                                                                                     | Si           |
| Datos legales        | Clave CLUNI                                      | Después                    | Constancia de la CLUNI                                                                                                  | Si           |
| Datos legales        | Representante y vigencia de su poder             | Después                    | Poder (vigencia) y «Llenarlo por mí» (nombre)                                                                           | Si           |
| Capacidad            | Capacidad total                                  | **Sí**                     | C: la ocupación de Inicio                                                                                               | Si           |
| Capacidad            | ¿Cuántas atienden hoy?                           | **Sí**                     | C: igual                                                                                                                | Si           |
| Capacidad            | Personal con sueldo y voluntario                 | **Sí**                     | B: la IA lo usa para los proyectos y no es una pregunta pesada (decisión de la institución, 2026-10-16); las fichas de Personal mandan | Si           |
| Capacidad            | Notas                                            | Después                    | Libre                                                                                                                   | Si           |
| Patronato            | Integrantes                                      | Después                    | A mano (datos protegidos)                                                                                               | Si           |
| Cuentas bancarias    | CLABE y uso                                      | Después                    | A mano; opcional                                                                                                        | Si           |
| Expediente           | Documentos                                       | Después                    | Se suben en el Expediente. Los campos «A» deciden cuáles pide                                                           | Si           |
| (Finanzas)           | Gasto anual e ingresos                           | **Sale del primer inicio** | Es del módulo de Finanzas, no de «Mi institución»                                                                       | Si           |
| (Instalaciones)      | Pisos y de quién es el inmueble                  | **Sale del primer inicio** | Es del módulo de Instalaciones                                                                                          | Si           |

Resultado: **14 datos** en lugar de unos 19, todos con su lugar en «Mi institución».

**Dudas resueltas (2026-10-16):** la propuesta dejaba para después el año de fundación y el personal con sueldo y voluntario. La institución los pidió en el primer inicio: el año porque las convocatorias lo piden, y el personal porque no es una pregunta pesada.
### 3. El primer inicio, progresivo

Sigue en pantalla completa antes de entrar a la app (ADR-031), con las mismas reglas de quién lo ve y quién puede dejarlo para después. Cambia lo que pide: una pantalla por grupo, pocos datos en cada una.

| Paso | Pide |
|---|---|
| 1. Quiénes son | Nombre; a qué se dedica |
| 2. A quién atienden | A quién; cómo |
| 3. Dónde están | Estado; municipio |
| 4. Cómo están constituidos | Figura jurídica; año de fundación; donataria; CLUNI |
| 5. Cuántos | Capacidad; cuántas atienden hoy; cuántas trabajan con sueldo y cuántas como voluntarias (0 es respuesta; con fichas en Personal o Beneficiarios no se pregunta) |

- Los pasos y su orden viven en `core::onboarding`. Cada campo lo describe el catálogo de formularios: la misma etiqueta, las mismas opciones y la misma validación que en su ventana.
- Una prueba exige que cada campo indispensable esté en un paso, y solo en uno.
- Al terminar, Inicio muestra «lo que falta», que lleva a cada ventana, donde están «Llenar con mis documentos» y «Llenarlo por mí».
- Una instalación que ya terminó el primer inicio no lo vuelve a ver (`onboarded_at`). Una que no lo terminó ve solo los pasos nuevos que le falten.
- El diseño de estas pantallas va en el bloque de diseño, no en este.

### 4. Un archivo solo se guarda en su lugar si el código confía en que es de ahí

Reemplaza a ADR-038 §3. Sin IA: el código ya reconoce el tipo de un documento por sus palabras (`reader::kinds`), y ahora cuenta además qué tan seguro está.

| Lo que ve el código | Qué pasa |
|---|---|
| Sus palabras dicen que es el documento de ese lugar, y ningún otro tipo le gana | Se guarda, se lee y llena como hoy |
| Sus palabras dicen que es **otro** documento, no el de ese lugar | **No se guarda ahí.** Pregunta: «Este archivo parece «Programa de protección civil». ¿Lo guardo ahí?», con tres botones: guardarlo en su lugar, «Sí es este documento» o cancelar |
| No se reconoce ningún tipo (imagen ilegible, formato raro) | Confianza baja: se pregunta «No pudimos confirmar que sea «Acta constitutiva». ¿Lo es?». El nombre del archivo puede subir la confianza si dice el nombre del documento |

- **La confianza** es una cuenta del código: las frases del tipo de ese lugar halladas en el archivo, contra las del mejor de los otros tipos, más el nombre del archivo.
- **El umbral se mide, no se adivina:** se corre en seco con los 24 archivos de la prueba completa y los 13 de la batería. Se elige el que no deje ningún archivo en un lugar equivocado y pregunte lo menos posible. La medición se anota en este ADR.
- Si la persona dice «Sí es este documento», se guarda, se lee con las reglas de ese lugar y la decisión queda en la bitácora.
- El aviso de un archivo puesto en otro lugar nunca es verde.

### 5. Las sugerencias llegan al subir, de lo que ese documento sí dice

Reemplaza en parte a ADR-036 y a la metodología de los Lectores §3 («la capa con IA, solo cuando la persona la pide»).

1. Al terminar de leer un archivo, las reglas hacen lo de hoy.
2. Después, el código busca qué **campos vacíos** de «Mi institución» tienen un pasaje de **ese documento** (palabras del campo y vectores, como `fill`). Esta búsqueda no usa IA.
3. Si encuentra alguno, hace **una llamada** con el modelo más barato, solo con esos campos y esos pasajes, y la misma revisión de hoy: la cita, la cifra y el valor. Lo que pasa queda como sugerencia, con su documento y su página, bajo la política de ADR-038.
4. Sin IA o sin campos vacíos, no se llama. A lo más una llamada por archivo.
5. «Llenar con mis documentos» sigue en cada ventana, para volver a intentarlo con todo el Expediente.

Se mide primero en seco con la prueba completa: por archivo, qué campos detecta contra los esperados del documento 16, §4. Después, una corrida real acotada.

### 6. El chat no llena con documentos y no se contradice

- **«Llenar con mis documentos» sale del chat.** Su lugar es la ventana.
- **Cuando la persona cuenta datos en el chat,** «Llenarlo por mí» los reparte entre las ventanas a las que pertenecen. Hoy el modelo elige una sola ventana; con este cambio, el nombre de la representante va a «Datos legales» aunque la persona esté en «Contacto».
- **Lo que ya está guardado igual no se sugiere:** si lo que la persona cuenta es lo mismo que ya tiene el campo, después de normalizar mayúsculas, acentos y espacios, la sugerencia se descarta y la respuesta dice que ya estaba.
- **El Asistente nunca dice que hizo algo que no puede hacer.** El texto de lo que se dejó lo arma el código con lo que de verdad quedó.
- **Letras cambiadas por números** («dej3», «bot3n»): se revisa si es el modelo o el camino del texto, y se corrige donde esté la causa.

## Consecuencias

- **Primer inicio:** 14 datos en 5 pasos cortos. Dinero e inmueble salen de él y quedan en sus módulos.
- **Una sola fuente del avance:** el aviso de Inicio, la tarjeta y la lista de «lo que falta» ya no se contradicen.
- **Documentos:** ningún archivo queda en el lugar equivocado sin que la persona lo diga.
- **Costo:** a lo más una llamada barata por archivo subido que hable de campos vacíos. La corrida real lo anota en `07-ia-y-costos.md`.
- **Pruebas:**
  - `onboarding`: pasos desde el catálogo; cada indispensable en un paso; todo campo de condición es indispensable.
  - `overview`: una sola cuenta; el porcentaje sin fichas de módulos.
  - `core::reader::kinds`: confianza y umbral medido.
  - `core::reader::fill`: detección al subir, en seco.
  - `core::agents::capturist`: repartir entre ventanas; lo igual no se sugiere.
- **Documentos que cambian:** ADR-031 (pasos), ADR-038 §3, metodología de los Lectores §1 y §3, el guion del núcleo (15) y la institución de prueba (16).

## Plan por fases (una a la vez; pruebas de la institución entre fases)

1. **Indispensables y un solo avance** (hecha, 2026-10-16): la marca `essential`, la prueba de los campos de condición, los pasos desde el catálogo y `completion` como única cuenta. Ver «Fase 1, al aplicarla».
2. **Archivo en su lugar** (hecha, 2026-10-16): primero la medición en seco y el umbral; después la pregunta. Ver «Fase 2, al aplicarla».
3. **Sugerencias al subir** (hecha, 2026-10-17): la detección en seco, la llamada por archivo y una corrida real. Ver «Fase 3, al aplicarla».
4. **Chat:** sin «Llenar con mis documentos»; «Llenarlo por mí» reparte entre ventanas; lo igual no se sugiere; el texto de lo dejado lo arma el código; las letras cambiadas.
5. **Detalles de lectura de la prueba:** el municipio con su nombre del catálogo (con acentos, sin mayúsculas sostenidas), «Calle» fuera de la calle, y los servicios desde el informe y no desde el acta.

Después de este ADR va el bloque de diseño: formularios por secciones, sugerencias distintas de un botón y avisos a la vista (problemas 3 y 5).

## Fase 1, al aplicarla (2026-10-16)

- **El catálogo manda.** `FieldSpec::essential()` marca los 14 campos (y los hace `required`). Los pasos (`onboarding::domain::STEPS`) son listas de esos campos, cada paso dentro de una sola ventana. Las pruebas exigen que los pasos y el catálogo digan lo mismo, que cada uno se pregunte una sola vez y siempre (sin condición), y que todo campo del que dependa una condición de un formulario, del Expediente (`Applies::field`) o el tipo de institución sea indispensable.
- **Cada paso se guarda por su ventana** (`profile::forms::save`): las mismas reglas, el escáner y el historial. Solo cambian los campos del paso; lo demás de la ventana se queda. La pantalla dibuja cada paso con el mismo componente que las ventanas (`FormRenderer`), con las mismas palabras, opciones y «?».
- **Se cierra solo.** El guardado que deja completo el último dato indispensable, venga del primer inicio o de una ventana, confirma el perfil y pone `onboarded_at`. «Terminar y entrar» queda para una institución que ya tenía todo antes de este cambio.
- **Las fichas de los módulos mandan** como antes: con personas en Beneficiarios o en Personal, su cifra rápida no se pregunta ni falta en su ventana.
- **Una sola cuenta.** `overview::completion` toma los campos obligatorios vacíos de las ventanas y el patronato; los indispensables van primero. Las fichas de Personal y Beneficiarios, los espacios y el dinero ya no cuentan ahí: Inicio los pide en «Siguientes pasos», que ahora incluye Finanzas. El aviso de Inicio cuenta datos, no pasos.
- **Hallazgo al probar en la app:** al escribir «Benito Juárez» como alcaldía, el escáner lo tomó por el nombre de una persona. Muchos municipios se llaman como una persona. El municipio ya no pasa por el escáner: una regla del código (`Rule::PlaceName`) solo acepta letras y los signos de un nombre, hasta 80, así que no cabe un teléfono, un correo ni una clave.
- **Pruebas:** `cargo test onboarding` (13), `overview`, `profile::forms` y `common::forms` (nombre de lugar); Vitest del primer inicio reescrito. Todas pasan.

## Fase 2, al aplicarla (2026-10-16)

- **La cuenta** (`core::reader::kinds::confidence`): las frases del tipo de ese lugar halladas en el archivo (3 si están en su encabezado, 1 más adelante), menos las del mejor de los otros tipos, más **1** si el nombre del archivo dice ese documento. Cada tipo tiene sus frases para el nombre del archivo («constancia» y «fiscal», «cluni», «registro» y «junta» o «jap»…).
- **La decisión** (`kinds::placement`): con 1 o más, se guarda como hoy. Si no, y las palabras dicen otro tipo, pregunta «Este archivo parece «…»» con «Guardarlo en «…»», «Sí es «…»» y «No guardarlo». Si nada lo reconoce, pregunta «No pudimos confirmar que sea «…»» con «Sí es «…»» y «No guardarlo». Un «Otro documento» nunca pregunta: es lo que la persona dice.
- **Mientras pregunta, nada se escribe:** ni el documento como «Lo tenemos», ni su texto, ni sugerencias. El archivo ya leído espera en memoria (`core::records::placing`), así que la respuesta no lo vuelve a leer con OCR. Cerrar la pregunta es «No guardarlo». Si la persona responde con la ventana cerrada, la pregunta sale donde esté.
- **«Guardarlo en «…»»** lo pone en el documento que dicen sus palabras, leído con sus reglas; si ese ya tenía archivo, la pregunta avisa que lo reemplaza. **«Sí es «…»»** lo guarda donde se subió, leído con las reglas de ese lugar, y la bitácora anota «Dijo que un archivo es el documento donde lo subió, aunque parecía otro» con los dos tipos, nunca el archivo.

**La medición** (`cargo test kind_confidence_measure -- --ignored --nocapture`, en seco): los 20 archivos de la prueba completa que se pueden abrir y los 13 de la batería, cada uno puesto en su lugar (31 tienen tipo) y en cada uno de los otros 17 (563 lugares equivocados).

| Peso del nombre / umbral | En un lugar equivocado | Preguntas en su lugar |
|---|---|---|
| 0 / 1 | 0 | 1 |
| 0 / 2 | 0 | 3 |
| **1 / 1 (el elegido)** | **0** | **1** |
| 1 / 2 | 0 | 1 |
| 1 / 3 | 0 | 3 |
| 2 o 3 / cualquiera | 0 | 1 |

- La única pregunta en su lugar es la foto sin texto (`20-foto-sin-texto.jpg`): está bien que pregunte.
- **Se eligió 1 / 1:** el umbral más bajo y el peso más chico que preguntan lo menos. Con peso 1 el nombre vale lo que una frase: decide solo cuando el texto no dice nada, y nunca le gana al texto de otro tipo (la constancia llamada «acta-constitutiva-version-final.pdf» se pregunta). Con 0 da igual en estos archivos, pero un escaneo ilegible llamado «Registro_JAP.png» preguntaría sin necesidad.
- **La primera medición encontró dos causas**, y se corrigieron en las reglas, no en el umbral: la licencia dice «uso **del** suelo» y la regla buscaba «uso de suelo»; y el OCR de la copia desteñida lee «Junta**l** de asistencia privada», así que las frases de la Junta buscan «junta» y «asistencia privada» por separado. Antes de eso, 3 archivos legítimos preguntaban.
- **Pruebas:** `cargo test core::reader::kinds` (cada documento de la batería solo se guarda en su lugar; el nombre nunca le gana al texto), `core::reader` (la pregunta y sus tres respuestas, sin escribir nada antes; lo que nada reconoce), `full_expediente` (el archivo 19 se pregunta y se cancela; la foto sin texto se pregunta y se guarda) y Vitest de la ventana del documento.

## Fase 3, al aplicarla (2026-10-17)

- **Qué se pregunta** (`core::reader::upload::asked`): en las cuatro ventanas, los campos vacíos que la IA puede ver, menos dos:
  - «Notas generales», que es «lo que quiera que se sepa de la institución»: lo dice la persona, no un archivo;
  - un campo que ya espera una sugerencia con tanta autoridad como este archivo, o más. Las reglas del mismo archivo pueden acabar de dejar una. Un archivo con más autoridad sí se pregunta, y su sugerencia toma el lugar de la otra (ADR-038).
- **De qué campos habla el archivo** (`upload::detect`, sin IA): un pedazo del archivo habla de un campo si trae **2 palabras distintas** del campo o más (las de `fill::SEARCH` y su título del manual), o si nombra una de sus opciones con palabras propias («Ciudad de México», «estancia permanente»; un nombre corto como «Sí» o «No» no cuenta).
  - Por campo van los 3 pedazos que más hablan de él. Cada pedazo va una sola vez.
- **Solo por palabras:** al guardar el archivo, sus vectores todavía no existen (se hacen después, en segundo plano). «Llenar con mis documentos» sigue buscando en todo el Expediente, por palabras y por significado.
- **Una llamada por archivo**, con el Lector y el modelo más barato. Su oficio pasó a `reader.v2.md`, que sirve para una ventana o para varias. La revisión es la misma de C2 (`fill::keep`): la cita, la cifra y el valor, y cada campo va a su ventana.
- **La pantalla:** al terminar, el evento `record-suggested` dice qué quedó. Si quedó algo, un aviso dice cuántos datos y en qué ventanas esperan. Si la ayuda automática falló, el aviso lo dice y ofrece «Llenar con mis documentos». Si no encontró nada, no dice nada.

**La medición en seco** (`cargo test upload_detection_measure -- --ignored --nocapture`): los 13 documentos ficticios contra los 39 pares archivo-campo que `lector.json` dice que cada uno escribe.

| Umbral | Pares hallados | Preguntados de más | Campos por archivo |
|---|---|---|---|
| 1 | 39 de 39 | 160 | 15.3 |
| **2 (el elegido)** | **35 de 39** | **56** | **7.0** |
| 3 | 27 de 39 | 19 | 3.5 |
| 4 | 16 de 39 | 6 | 1.7 |

- **Los 4 que faltan con 2 son el nombre y «a qué se dedica»**: los pide el primer inicio, así que nunca están vacíos cuando se sube un archivo. Con 2 se hallan todos los campos que pueden faltar. Una prueba lo exige.
- Un campo de más solo alarga la llamada: el código tira lo que no cita bien.
- **La primera medición encontró tres causas,** y se corrigieron en las palabras, no en el umbral:
  - el estado nunca se hallaba: los documentos dicen «Ciudad de México», no «estado». Ahora cuenta el nombre de una opción;
  - la edad mínima se dice «podrán ingresar… de sesenta años»: su búsqueda lleva «ingresar»;
  - a qué se dedica se dice como «giro» o «atendemos»: su búsqueda lleva «giro», «dedica» y «atend».
- **En la prueba completa** (`full_expediente`, en seco, los archivos por el camino de la pantalla): cada archivo guardado preguntaría de 0 a 10 campos. Lo demás sigue igual: nada distinto de lo esperado.

**La corrida real** (`cargo test upload_live -- --ignored --nocapture`, Gemini `gemini-3.5-flash-lite`). Los 13 documentos ficticios se subieron uno tras otro, con solo el nombre escrito y las reglas de cada archivo antes:

| Corrida | Bien | Inventado | Páginas | Llamadas | Costo |
|---|---|---|---|---|---|
| Primera | 18 de 18 | 0 | 17 de 18 | 10 | $0.17 MXN |
| Después de los arreglos | 18 de 18 | 0 | 17 de 18 | 9 | $0.17 MXN |

- Cada llamada tarda de 0.6 a 1.8 s. Cuatro archivos no llamaron: no hablan de ningún campo vacío.
- **Lo que encontró la primera corrida:**
  - «Notas generales» recibió el título del poder: ya no se pregunta al subir;
  - «Servicios» se quedó con «Casa de reposo», de la opinión del SAT, porque una sugerencia en espera bloqueaba el campo aunque llegara un archivo con más autoridad. Ahora no lo bloquea, y en los servicios mandan el informe anual, el reglamento y el acta (`reader::authority`).
- **«Llenar con mis documentos» con el oficio nuevo** (`reader_live`): 18 de 18, páginas 18 de 18, nada inventado, $0.14 MXN. No empeoró.
- **El caché del proveedor:** en las 19 llamadas, 0 tokens salieron del caché. El oficio va primero y no cambia, pero es corto: no alcanza el mínimo que Gemini pide para guardar en caché. Hacer más larga la parte fija (por ejemplo, con el catálogo completo de campos) haría cada llamada más cara a cambio de un descuento que no está medido. Se mide antes de cambiarlo.
- **Pendiente para la fase 5:** el informe no dio los servicios (el código tiró sus tres sugerencias).
- La nota que la IA escribe al llenar («Dejé sugeridas las cifras de personas atendidas…») puede nombrar un dato que el código tiró, pero la pantalla nunca la muestra: el aviso lo arma el programa con lo que quedó.
- **Pruebas:** `cargo test core::reader::upload` (cada documento se pregunta por los campos que escribe; el nombre de una opción; una llamada por archivo, solo con sus pasajes y sin el nombre del archivo; un campo que espera no se vuelve a preguntar, salvo a un archivo con más autoridad; sin IA o sin campos, no se llama) y Vitest del aviso.

## Alternativas descartadas

- **Un chat de bienvenida que llena todo con archivos:** se pensó primero. Se descartó por ahora: la app ya tiene el botón por ventana, el Capturista y el Asistente. Lo que faltaba era un primer inicio corto y un solo camino después.
- **Escribir la lista de indispensables a mano:** es lo que había, y se desfasó de los formularios. La regla «todo campo de condición es indispensable» la revisa una prueba.
- **Que la IA decida dónde va un archivo:** el código ya reconoce los tipos por sus palabras. Cuando no está seguro, decide la persona.
