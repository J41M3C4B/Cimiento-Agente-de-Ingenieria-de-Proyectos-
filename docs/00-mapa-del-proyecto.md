---
tipo: mapa
actualizado: 2026-10-13
---

# Mapa del proyecto

La página de inicio: dónde estamos, qué sigue, qué espera tu respuesta y dónde está cada cosa. **Claude la actualiza al cerrar cada bloque.**

> [!info] Hoy (2026-10-12)
> Estamos en el **ADR-036 · Cajones de documentos con dueño**. **Primero solo el núcleo** («Inicio» y «Mi institución»): lo dejamos sin problemas y probado, y los módulos repiten después la misma forma de trabajar.
> **C1 está hecho:** Documentos ya es la bandeja del Expediente. **Siguiente bloque: C2**, el Lector del Expediente. No empieza hasta que pruebes C1 y lo apruebes.

> [!warning] Por decidir (2026-10-13): el OCR con documentos reales
> Las actas constitutivas reales se rechazan como «lista de personas», la letra chica se lee mal y un acta larga ocupa demasiado el equipo. Mediciones y propuestas en el [ADR-035, precisiones 2026-10-13](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md). Hoy un acta real con varios socios suele quedar fuera del Expediente.

## Qué es SociAI, en una línea

Un programa de escritorio, local y con IA, para que asilos y casas hogar ordenen sus datos y armen proyectos para pedir donativos. Lo usan directivas y religiosas sin perfil técnico. Ver [Visión](00-vision.md).

## Qué sigue

| | Bloque | Qué hace | Estado |
|---|---|---|---|
| 1 | **C1** | Cada documento va a un solo cajón; Documentos es la bandeja que pregunta a qué cajón va (por ahora solo el Expediente); la búsqueda siempre dentro de un cajón | **Hecho, por probar** |
| — | **OCR** | Las actas ya no se rechazan como lista (van a cuarentena); la letra chica se agranda; Tesseract con prioridad baja; medir `tessdata_fast` | **Por decidir** (ADR-035, precisiones 2026-10-13) |
| 2 | **C2** | El Lector del Expediente: llena todo «Mi institución» buscando en el Expediente campo por campo, con la cita de cada dato; botón «Llenar con mis documentos» | Pendiente |
| 3 | **C3** | El Asistente pregunta al Lector del Expediente y sabe qué documentos existen («según mi registro, ¿qué nombre legal pongo?») | Pendiente |
| 4 | **C4** | Revisión completa del núcleo: cada agente aprobado con el modelo más barato y cada pantalla probada por ti | Pendiente |
| 5 | **N5** | El expediente hacia Proyectos, la IA e Inicio | Pendiente |
| — | Módulos | Finanzas (facturas), Personal y Beneficiarios (Lector solo con reglas), Instalaciones, Donantes: cada uno con su cajón, su Lector y su batería | Esperan al núcleo |

Detalle de cada bloque en el [Roadmap](09-roadmap.md), «Fase 10».

## Lo que espera tu respuesta

- **Probar C1** (pasos abajo) y aprobar el inicio de C2.
- **Decidir el orden del OCR** (ADR-035, precisiones 2026-10-13): propuesta, primero la regla de «lista de personas», después prioridad baja y agrandado de letra chica, al final `tessdata_fast` y la lectura en segundo plano.
- Borrar el archivo suelto `E:\Git\search_live3.txt` (lo dejó un error mío; el sistema no me deja borrarlo).
- Confirmar el precio de `gemini-embedding-2` en la página oficial de Google (anotado 0.20 USD por millón de tokens, tomado de sitios que comparan precios).

## Cómo probar el último bloque (C1)

1. Empieza con una base vacía o vuelve a subir tus documentos: los de antes de C1 no tienen cajón y no aparecen.
2. **Documentos → «Subir documento»:** debe preguntar «¿A qué cajón va?» (solo Expediente) y «¿Qué documento es?» (los del Expediente y «Otro documento»). Al seguir, se abre la misma ventana del Expediente de «Mi institución»; ahí pones el archivo.
3. El documento aparece en Documentos con el nombre de su documento del Expediente, agrupado por clase. El lápiz vuelve a abrir su ventana.
4. Sube el mismo documento desde «Mi institución» → Expediente: debe verse igual en Documentos.
5. Reemplaza el archivo de un documento: en Documentos debe quedar solo el nuevo.
6. «Buscar en el expediente» encuentra por nombre y por dentro.
7. Acepta una sugerencia que venga de un documento y luego bórralo con el bote de basura: debe borrarse sin error y el dato aceptado se queda.

## Lo último que se hizo

