---
tipo: observacion
sobre: "[[ADR-036-cajones-de-documentos-y-un-agente-por-cajon]]"
estado: resuelta
fecha: 2026-10-12
---

## Mi observación

Por el momento debemos dejar de lado los modulos, documentos, Institucion e inicio son lo unico, lo demas por el momento queda fuera del proyecto. 
Razón: si nos ponemos a trabajar en todo de un jalon entonces vamos a perder mucho tiempo iterando en reparar o solucionar problemas de otros modulos que no estamos viendo, el objetivo es conseguir una metodologia de implementacion replicable en los demas modulos que nos facilite el trabajo. Entonces con el Core es lo que vamos a hacer, si encontramos el camino correcto ya de dejar los agentes (Asistente, Lector, etc)
del core, entonces replicar el procedimiento para cada modulo sera lo mas optimo, entonces ahorita debemos tratar a los demas modulos como si no existieran para conseguir esto, una metodologia replicable para toda la arquitectura de cada modulo y no ajustarnos a lo que los modulos ya tienen por que al final vamos a terminar arreglando todo. Mejor creamos la base y los modulos se construyen a medida del core.
## Respuesta

De acuerdo (Claude, 2026-10-12, al empezar C1). Qué cambia:

- **El proyecto, por ahora, es solo el núcleo:** Inicio, Mi institución y Documentos. Personal, Beneficiarios, Instalaciones, Finanzas y Proyectos se tratan como si no existieran: no se adapta nada del núcleo a lo que ellos tienen hoy, ni se arregla nada de ellos.
- **Lo que se busca es la metodología:** el cajón, su agente (Asistente, Lector), su batería con el modelo más barato y tu prueba. Cuando funcione en el núcleo, los módulos se rehacen a la medida del núcleo, uno por uno.
- **En C1** la bandeja de Documentos solo tiene el cajón del Expediente. La columna «De los donantes» sale de la pantalla mientras no exista ese cajón. Las convocatorias de Proyectos quedan en su propio lugar y no se tocan.
- Quedó escrito en el [[ADR-036-cajones-de-documentos-y-un-agente-por-cajon|ADR-036]], «Orden de trabajo».
