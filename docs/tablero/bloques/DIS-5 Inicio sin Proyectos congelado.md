---
tipo: bloque
id: DIS-5
carril: diseno
estado: hecho
prioridad: media
caja: Interfaces de los módulos
pedido_por: ERP-4
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado: 2026-10-20
---

## Qué y por qué

El líder congeló Proyectos por completo (2026-10-20), como Personal, Beneficiarios, Instalaciones y Finanzas: hasta que el núcleo (Mi institución y Documentos, con la plataforma) esté listo, no se avanza en él. El ERP-4 lo oculta del mapa de lugares, y por eso la barra y la búsqueda del manual ya no lo muestran.

Pero **Inicio sigue ofreciéndolo**: el botón «Nuevo proyecto», el bloque de proyectos en curso y el paso de proyectos en los primeros pasos, y para dibujarlos llama a `project_list`. Con Proyectos oculto, la persona vería una puerta a algo que no puede usar.

Se necesita que **mientras Proyectos esté congelado, Inicio no ofrezca nada de Proyectos** ni le pregunte nada. Cuando se descongele, vuelve como estaba o como se rediseñe entonces.

## Terminado cuando

**Propuesta del riel de diseño: este pedido lo cubre el [[DIS-4 Proyectos sin la IA vieja|DIS-4]]** y no necesita un bloque aparte. El DIS-4 ya quita de Inicio el botón de nuevo proyecto, la carpeta de proyectos en curso y la consulta `project_list`, y de la bienvenida y el arranque lo que lleve a Proyectos o lo prometa. Hacerlos en dos PR sería partir lo que hay que fusionar junto. El líder decide si este bloque se cierra como cubierto por el DIS-4.

- [x] Lo cubre el DIS-4 (ver su «Terminado cuando»).

## Plan

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

## Resultado

Cubierto por el [[DIS-4 Proyectos sin la IA vieja|DIS-4]] (PR #19, `e2df0bf`): Inicio ya no ofrece ni pregunta nada de Proyectos.
