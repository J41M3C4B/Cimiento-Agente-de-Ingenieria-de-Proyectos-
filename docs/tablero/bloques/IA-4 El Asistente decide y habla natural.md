---
tipo: bloque
id: IA-4
carril: ia
estado: backlog
prioridad: alta
fase: MVP-2
patron: [acotada, agente]
rama:
adr: "[[ADR-045-tres-patrones-de-la-ia]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

Salió de la prueba de IA-1 (observación del líder). Cuando la persona cuenta un dato que ya está guardado («Estamos ubicados en la alcaldía Benito Juárez»), hoy pasan dos cosas:
1. El Asistente ofrece «Llenarlo por mí».
2. Al pulsarlo, el Capturista hace otra llamada a la IA solo para decir «Ya estaba guardado igual».

Esa segunda llamada es inútil y la conversación suena robótica: cuadrada, con frases fijas del programa.

Lo que se quiere:
- **Si el dato ya está cubierto, se dice en la misma respuesta** y se invita a contar otro, sin botón ni segunda llamada.
- **El botón «Llenarlo por mí» aparece solo cuando de verdad aplica,** y lo decide la IA, no una regla del código.
- **Un lenguaje más natural:** que la IA decida cuándo decir todo en un mensaje y cuándo en dos.

**Con los tres patrones ([[ADR-045-tres-patrones-de-la-ia]], 2026-10-19)** el problema es de forma, no de redacción:
- **Una sola decisión está partida en dos llamadas unidas por un botón.** El Asistente decide «son datos» sin ver lo guardado, y el Capturista lo descubre en otra llamada.
- **El Asistente enruta en su oficio:** cinco caminos escritos como casos en el prompt (v11), cada falla agrega uno. Esos caminos se conocen, así que no hace falta un agente para elegirlos: es el patrón 2.
- **Hay remiendos en el código** que vienen de esa forma: `BACK_TO_IT` («ese mismo»), `claims_done` (borra un «guardé» del modelo) y `mend_digits` («dej3»).

**Propuesta del CTO** (se decide al definir «Terminado cuando»):
1. **Un operador que enruta** (patrón 2) recibe el mensaje y devuelve una intención cerrada: cuenta datos, pregunta del programa, de la institución, de un documento, por sus documentos, o abierta. El código despacha cada intención a su camino, cada uno con un oficio corto. «Ese mismo» lo resuelve este operador, no una lista de frases.
2. **«Cuenta datos» va directo al Capturista** (patrón 2), que ya ve lo guardado. El código sabe qué es nuevo, qué ya estaba y qué no cabe. La respuesta lo dice de una vez y solo afirma lo que el código confirmó.
3. **El agente (patrón 3) queda solo para lo abierto:** preguntas que mezclan la ficha, el manual y los documentos, donde qué leer después depende de lo leído. Con sus mismas herramientas y topes.
4. **La medición decide:** las baterías del Asistente y del Capturista con el modelo más barato, más casos nuevos: «un dato que ya estaba», «uno nuevo», «uno a medias». Se compara contra el v11 en aciertos, llamadas, costo y tiempo.

**Pregunta para el líder:** si el Capturista corre solo cuando la persona cuenta datos, las sugerencias quedan esperando en su ventana sin pulsar nada (nada se guarda sin el clic de «Usar»). ¿Se queda «Llenarlo por mí» como permiso para hacerlo, o el botón pasa a ser «Ver sugerencias», que lleva a la ventana?

Conviene hacerlo después de la bóveda (IA-2), o junto con ella, porque los dos cambian lo que el Asistente recibe y cómo responde.

**Anotado desde ERP-1 (2026-10-19, observación del líder):** a «¿Cuántos extintores tenemos?», sin el dato en los documentos, el Asistente contestó: «Puede revisar en sus instalaciones o anotar ese dato cuando agreguemos la sección correspondiente».
- No es un error, pero es ambiguo: «instalaciones» puede ser el edificio o la parte del programa (sin «» no se sabe cuál).
- Además promete una parte que todavía no existe, cuando nadie preguntó dónde anotarlo.
- Entra en la revisión de cómo habla el Asistente: cuándo nombra una parte del programa, siempre entre «», y que no ofrezca lo que todavía no existe si la persona no lo pidió.

## Terminado cuando

(Por definir.)

- [ ] Un dato ya guardado se dice en la misma respuesta, sin botón ni segunda llamada.
- [ ] El botón aparece solo cuando hay algo nuevo que sugerir.
- [ ] Nada dice «guardado» o «sugerido» si el código no lo confirma.
- [ ] El oficio del Asistente ya no enumera caminos, y `BACK_TO_IT` y `claims_done` desaparecen.
- [ ] Las baterías del Asistente y del Capturista con el modelo más barato no empeoran contra el v11, en aciertos, inventados ni costo.

## Plan

## Cómo probarlo

## Resultado
