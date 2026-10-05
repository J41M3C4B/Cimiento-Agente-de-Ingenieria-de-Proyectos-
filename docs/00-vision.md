# 00 · Visión

## El problema

Las instituciones de asistencia privada tienen acceso a convocatorias de donativos (JAP, Nacional Monte de Piedad, fundaciones), pero:

- **Enmarcan mal el proyecto.** Piden "remodelar un baño" cuando el proyecto real es "reducir el riesgo de caídas y dar autonomía a los adultos mayores" o "un espacio digno y adecuado para las niñas". El donante financia impacto, no obras.
- **No priorizan.** No hay un análisis que diga si esa es la necesidad más importante o la que mejor encaja en la convocatoria.
- **Pierden tiempo con formatos.** Cuestionarios en Excel con fórmulas y celdas combinadas, formatos Word rígidos.
- **Tienen la información dispersa.** Cada convocatoria empieza desde cero.

Además, en un asilo pocas personas atienden a muchos adultos mayores con alta dependencia. El tiempo de los directivos es lo más escaso.

## La solución

Una app de escritorio que:

1. Guarda el **perfil de la institución** una vez (población atendida en cifras, personal, instalaciones, finanzas).
2. Hace un **diagnóstico guiado**: la IA pregunta como lo haría un buen analista y ayuda a convertir "un baño" en un problema bien planteado, con alternativas y prioridades.
3. Lee las **convocatorias** y revisa si el proyecto cumple los requisitos.
4. **Redacta el proyecto** en Word con la plantilla del donante.
5. **Llena los cuestionarios** de Excel sin romper el formato y sin errores de cálculo.

Efecto indirecto buscado: que el diagnóstico mejore la gestión general de la institución, no solo los proyectos.

## Para quién

| Persona | Necesita | No tolera |
|---|---|---|
| Directora / madre superiora | Respuestas rápidas, decidir con claridad | Jerga técnica, pantallas confusas |
| Personal administrativo | Llenar formatos sin errores | Perder trabajo, rehacer cálculos |
| Desarrollador (responsable) | Mantener e instalar la app fácilmente | Complejidad innecesaria |

## Qué NO es

- No es un expediente de residentes ni de niñas. No guarda datos personales.
- No envía solicitudes por sí misma. Prepara; las personas envían.
- No reemplaza el criterio de los directivos.

## Cómo sabremos que funciona

- El caso del baño (`fixtures/caso-dorado-bano.md`) termina replanteado como lo haría un analista.
- Un proyecto completo se prepara en una fracción del tiempo actual.
- Los cuestionarios de Excel salen sin errores de formato ni de cálculo.
- El costo de IA por proyecto se mantiene en pocos pesos.
- Cero datos personales en la base de datos (verificable corriendo el escáner sobre la base).
