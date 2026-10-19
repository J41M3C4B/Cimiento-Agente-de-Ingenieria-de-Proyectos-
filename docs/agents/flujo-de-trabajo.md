# Flujo de trabajo

0. **Todo trabajo es un bloque del [tablero](../tablero/Tablero.md)**, con su rama y su PR, y sigue los roles de [roles y tablero](roles-y-tablero.md). Nada se empieza sin un bloque en «Listo» ni se fusiona a `main` sin la validación del líder.
1. Antes de implementar una fase, lee su sección en `docs/09-roadmap.md` y los documentos que referencia.
2. Si una especificación es ambigua o contradictoria, **pregunta** antes de inventar.
3. Trabaja en pasos pequeños y verificables; corre pruebas al terminar cada uno.
4. Al cerrar una tarea, actualiza la documentación afectada y marca el avance en el roadmap.
5. **Datos reales: nunca en el repositorio ni en las pruebas.** El código, las pruebas y los fixtures usan solo datos ficticios de `fixtures/`. Cuando una institución prueba la aplicación instalada con datos suyos, esos datos viven únicamente en su equipo; nunca se copian al repositorio, a un fixture ni a un documento.
6. Una decisión importante se registra en un ADR; una decisión que cambia se registra en un ADR nuevo que reemplaza al anterior.
7. Los cambios de modelo, de prompt o de arquitectura se justifican con una medición, no con una opinión: primero en seco, después una corrida real acotada.
8. **Observaciones de la persona dueña del proyecto:** al empezar un bloque, lee las notas con `estado: abierta` en `docs/notas/` y los bloques `> [!nota]` de los ADR; responde en cada nota y márcala `resuelta` cuando quede atendida. Al cerrar un bloque, actualiza `docs/00-mapa-del-proyecto.md` (hoy, qué sigue, lo que espera respuesta y lo último que se hizo) y, si un ADR es nuevo o cambia de estado, sus propiedades (`estado`, `fecha`, `reemplaza…`).
9. **Un cajón que lee archivos** (Expediente, Personal, Beneficiarios, Instalaciones, Finanzas, Donantes) sigue la [metodología de los Lectores](metodologia-lectores.md): el mismo camino del archivo, la misma política de llenado (ADR-038) y su batería con pruebas de ciclo de vida.