| Fecha | Qué | Dónde leerlo |
|---|---|---|
| 2026-10-13 | **OCR con documentos reales (solo medición):** agrandar la letra chica lleva una captura de 13 palabras mal leídas a 2; dos actas reales se rechazan como «lista de personas» por dos reglas toscas del escáner; 37 páginas tardan 171 s en un equipo de 4 núcleos; no se entrena Tesseract ni se usa la nube | [ADR-035, precisiones 2026-10-13](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-12 | **C1:** cada documento lleva su cajón; Documentos es la bandeja (pregunta qué documento del Expediente es y abre su ventana); la búsqueda siempre en un cajón; quitar o reemplazar un archivo saca su texto del cajón; «Borrar esto y todo…» ya no falla con un dato aceptado | [ADR-036, precisiones C1](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **ADR-036:** cajones con dueño, un agente por cajón, el Asistente como orquestador, cada agente validado con el modelo más barato; primero el núcleo | [ADR-036](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **IA6a con documentos reales:** el Lector entiende un acta real de 14 páginas y la ficha de la Junta (nombre, figura, año, objeto, número de institución); el objeto nunca incluye nombres del patronato | [Roadmap, IA6a](09-roadmap.md) |
| 2026-10-12 | **IA6a:** la clase y el tipo de cada documento; sus datos llegan como sugerencias con documento y página; un dato protegido solo por reglas | [ADR-034, precisiones IA6a](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-12 | **IA5:** búsqueda dentro de los documentos por palabras y por significado (vectores); medido antes de activarla: de 50 % a 100 % de aciertos con preguntas dichas con otras palabras | [ADR-034, precisiones IA5](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-10 | **OCR:** leer páginas escaneadas y fotos | [ADR-035](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-10 | **N4:** el expediente de la institución con vigencia | [ADR-033 §5](adr/ADR-033-nucleo-expediente-y-formularios-universales.md) |
| 2026-10-10 | **IA3–IA4:** el Asistente en toda la app y el Capturista («Llenarlo por mí») | [ADR-034](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |

## Cómo está armado, en un dibujo

```
            Documentos  (la bandeja: aquí se sube y se ve todo)      ← ADR-036
                 │  «¿A qué cajón va?»
   ┌─────────────┼──────────────┬──────────────┬──────────────┐
Expediente     Personal       Finanzas       Proyectos     Beneficiarios,
(núcleo)       (módulo)       (módulo)       (convocatorias) Instalaciones…
 └ su Lector    └ solo reglas  └ su Lector    └ lectura canónica
 └ llena «Mi institución»

Asistente: la puerta de entrada en toda la app; pregunta al agente del cajón que corresponde.
Capas (ADR-032): base → módulos → núcleo → proyectos → pantallas; cada capa solo mira hacia abajo.
```

## Dónde está cada cosa

| Para saber… | Lee |
|---|---|
| Qué es y para quién | [Visión](00-vision.md) |
| Cómo está construido (capas, núcleo y módulos) | [Arquitectura](01-arquitectura.md) · [ADR-032](adr/ADR-032-monolito-modular-nucleo-y-modulos.md) |
| Qué datos nunca salen y cómo se cuidan | [Gobernanza de datos](03-gobernanza-datos.md) · [Escáner](04-escaner-datos-sensibles.md) |
| Qué tablas hay y de quién es cada una | [Modelo de datos](05-modelo-datos.md) |
| Cómo usamos la IA y cuánto cuesta | [IA y costos](07-ia-y-costos.md) |
| Cómo se escriben los textos de la pantalla | [Estilo de redacción](08-estilo-redaccion.md) |
| Qué se ha hecho y qué falta, bloque por bloque | [Roadmap](09-roadmap.md) |
| Cómo se ve el programa | [Sistema visual](13-sistema-visual.md) |
| Qué dice la ayuda del programa | [Manual](manual/programa.md) · [Mi institución](manual/mi-institucion.md) |
| Las reglas para trabajar en el código | [Principios](agents/principios-de-ingenieria.md) · [Flujo de trabajo](agents/flujo-de-trabajo.md) · [Comandos y pruebas](agents/comandos-y-pruebas.md) |
| Por qué se decidió cada cosa | Los ADR, abajo · [Índice de ADR](adr/README.md) |

## Tus observaciones

Cómo dejarlas: [Notas y observaciones](notas/README.md). Una nota por observación, con la plantilla `plantillas/observacion`.

Abiertas (necesita el complemento Dataview):

```dataview
TABLE WITHOUT ID file.link AS "Nota", sobre AS "Sobre", fecha AS "Fecha"
FROM "notas"
WHERE estado = "abierta"
SORT fecha DESC
```

Observaciones dentro de los ADR: búsquelas con la búsqueda de Obsidian (Ctrl + Shift + F) escribiendo `"[!nota]" path:adr`.

## Las decisiones (ADR)

Las vigentes, de la más nueva a la más vieja:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha", tema AS "Tema", reemplaza_en_parte AS "Reemplaza en parte"
FROM "adr"
WHERE adr AND estado != "reemplazada"
SORT adr DESC
```

Las que ya no rigen, o que otra cambió en parte:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", reemplazado_por AS "La reemplaza", reemplazado_en_parte_por AS "La cambia en parte"
FROM "adr"
WHERE adr AND (estado = "reemplazada" OR reemplazado_en_parte_por)
SORT adr DESC
```

Por tema (para ver solo lo de IA, del núcleo o de documentos):

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha"
FROM "adr"
WHERE contains(tema, "ia")
SORT adr DESC
```

Cambie `"ia"` por `"nucleo"`, `"documentos"`, `"proyectos"`, `"modulos"`, `"seguridad"`, `"interfaz"`, `"arquitectura"`, `"infraestructura"` o `"datos"`.
