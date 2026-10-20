---
tipo: tablero
---

# Tablero

Cada bloque de trabajo es una nota en `tablero/bloques/` hecha con la plantilla «bloque». Para moverlo de columna se cambia su propiedad `estado`. Las reglas, los roles y las ramas están en [[roles-y-tablero|Roles y tablero]].

**Carriles:**
- `erp`: el producto y su arquitectura (pantallas, datos, formularios, flujos, proyectos);
- `ia`: la arquitectura de la IA (agentes, bóveda, oficios, herramientas, modelos y baterías);
- `diseno`: cómo se ve y cómo se recorre la app (sistema visual, componentes, ventanas, sugerencias, avisos y errores), para el agente de diseño.

**En curso:** a lo más un bloque por carril.

## En curso

```dataview
TABLE WITHOUT ID file.link AS "Bloque", carril AS "Carril", rama AS "Rama", fase AS "Fase"
FROM "tablero/bloques"
WHERE estado = "en-curso"
SORT carril ASC
```

## Revisión

Hecho por el CTO; espera que el validador lo pruebe.

```dataview
TABLE WITHOUT ID file.link AS "Bloque", carril AS "Carril", rama AS "Rama"
FROM "tablero/bloques"
WHERE estado = "revision"
SORT carril ASC
```

## Listo

Definido, con «Terminado cuando» aprobado; el siguiente se toma de aquí.

```dataview
TABLE WITHOUT ID file.link AS "Bloque", carril AS "Carril", prioridad AS "Prioridad", fase AS "Fase"
FROM "tablero/bloques"
WHERE estado = "listo"
SORT choice(prioridad = "alta", 1, choice(prioridad = "media", 2, 3)) ASC
```

## Backlog

Ideas, hallazgos y pendientes sin definir todavía.

```dataview
TABLE WITHOUT ID file.link AS "Bloque", carril AS "Carril", prioridad AS "Prioridad", fase AS "Fase"
FROM "tablero/bloques"
WHERE estado = "backlog"
SORT choice(prioridad = "alta", 1, choice(prioridad = "media", 2, 3)) ASC
```

## Hecho

```dataview
TABLE WITHOUT ID file.link AS "Bloque", carril AS "Carril", cerrado AS "Cerrado"
FROM "tablero/bloques"
WHERE estado = "hecho"
SORT cerrado DESC
LIMIT 15
```
