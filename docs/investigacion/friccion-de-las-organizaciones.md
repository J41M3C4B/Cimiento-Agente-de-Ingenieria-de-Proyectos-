# Fricción de las organizaciones

Lo que se vio en las pruebas de SociAI hasta el 2026-10-20: las del líder con la institución de prueba y la validación con instituciones. Las citas son textuales. Las fuentes están en `docs/archivo/`:

| Fuente | Dónde |
|---|---|
| Observaciones del líder (problemas 1 a 8) | `archivo/notas/Observaciones Pruebas.md` |
| Resultados de los bloques IA-1, IA-2, ERP-1 y ERP-2 | `archivo/tablero/bloques/` |
| Validación con instituciones | `archivo/documentos/12-validacion-con-instituciones.md` |
| Visión | `archivo/documentos/00-vision.md` |

---

## 1. No saben por dónde empezar

- **Muchas formas de empezar** (problema 2): «Se complicó el inicio porque hay muchas formas de empezar, debemos definir **una sola forma de empezar, de llenar**… híbrido, donde el asistente global de la bienvenida nos otorgue una intro rápida… nos pida contarle sobre nosotros y al mismo tiempo subir los archivos automáticamente en el chat».
- **El aviso llevó a otro lado** (problema 1 y «Final»): el aviso de datos faltantes lo mandó al primer inicio y no a «Mi institución». «Ahí fue donde vi dónde llenar los datos resumidos y no en los módulos, **confusión mía**». Quedó «un residuo de que los módulos están apagados pero me sigue pidiendo registrar personal».
- **Tres puertas para lo mismo** (problema 7): el chat, «Llenarlo por mí» y «Llenar con mis documentos». El líder tuvo que pensar «cuándo sugerir rellenar con documentación y cuándo con lo que el usuario dice».

**Lectura:** la persona no distingue entre pantallas, ventanas, módulos y botones de IA. Cada puerta nueva le exige saber cómo funciona el programa por dentro.

## 2. Formularios pesados y sugerencias que no se distinguen

- **Problema 3:** «Se pierde la diferencia entre sugerencia y un botón normal de los formularios, además son **pesadísimos los formularios**, estamos violando la complejidad de un formulario al mostrar todo en un solo modal sin segmentar».
- **Problema 6:** después de subir todos los archivos, varios campos que podían llenarse quedaron sin sugerencia. Había que ir a la ventana y pulsar «Llenar con mis documentos».
- **Un clic por campo:** cada sugerencia pedía un «Usar», y las sugerencias se repartían entre cuatro ventanas, con un aviso por ventana.

## 3. Errores que bloquean o no se ven

- **Un PDF con contraseña** (problema 5): «tengo que hacer scroll para ver por qué dio error… una tarjeta en rojo abajo… dificulta al usuario y **puede bloquearlo**».
- **Un archivo en el lugar equivocado** (problema 4) se subía igual; solo se sugería otra cosa.
- **Ventanas que interrumpen:** la ventana de «datos personales» frenaba a una persona que no entendía por qué el programa le preguntaba, cada vez que el escáner creía ver un dato, aunque fuera el de su propia institución.
- **Estados que no avisan** (IA-1): tras el aviso del escáner, la frase de la persona se repetía, y el chat se quedaba en «Llenando…» sin haber dejado nada.
- **Datos revueltos** (ERP-2): los datos de un ejemplo aparecían en otro: «los números y datos se revuelven». Un Expediente «aparece como si hubiera archivos pero no tienen nada, pero sus estados marcan cosas».

## 4. El programa responde como robot y pide pasos de más

- **IA-1:** «Manejar la conversación menos cuadrada o robotizada y sonar más natural… el botón Llenarlo por mí solo aparece cuando de verdad aplica, **otra decisión que la IA debe hacer y no el código, mostrar botones**… se evita que el usuario haga una llamada al modelo que dará como respuesta que ya estaba guardado».
- **Dos llamadas para una respuesta** (IA-2, prueba 3): «Volvimos a tener el mismo problema de 2 llamadas». Se aplazó tres veces.
- **Respuestas cortas** (problema 7): la respuesta a «¿qué me falta?» «se queda algo corta… se limitó a 1 formulario cuando además podía ofrecerme las sugerencias de "Su institución"».
- **Validación con instituciones:** su única queja fue que las preguntas del chat del diagnóstico eran «algo cortantes», y no entendían por qué pesaba más la causa de fondo. «La herramienta no lo explica sola».

## 5. Se expresan de forma ambigua, y el programa no lo entiende

- **Problema 8:** «El nombre de la directora no es un nombre… la IA se comió una letra y además detectó el nombre de la directora como el de una calle».
- **Cómo escribe la gente:** en minúsculas en el chat, y en MAYÚSCULAS en las actas. El escáner con listas no reconoció completa a **ninguna** persona escrita así; en total, solo al 17 %.
- **El contexto se pega** (IA-2, prueba 4): el Asistente «se quedó en el contexto de la ventana de legales». La secuencia de mensajes confundió al modelo.
- **Palabras con dos sentidos** (ERP-1): a «¿Cuántos extintores tenemos?» contestó «puede revisar en sus instalaciones». Eso puede ser el edificio o la parte del programa, y además prometió una sección que no existe.
- **Aclarar no basta** (problema 7): el líder le aclaró al chat que «no son datos personales de nadie», y aun así recibió recomendaciones limitadas.

## 6. Esperas largas

IA-2: «hay más latencia… **24 s de media por respuesta**» con `gemini-3.5-flash-lite` (de 21 a 31 s ese día). En el diseño no había nada para la espera: ni respuesta por partes, ni otro modelo, ni un aviso.

## 7. No ven el beneficio, o lo ven en otro lado

- **Lo que sí valoraron las instituciones:** la **reducción de carga** y el «cerebro» que arma proyectos. Usaron el programa para **consultar las cifras de su institución**, y pidieron **empleados y beneficiarios**.
- **La visión** dice que «el tiempo de la dirección es lo más escaso», pero las pruebas pedían decenas de clics de «Usar» y revisar ventana por ventana.
- **El beneficio prometido, los proyectos, nunca llegó a sus manos.** Las instituciones pasaron semanas probando el llenado, que es la parte menos visible para ellas.

## 8. Lo que el líder espera del programa (en sus palabras)

- Una sola forma de empezar, conversando, con los papeles subidos en el mismo chat.
- Que la IA decida qué decir y qué botón mostrar, y que no lo decida el código.
- Que no se repita lo que ya está guardado.
- Formularios cortos y por partes, y errores a la vista.
- Nada de pasos ni llamadas inútiles.
