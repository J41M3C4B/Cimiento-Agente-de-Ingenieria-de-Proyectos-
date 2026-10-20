---
adr: 47
estado: aceptada
fecha: 2026-10-19
tema: [nucleo, formularios]
reemplaza_en_parte: ["[[ADR-042-primer-inicio-solo-lo-indispensable]]"]
tags: [adr]
---
# ADR-047 · Un dato indispensable puede esperar a otro: el primer inicio en un núcleo genérico

**Estado:** aceptada (2026-10-19). El cambio salió del bloque [[ERP-2 El núcleo genérico|ERP-2]]: el líder lo probó y lo validó, y eligió escribirlo como un ADR corto. Cambia una regla del [ADR-042](../archivo/adr/ADR-042-primer-inicio-solo-lo-indispensable.md) §1; lo demás del 042 sigue igual.

## Contexto

El ADR-042 dice que el primer inicio pide solo los datos indispensables y que **cada uno se pregunta siempre, sin condición**. Una prueba lo exigía.

Con el núcleo genérico (ERP-2) eso ya no cabe. Las cifras de personas dependen de cómo atiende la institución:
- la **capacidad** y «¿Cuántas personas atienden hoy?» solo aplican donde las personas viven o pasan el día;
- «¿A cuántas personas atendieron en el último año?» aplica con las demás formas de atender;
- una institución mixta da las dos.

Son indispensables porque Inicio las muestra (criterio C del 042). Preguntarle la capacidad a una A.C. comunitaria es justo una pregunta ajena a ella, lo que ERP-2 quitó.

## Decisión

1. **Un dato indispensable se pregunta siempre, o solo cuando lo decide otro dato indispensable que se pregunta en un paso anterior.** Nunca espera una respuesta que llega después. La prueba `the_steps_are_the_indispensable_fields_of_the_catalog` lo revisa.
2. **Lo que no aplica no se pregunta ni se cuenta.** El primer inicio, las ventanas de «Mi institución» y el avance (`overview::completion`) cuentan solo lo que aplica a esa institución. El primer inicio se cierra cuando no falta ningún indispensable que aplique.
3. **La condición puede mirar otra ventana.** La capacidad está en «Capacidad y cifras», pero la decide «¿Cómo los atienden?», de «Su institución» (`profile::forms::context`).
4. **Si después cambia la respuesta que decide, el primer inicio no se vuelve a abrir.** Se cierra una sola vez (`onboarded_at`). Un dato que empieza a aplicar aparece como faltante en «Mi institución» y en el avance de Inicio.

## Consecuencias

- El primer inicio pide 15 datos indispensables, pero a cada institución solo los que le tocan. Una A.C. comunitaria no ve la capacidad; un asilo no ve la cifra del último año.
- Los criterios A, B y C del ADR-042 para decidir qué es indispensable no cambian. Un campo que decide lo que el programa pide sigue siendo indispensable.
- Una condición nueva sobre un indispensable debe mirar un dato de un paso anterior; si no, la prueba falla.
