---
tipo: bloque
id: IA-4
carril: ia
estado: backlog
prioridad: alta
fase: MVP-2
rama:
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
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

**Notas del CTO para definirlo:**
- Hoy las frases del botón y de lo que quedó sugerido las escribe el programa (decisión del ADR-042 archivado): la IA prometía cosas que el código descartaba. Darle la palabra a la IA pide un límite nuevo: la IA habla y decide, pero **solo puede decir que algo quedó guardado o sugerido si el código lo confirma**. Va en un ADR nuevo.
- Para saber si el dato ya está guardado sin otra llamada, el Asistente necesita ver lo guardado de los campos que no son protegidos, como ya lo ve el Capturista.
- Conviene hacerlo después de la bóveda (IA-2), o junto con ella, porque los dos cambian lo que el Asistente recibe y cómo responde.
- Se mide con las baterías del Asistente y del Capturista con el modelo más barato, más casos nuevos: «un dato que ya estaba», «uno nuevo», «uno a medias».

**Anotado desde ERP-1 (2026-10-19, observación del líder):** a «¿Cuántos extintores tenemos?», sin el dato en los documentos, el Asistente contestó: «Puede revisar en sus instalaciones o anotar ese dato cuando agreguemos la sección correspondiente».
- No es un error, pero es ambiguo: «instalaciones» puede ser el edificio o la parte del programa (sin «» no se sabe cuál).
- Además promete una parte que todavía no existe, cuando nadie preguntó dónde anotarlo.
- Entra en la revisión de cómo habla el Asistente: cuándo nombra una parte del programa, siempre entre «», y que no ofrezca lo que todavía no existe si la persona no lo pidió.

## Terminado cuando

(Por definir.)

- [ ] Un dato ya guardado se dice en la misma respuesta, sin botón ni segunda llamada.
- [ ] El botón aparece solo cuando hay algo nuevo que sugerir.
- [ ] Nada dice «guardado» o «sugerido» si el código no lo confirma.

## Plan

## Cómo probarlo

## Resultado
