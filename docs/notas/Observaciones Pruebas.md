---
tipo: observacion
sobre: "[[16-institucion-de-prueba]]"
estado: abierta
fecha: 2026-10-16
---

## Mi observación
**Problema 1:** llenar los datos envia a onborading![[{4B23FC47-F5A3-4CFF-A0ED-6D4D3A390AC3}.png]]

Al entrar por primera vez el boton de la alerta de datos faltantes envia al onboarding y no a mi institucion. 
#### Problema 2: empty-state y su accesibilidad de inicio
![[{CC74EED4-1334-4380-8590-E61CFD952CC5}.png]]

Se complico el inicio por que hay muchas formas de empezar, debemos definir una sola forma de empezar, de llenar, o decidimos manual y rigido o hibrido, donde el asistente global de la bienvenida, nos otorgue una intro rapida de la UI/UX y finalmente nos pida contarle sobre nosotros y al mismo tiempo subir los archivos automáticamente en el chat y hacer que la mayor parte de la información indispensable quede cubierta.


#### Problema 3: Sugerencias
![[{6162323A-7F0E-40C5-9CCA-0C51D8CA1BC0}.png]]

Las sugerencias se van creando automaticamente con los archivos que se suben, eso funciona perfectamente pero ya tenemos el rezago de la UI, se pierde la diferencia entre sugerencia y un boton nornal de los formularios, ademas son pesadismos los formularios, estamos violando la complejidad de un formulario al mostrar todo en un solo modal sin segmentar correctamente. 


#### Problema 4: ingestion inteligente
![[{FF619BEF-D8D8-441B-8E11-9B2A58F94E9A}.png]]

El sistema de ingestion detecta que un archivo no corresponde a donde se sube pero solo sugiere acciones mas no evita la subida, esto podria causar errores tanto en la consulta manual del usuario como en la consulta de la IA. Hay que revisar esto para evitar casos edge. 

#### Problema 5: alertas dentro de los modales poco visibles e indicativos
![[{6FB3EB8A-0D32-4664-8371-DCCAAB1A47FB}.png]]

En este caso subi el archivo con contraseña pero el diseño del modal y los estados no permiten que se vean por que tengo que hacer scroll para ver  por que dio error o no se subio. Se alcanza a notar una tarjeta en rojo abajo pero eso dificulta al usuario y puede bloquearlo.

#### Problema 6: Sugerencias limitadas
Despues de  haber subido todos los archivos aun habian campos que podian haberse rellenado por sugerencia automatica pero no sucedio, quedaron varios campos sin sugerencia, se tiene que ir al formulario, dar clic en rellenar con mis documentos para rellenar los campos faltantes que aun pueden recibir sugerencias. Hay que revisar por que.

#### Problema 7: lectura de formularios incompleta y sugrencias limitadas
![[{88052376-4896-496B-A37D-6C952AB52AFA}.png]]

Se le pregunto a la IA que datos me faltaban por rellenar y funciona bien pero se queda algo corto, creo que aqui podriamos mostrar las opciones si de rellenar con documentos pero para cada seccion que encontro que se puede rellenar con los documentos actuales. Es decir la respuesta debe ser basada primero en la informacion que los documentos brindan y despues entonces sugerir los campos y formularios que pueden ser rellenados, en este caso se limito a 1 formulario cuando ademas podia ofrecerme las sugerencias de "Su institución"

![[{9C3A9900-54D3-4DD9-864B-5EAE566DF39E}.png]]

Al dar mas informacion sobre contacto y ubicacion lo hice mas complicado, no lo hice con llenarlo por mi, se lo solte directo en el chat normal y me hizo recomendaciones limitadas me ofrecio llenar con documentos o ir al formulario solamente. Si le indique que no son datos personales de nadie. Hay que revisar cuando sugerir rellenar con documentacion y cuando con lo que el usuario dice. En este caso creo que llenar con documentacion no debe estar disponible en el chat como tal ya que es redundante con el boton de los formularios, que ese es su lugar correcto. Usando llenarlo por mi funciona correctamente pero ese es su comportamiento esperado, sin embargo, si necesitamos revisar cuando enviamos al chat normal informacion que es relevante para los formularios, ademas de aplicar reglas comparativas donde si el campo ya fue rellenado y es la misma informacion nueva que llego al chat entonces no sugerir nada, por que seria redundante y podria confundir al usuario al hacerlo pensar que quiza no ha llenado ese campo cuando si ya fue llenado. 


#### Problema 8: detección de contexto sesgado por mala escritura
![[{0B1E6B26-E988-4B4F-8FE0-903658BAC180}.png]]

