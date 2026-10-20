# 00 · Visión

Decisión de origen: [ADR-043](adr/ADR-043-un-erp-con-la-ia-en-el-nucleo.md).

## Qué es SociAI

Un **ERP para organizaciones de la sociedad civil y sin fines de lucro, con la IA como núcleo y no como complemento**. Es una app de escritorio que guarda todo en el equipo de la institución.

Un ERP de siempre pide que la persona aprenda sus pantallas, sus campos y sus reglas. SociAI va al revés: **la persona cuenta y sube sus papeles, y el programa ordena, llena, avisa y prepara**. La IA está para quitar la fricción y la complejidad técnica, no para agregar un botón más.

Su filosofía está enfocada en un sector desatendido: asistencia social y organizaciones parecidas, con poco personal y sin perfil técnico. Su uso es genérico: no supone que la institución es un asilo o una casa hogar.

## El problema

- **La información está dispersa.** Actas, poderes, constancias, informes y cifras viven en carpetas, correos y cabezas. Cada trámite y cada convocatoria empieza desde cero.
- **Los ERP de siempre no sirven aquí.** Son caros, técnicos y suponen a alguien que capture todo con paciencia. En una institución pequeña, el tiempo de la dirección es lo más escaso.
- **Los proyectos se pierden.** Las instituciones tienen acceso a convocatorias de donativos (Juntas de Asistencia, Nacional Monte de Piedad, fundaciones), pero:
  - **ponen tiempo en proyectos inviables,** que no caben en la convocatoria o que ni la propia institución entiende;
  - **enmarcan mal el proyecto,** porque piden «remodelar un baño» cuando el proyecto real es «reducir el riesgo de caídas y dar autonomía a las personas mayores». El donante financia impacto, no obras;
  - **no priorizan:** nada les dice si esa es su necesidad más importante o la que mejor encaja en la convocatoria;
  - **pierden tiempo con formatos rígidos** de Word y Excel.

## El MVP

El MVP es lo que se valida primero, con las dos instituciones que ya prueban el programa. No es el objetivo completo.

1. **Centralizar y digitalizar la institución (el núcleo).**
   - «Mi institución» guarda una vez quién es, a quién atiende y cómo.
   - Su Expediente guarda sus documentos y lee de ellos lo que puede llenar.
   - El Asistente contesta, lleva a cada lugar y convierte lo que la persona cuenta en sugerencias.
   - Inicio dice qué falta y qué vence. Cuentas, permisos, seguridad y respaldo cuidan todo.
2. **Resolver los proyectos.** De una convocatoria a un proyecto que **cabe** en ella y que la institución **entiende**:
   - la lectura de la convocatoria y sus requisitos;
   - una conversación que lleva de «un baño» a un problema bien planteado y dice si conviene;
   - la redacción, y una guía en Word para llenar el formato del donante.

**Después del MVP:** Personal, Beneficiarios, Instalaciones, Finanzas y Donantes. Su código ya existe en parte y está oculto hasta validar el MVP.

## Cómo trabaja la IA

La IA decide dentro de la casa que le arma el programa, con pocos límites que no crecen con cada caso:

1. **El valor de un dato personal no sale del equipo.** Hacia la IA va una marca, nunca el valor (ADR-044).
2. **Nada se guarda sin una persona.** La IA propone; la persona acepta; el código guarda.
3. **La IA no calcula.** Totales, presupuestos y validaciones son del código.
4. **Permisos, no reglas de lenguaje.** Cada agente tiene sus herramientas y cada campo dice qué acepta.

Fuera de eso, la IA decide qué es cada cosa, a dónde va y qué preguntar. Detalle en los [principios de ingeniería](agents/principios-de-ingenieria.md).

## Para quién

| Persona | Necesita | No tolera |
|---|---|---|
| Dirección (directora, madre superiora, presidente del patronato) | Respuestas rápidas, decidir con claridad, proyectos que sí puede presentar | Jerga técnica, pantallas confusas, capturar lo mismo dos veces |
| Personal administrativo | Tener todo en un lugar y llenar formatos sin errores | Perder trabajo, rehacer cálculos |
| Desarrollador (responsable) | Mantener e instalar la app fácilmente | Complejidad innecesaria, reglas que crecen sin fin |

## Qué NO es

- **No manda nada por su cuenta.** Prepara; las personas envían.
- **No reemplaza el criterio de la dirección.** Propone y explica; la persona decide.
- **No saca datos de personas del equipo.** En el MVP no guarda fichas de personas. Cuando lleguen Personal y Beneficiarios, sus fichas tampoco saldrán del equipo.

## Cómo sabremos que funciona el MVP

- Las dos instituciones dejan su institución completa sobre todo contando y subiendo sus papeles, sin capturar campo por campo.
- Preparan un proyecto con una convocatoria real en una fracción del tiempo de hoy, y lo entienden. El caso del baño (`fixtures/caso-dorado-bano.md`) termina replanteado como lo haría un analista.
- Una institución que no es asilo ni casa hogar puede usar el núcleo sin opciones ni textos ajenos.
- El costo de IA por proyecto se mantiene en pocos pesos.
- Ningún valor de un dato personal llega a la IA (se revisa en cada batería).
