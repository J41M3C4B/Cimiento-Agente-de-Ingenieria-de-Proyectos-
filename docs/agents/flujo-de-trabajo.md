# Flujo de trabajo

1. Antes de implementar una fase, lee su sección en `docs/09-roadmap.md` y los documentos que referencia.
2. Si una especificación es ambigua o contradictoria, **pregunta** antes de inventar.
3. Trabaja en pasos pequeños y verificables; corre pruebas al terminar cada uno.
4. Al cerrar una tarea, actualiza la documentación afectada y marca el avance en el roadmap.
5. **Datos reales: nunca en el repositorio ni en las pruebas.** El código, las pruebas y los fixtures usan solo datos ficticios de `fixtures/`. Cuando una institución prueba la aplicación instalada con datos suyos, esos datos viven únicamente en su equipo; nunca se copian al repositorio, a un fixture ni a un documento.
6. Una decisión importante se registra en un ADR; una decisión que cambia se registra en un ADR nuevo que reemplaza al anterior.
7. Los cambios de modelo, de prompt o de arquitectura se justifican con una medición, no con una opinión: primero en seco, después una corrida real acotada.