Tuvimos una confusion por redaccion, coom el nombre de la directora no es un nombre, hubo un detallito donde la IA se comio una letra y ademas detecto el nombre de la directora como el de una calle o no se que paso pero me sugirio que el dato deberia de ir en contacto y ubicacion. 


Final
![[{919856F7-037B-446B-AF27-90CED72DE4F9}.png]]
Se termino la prueba, se completaron todos los campos proporcionados, solo me solicita los campos de datos financieros, personal. Sin embargo haciendo clic en llearlos ahora me llevo al onboarding como se habia mencionado en el problema 1 y ahi fue donde vi donde llenar los datos resumidos y no en los modulos, confusion mia. 
![[{62B0CD57-99E3-476E-BC57-6D763D6FC584}.png]]

Quedo un residuo de que los modulos estan apagados pero me sigue pidiendo registrar personal y a quienes atendemos pero ya se entiende que ahorita no importa esto. 

## Respuesta

**2026-10-16.** Revisé las capturas y la columna «Resultado» del documento 16. De 39 datos, 33 quedaron bien. Los otros:

- **¿A qué se dedica?** El programa funcionó bien: el error es del guion. En el paso B escribiste «una casa de reposo para personas mayores», el Capturista lo usó y, como el campo ya estaba lleno, los documentos no lo tocan (ADR-038). El guion se corrige cuando cambie el primer inicio.
- **Servicios:** tomó el acta y no el informe. **Municipio:** salió en mayúsculas y sin acento. **Calle:** le dejó «Calle». Los tres van en la fase 5 del ADR-042.
- **Folio ante la Junta:** ya lo conocíamos. **Representante:** es el problema 8.

**La causa del problema 1:** el primer inicio tenía su propia lista de pasos, que pedía dinero e inmueble. Esos datos no están en ninguna ventana de «Mi institución». Por eso el aviso seguía diciendo «Quedaron 2 pasos» aunque llenaste las 4 ventanas.

**Lo decidido** (en [[ADR-042-primer-inicio-solo-lo-indispensable]], como propuesta):

- **Problemas 1 y 2:** el primer inicio pide solo lo indispensable (11 datos en 5 pasos cortos), sale de los formularios y tiene una sola cuenta de avance.
- **Problema 4:** un archivo solo se guarda en su lugar si el código confía en que es de ahí; si no, te pregunta.
- **Problemas 6 y 7:** las sugerencias llegan al subir, de lo que ese documento sí dice, y «Llenar con mis documentos» sale del chat.
- **Problemas 7 y 8:** «Llenarlo por mí» reparte los datos entre las ventanas a las que pertenecen, no sugiere lo que ya está igual, y el texto de lo que se dejó lo arma el código.
- **Problemas 3 y 5:** van en el bloque de diseño, después de este.

**2026-10-16, más tarde.** Revisaste la tabla: el año de fundación y el personal también entran. Quedan 14 datos.

**Fase 1 hecha (problemas 1 y 2):**
- el primer inicio pide solo esos 14 datos, en cinco pasos cortos, dibujados igual que en sus ventanas;
- se cierra solo con el último dato, venga de donde venga;
- el aviso de Inicio y la tarjeta de «Mi institución» cuentan lo mismo;
- dinero, personal, beneficiarios y espacios quedan en «Siguientes pasos».

Al probarlo encontré que «Benito Juárez» como alcaldía hacía saltar el escáner; ya no pasa.

Esta nota sigue abierta: faltan las fases 2 a 5 (problemas 4, 6, 7 y 8) y el bloque de diseño (problemas 3 y 5).

**2026-10-16, noche. Probaste el guion completo: todo bien. Fase 2 hecha (problema 4):**
- un archivo ya no se guarda donde no va: si sus palabras dicen que es otro documento, o nada lo reconoce, el programa pregunta antes de guardar nada («Guardarlo en «…»», «Sí es «…»» o «No guardarlo»);
- la confianza la cuenta el código, sin IA: las palabras del documento contra las de otro, y el nombre del archivo, que nunca le gana al texto;
- medido en seco con los 37 archivos: ningún archivo quedó en un lugar equivocado y solo pregunta por la foto sin texto;
- la medición encontró dos reglas que fallaban (la licencia «uso del suelo» y el registro desteñido «Juntal») y se corrigieron.

Para probarlo: [guion, sección 5](../15-guion-pruebas-nucleo.md), las dos casillas sin marcar. Siguen las fases 3 a 5 (problemas 6, 7 y 8) y el bloque de diseño (problemas 3 y 5).
