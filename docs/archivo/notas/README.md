---
tipo: guia
---

# Notas y observaciones

Aquí van las observaciones de la persona dueña del proyecto sobre los ADR, los bloques y las pantallas. **Una nota por observación.**

## Cómo dejar una observación

1. En Obsidian, cree una nota nueva dentro de esta carpeta (`notas`) con un nombre corto: «Bandeja de documentos – dudas».
2. Inserte la plantilla «observacion» (paleta de comandos → «Plantillas: Insertar plantilla»). La primera vez, en *Ajustes → Plantillas*, elija la carpeta `plantillas`.
3. En `sobre`, enlace el ADR o el documento del que habla (escriba `[[` y elija).
4. Escriba su observación. Deje `estado: abierta`.

También puede dejar una observación dentro de un ADR, sin cambiar su texto, con un bloque así:

```
> [!nota] Observación
> Lo que quiero decir.
```

## Qué hace Claude con ellas

- Al empezar cada bloque lee las notas con `estado: abierta` y los bloques `[!nota]` de los ADR.
- Responde en la sección «Respuesta» de cada nota: qué se decidió y qué cambió (y en qué ADR o bloque).
- Cuando queda resuelta, cambia `estado` a `resuelta`. Las notas no se borran: son la historia de las decisiones.
- Las decisiones que cambian algo importante pasan a un ADR (nuevo o con sus «Precisiones»), como siempre.

Las notas abiertas se ven en el [[00-mapa-del-proyecto|Mapa del proyecto]].

**Nunca escriba aquí datos reales de la institución** (nombres, RFC, cuentas): esta carpeta está en el repositorio.
