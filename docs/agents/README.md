# Guía operativa para agentes de desarrollo

Cimiento se desarrolla con apoyo de agentes de código. Esta carpeta es el **manual de operación** que esos agentes leen antes de tocar el proyecto: qué reglas no se negocian, cómo se llaman las cosas, cómo se prueba y en qué orden se trabaja.

La idea es la misma que rige el producto: **el agente propone, y las pruebas, los ADR y el criterio del autor deciden.** Las especificaciones de `docs/` son la fuente de verdad; esta carpeta solo les dice a los agentes cómo respetarlas.

| Documento | Contenido |
|---|---|
| [`principios-de-ingenieria.md`](principios-de-ingenieria.md) | Qué es el proyecto, los principios que no se negocian, stack, estructura de carpetas y reglas de código |
| [`glosario-dominio.md`](glosario-dominio.md) | Vocabulario del dominio (español) y su nombre en el código (inglés) |
| [`comandos-y-pruebas.md`](comandos-y-pruebas.md) | Cómo correr, probar y empaquetar; pruebas en seco y pruebas con IA real (opt-in, con tope de llamadas) |
| [`flujo-de-trabajo.md`](flujo-de-trabajo.md) | Orden de trabajo, cuándo preguntar, cómo cerrar una tarea y qué datos se pueden usar |

## Controles que hacen confiable el trabajo asistido

- **Especificación primero:** cada fase empieza leyendo su sección del roadmap y los documentos que referencia.
- **Pruebas obligatorias** para escáner, cálculos, transiciones de etapa y lectura/escritura de Office; la suite completa corre sin red y sin costo con una IA simulada.
- **Decisiones registradas:** si una librería o un enfoque no funciona, se documenta en un ADR nuevo; los ADR no se reescriben.
- **Medición antes que opinión:** los cambios de modelo, de prompt o de arquitectura se justifican con una corrida medida y su costo.
- **Datos reales fuera del repositorio:** solo `fixtures/` (ficticios) entra al código y a las pruebas.
