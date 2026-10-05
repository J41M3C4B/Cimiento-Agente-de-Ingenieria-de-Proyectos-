# ADR-016 · El proyecto nace de su convocatoria; institución global, proyectos independientes

**Estado:** Aceptada (2026-10-03). Implementada: migración 0005, `project_create_from_call`, pantalla «Mis proyectos». Queda pendiente reordenar las etapas (ver «Lo que no cambia todavía»).

## Contexto
Había tres puertas sueltas para cosas que se relacionan: «Convocatorias» (archivos + nombre), «Documentos» (tipo + nombre + texto) y «Mis proyectos» (texto libre). Nada ataba una convocatoria a un proyecto, y la persona tenía que clasificar documentos con una pregunta que no siempre sabe responder. Jaime propuso unificar el alta en un proceso por etapas lógicas y fijó un principio de datos.

## Decisión
1. **Institución = global, proyecto = independiente.** El perfil y los documentos internos son de la institución y valen para todo proyecto; nunca cuelgan de un proyecto. La convocatoria, sus archivos, su lectura, el diagnóstico y lo demás son de un proyecto y no se comparten con otro.
2. **El proyecto nace de la convocatoria.** «Mis proyectos» → «Empezar un proyecto con una convocatoria»: un archivo (que es la convocatoria) o, si marca «trae más documentos», varios archivos donde cada uno dice qué es (la convocatoria — exactamente una —, anexo, guía, formato para llenar, aviso o modificación, otro). Se piden nombre, quién convoca y año. Un solo comando (`project_create_from_call`) revisa y limpia los archivos, crea el proyecto, su lectura y sus archivos en **una transacción** y lo deja en `DIAGNOSIS`; la lectura corre en segundo plano (ADR-011, ADR-015). Si algo falla (perfil, archivo, texto) no se guarda nada.
3. **El tipo de documento lo da la sección, no una pregunta:** «Mis proyectos» sube la convocatoria; «Documentos» (de la institución) guarda documentos internos; las cotizaciones llegarán dentro del proyecto con el presupuesto (Fase 5).
4. **Lo que escribe la persona vale más que lo que lee el modelo.** El nombre, el donante, el año y el carácter de cada archivo se guardan como los dijo la persona (`call_reading.funder/year`, `call_reading_file.role`). Si la lectura encuentra otro donante u otro año, `call_reading_get` devuelve `differences` y la pantalla lo avisa («vale lo que usted escribió»); solo avisa, no bloquea. Esto resuelve el `nombre` inestable con portadas decorativas.
5. **Copia propia por proyecto.** Dos proyectos que aplican a la misma convocatoria suben y leen cada uno la suya (≈ $1.3–2.2 MXN por lectura). Reusar una lectura por la huella del archivo queda como optimización futura que no cambia el flujo.
6. **Proyectos sin convocatoria: puerta abierta, apagada.** `project.kind` es `call` o `internal`; `project.call_reading_id` puede ser nulo; el comando `project_create` (texto libre) sigue existiendo y crea proyectos `internal`. La pantalla ya no lo ofrece. Servirá para convertir casos cotidianos de la institución en proyectos planificados. Los proyectos que ya existían quedaron como `internal`.
7. **Borrar un proyecto** (`project_delete`, con confirmación) borra todo: lo suyo por cascada y su convocatoria (archivos, texto, lectura) por el borrado de emergencia de cada archivo. Reemplaza al «Borrar convocatoria» de la pantalla anterior. Borrar un archivo de la convocatoria desde otro lado deja al proyecto con «los archivos se borraron» (`call_reading_id` nulo, `ON DELETE SET NULL`).

## Consecuencias
- Desaparece la pestaña «Convocatorias»; su resumen («esto es lo que entendimos») vive en el proyecto (`CallPanel`). «Documentos» muestra solo los de la institución (`document_add_text` fija `internal`; `documents_list` omite los de convocatoria).
- Texto que se escribe al crear (nombre, donante) pasa por el escáner como cualquier otro; si parece dato de una persona, cuarentena.
- `project.initial_request` se llena con una frase armada por el código («La institución quiere postular a la convocatoria «X» de Y (2026)»), de modo que el diagnóstico actual sigue funcionando sin cambios.

## Lo que no cambia todavía
- **El orden de las etapas.** El proyecto ya nace con su convocatoria, pero `CALL_SELECTION` sigue siendo la 4.ª etapa y `call_chosen_without_blockers` sigue en falso. Falta decidir y hacer: convertir esa etapa en «confirmar el encaje» y llevar el resumen de la convocatoria al diagnóstico (ADR-010, aún propuesta).
- Cotizaciones dentro del proyecto, requisitos tipificados desde el documento canónico y la fusión de «Documentos» con «Mi institución» en una sola sección.
