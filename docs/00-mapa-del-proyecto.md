---
tipo: mapa
actualizado: 2026-10-13
---

# Mapa del proyecto

La página de inicio: dónde estamos, qué sigue, qué espera tu respuesta y dónde está cada cosa. **Claude la actualiza al cerrar cada bloque.**

> [!info] Hoy (2026-10-13)
> **La lectura de archivos y el llenado desde documentos quedaron cerrados** ([ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md)):
> - una sola política de llenado;
> - la ventana del propio documento se llena sola;
> - Pdfium lee texto y tablas;
> - el OCR endereza hojas y no congela el equipo;
> - el Expediente lee en segundo plano.
>
> La forma de trabajar quedó escrita para los módulos en la [metodología de los Lectores](agents/metodologia-lectores.md).
>
> **Siguiente:** pruebes todo esto junto (pasos abajo) y C1; después, **C2**, el Lector del Expediente que llena «Mi institución» buscando campo por campo.

## Qué es SociAI, en una línea

Un programa de escritorio, local y con IA, para que asilos y casas hogar ordenen sus datos y armen proyectos para pedir donativos. Lo usan directivas y religiosas sin perfil técnico. Ver [Visión](00-vision.md).

## Qué sigue

| | Bloque | Qué hace | Estado |
|---|---|---|---|
| 1 | **C1** | Cada documento va a un solo cajón; Documentos es la bandeja que pregunta a qué cajón va (por ahora solo el Expediente); la búsqueda siempre dentro de un cajón | **Hecho, por probar** |
| — | **Lectura y llenado** | ADR-035, 037 y 038: una sola política de llenado, datos propios solos, Pdfium para texto y tablas, OCR (letra chica, hojas giradas, páginas mixtas, imágenes en Word y Excel, prioridad baja), lectura en segundo plano, el Expediente tapa y no rechaza, avisos de contraseña | **Hecho, por probar** |
| 2 | **C2** | El Lector del Expediente: llena todo «Mi institución» buscando en el Expediente campo por campo, con la cita de cada dato; botón «Llenar con mis documentos» | Pendiente |
| 3 | **C3** | El Asistente pregunta al Lector del Expediente y sabe qué documentos existen («según mi registro, ¿qué nombre legal pongo?») | Pendiente |
| 4 | **C4** | Revisión completa del núcleo: cada agente aprobado con el modelo más barato y cada pantalla probada por ti | Pendiente |
| 5 | **N5** | El expediente hacia Proyectos, la IA e Inicio | Pendiente |
| — | Módulos | Finanzas (facturas), Personal y Beneficiarios (Lector solo con reglas), Instalaciones, Donantes: cada uno con su cajón, su Lector y su batería, siguiendo la [metodología de los Lectores](agents/metodologia-lectores.md) | Esperan al núcleo |

Detalle de cada bloque en el [Roadmap](09-roadmap.md), «Fase 10».

## Lo que espera tu respuesta

- **Probar la lectura y el llenado** (pasos abajo) y **probar C1** (pasos más abajo); después, aprobar el inicio de C2.
- Borrar el archivo suelto `E:\Git\search_live3.txt` (lo dejó un error mío; el sistema no me deja borrarlo).
- Confirmar el precio de `gemini-embedding-2` en la página oficial de Google (anotado 0.20 USD por millón de tokens, tomado de sitios que comparan precios).

## Cómo probar la lectura y el llenado (ADR-035, 037 y 038)

**Antes:** corre una vez `powershell -ExecutionPolicy Bypass -File scripts/fetch-ocr.ps1`, que ahora también baja el modelo que endereza las hojas giradas.

**La ventana del propio documento se llena sola**
1. **Mi institución → Expediente → Poder del representante legal → Subir archivo**, con tu poder (o el acta, la constancia, el oficio de donataria, la 32-D).
   - La ventana dice «Leyendo… Puede cerrar esta ventana»: ciérrala.
   - La fila dice «Leyendo: N de M páginas».
   - Al terminar, un aviso dice cómo fue.
2. Vuelve a abrir el poder.
   - Las fechas de emisión y de vigencia y el número de escritura están llenos.
   - Debajo de cada uno dice «Leído del archivo, página N».
   - En una 32-D, la vigencia dice «Calculado con la fecha de emisión del archivo».
3. Corrige a mano la vigencia, guarda y sube otra vez el archivo: la vigencia que escribiste no cambia; la fecha de emisión y el número sí se leen del archivo nuevo.

**«Mi institución» solo recibe sugerencias en campos vacíos**
4. En «Datos legales y fiscales», escribe a mano el nombre legal y guarda: su sugerencia desaparece.
   - Sube otro documento que traiga el nombre: no vuelve a sugerirlo.
5. Descarta una sugerencia y vuelve a subir el mismo archivo: no vuelve.

**Archivos que no van o no se pueden abrir**
6. Sube la constancia en el lugar del acta.
   - Debe guardarse y avisar «Este archivo parece «Constancia de situación fiscal», no este documento».
   - No llena nada.
7. Un PDF con contraseña (`D:\Agente Proyectos\pdf_pruebas\protected.pdf`) avisa «tiene contraseña», también en **Mis proyectos → nueva convocatoria**.

**Escaneos y fotos**
8. Un acta escaneada larga: la computadora no se congela mientras se lee.
9. Una hoja escaneada de lado se lee bien, y una página con encabezado impreso y cuerpo escaneado también.
10. Un Word con una hoja escaneada pegada: su texto aparece en Documentos.

**Pruebas:** `cargo test core::reader documents:: core::records scanner` y `pnpm test`.

## Cómo probar C1

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
| 2026-10-13 | **Lectura y llenado sin huecos (ADR-038):** una sola política de llenado (datos propios solos, sugerencias solo en campos vacíos, nunca lo rechazado, autoridad por campo, lugar equivocado); Pdfium lee también las tablas (salen `pdf-extract` y `lopdf`); OCR con prioridad baja, hojas giradas, páginas mixtas e imágenes en Word y Excel; el Expediente lee en segundo plano; la metodología para los módulos | [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md), [metodología](agents/metodologia-lectores.md) |
| 2026-10-13 | **Pdfium lee el texto de todo PDF:** palabras enteras (antes «Thi s report»), párrafos por la distancia entre renglones, guion de corte; de 5 a 20 veces más rápido. Un PDF, Word o Excel con contraseña lo dice en el Expediente y en las convocatorias | [ADR-037, bloque 2](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **Pdfium contra `pdf-extract` (medición):** en 24 PDF de muestra sacan las mismas letras, pero `pdf-extract` parte palabras («Thi s report outli nes») y es de 5 a 20 veces más lento; se decide Pdfium como único motor | [ADR-037](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **El Expediente tapa, no rechaza:** ningún archivo del Expediente se rechaza por «lista de personas» ni pide «Tapar y seguir»; el escáner tapa todo, la pantalla dice cuántos datos y la bitácora guarda los conteos. Las convocatorias conservan el rechazo | [ADR-036, precisión del 2026-10-13](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **OCR · letra chica:** una imagen con letra de menos de 20 píxeles se agranda hasta que mida 30 (a lo más ×4) y se lee otra vez; una imagen chica sin ninguna palabra, ×3. Una captura ficticia pasa de 4 errores y sin título a leerse completa con su correo; una foto chica, de cero palabras a completa | [ADR-035, precisión 1](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
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
| Cómo un cajón lee sus archivos y llena sus formularios (para cada módulo) | [Metodología de los Lectores](agents/metodologia-lectores.md) · [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md) |
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
