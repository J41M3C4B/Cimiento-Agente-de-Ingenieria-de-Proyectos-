# ADR-011 · Ingesta en silencio de la convocatoria y archivos Word/Excel solo de lectura

**Estado:** Aceptada (Fase 3). La revisión con IA se probó con un servicio simulado y con una corrida real sobre la convocatoria de NMP 2026 (ver «Primera corrida real»). Falta medirla en un documento donde las reglas fallan (NMP 2024).

## Contexto
Decisiones de Jaime tras la simulación con la convocatoria real de Nacional Monte de Piedad (`docs/11`):
- **Alcance de formatos:** todo lo que llega (convocatorias, reglas, anexos, guías) es PDF con texto; no hay documento de la JAP de donativos (el PDF que había era un aviso electoral). OCR, formularios y otros formatos se difieren hasta que aparezca uno real.
- **Word y Excel pasan a solo lectura.** La IA los usa para guiar; la persona los llena a mano. El trabajo pesado (conceptualizar el proyecto) ya queda hecho en el chat, y llenar los formatos a mano deja la responsabilidad del resultado en la persona y no en la plataforma. Falta además conocer más de esos formatos.
- **La convocatoria se procesa al subirla, sin que la persona lo note:** el chat arranca ya sabiendo de qué se habla, si el proyecto puede participar y qué requisitos hay, y la IA formula la primera pregunta con ese contexto completo.

## Decisión
1. **Cadena de ingesta de una convocatoria en PDF**, en este orden:
   1. `read_pdf` → texto por página (páginas sin texto se reportan; con la mayoría sin texto se pide OCR, diferido).
   2. `call_extract` → extracción determinista (límites, fechas, criterios, documentos, rubros, enlaces, indicadores, borradores de requisito), cada dato con página y fragmento; tipo de documento; páginas repetidas.
   3. `call_review` → **un modelo lee la convocatoria una sola vez**: recibe lo extraído por reglas (con la posición de cada elemento) y solo las páginas relevantes (hasta 24,000 caracteres, unos 8,000 tokens; no se envían las tablas de indicadores ni la lista de rubros, que las reglas leen completas). Devuelve **solo lo que cambia**: elementos a quitar, límites, fechas, criterios y documentos que faltan, el perfil de la convocatoria (quién convoca, objetivo, líneas de apoyo y población, cómo se evalúa, lo que no se financia) y dudas.
   4. **El código verifica cada cita.** Todo elemento del modelo lleva `page` y una `quote` copiada literalmente; el código la busca en el texto de esa página (sin importar saltos de línea, mayúsculas ni signos). Lo que no aparece se **descarta y se lista** (`report.rejected`); nunca se confía. Las quitas exigen cita y una posición que exista. Los valores que ya estaban no se duplican (semejanza por palabras; cifras distintas son cosas distintas); un valor distinto para un mismo límite se guarda al lado y queda condicionado.
   5. Lo que agrega el modelo queda con origen `ai_assumption` hasta que una persona lo confirme.
2. **La revisión nunca bloquea la subida.** Si no hay llave, internet o cupo, el resultado es la extracción por reglas y el reporte dice por qué (`report.ai`: `used`, `not_needed` o el tipo de falla). Solo se revisa con IA lo que es convocatoria de donativos; anexos de indicadores o rubros, guías y avisos que no son convocatoria no gastan una llamada.
3. **La confirmación se hace dentro del chat** (propuesta, a decidir con Jaime): el primer mensaje resume «esto es lo que entiendo de la convocatoria» y pregunta solo lo que quedó dudoso (valores condicionados a una categoría, calendarios sin etapa clara, supuestos de la IA). No hay pantalla aparte.
4. **Word y Excel:** el código de lectura y de escritura de `documents/` se conserva, pero el llenado automático y la generación del `.docx` final salen del alcance por ahora; las Fases 4 y 5 se reducen a guiar a la persona con lo que dicen esos formatos (ver el roadmap).

## Costo
Una revisión envía unos 8,000 tokens de entrada como máximo y recibe hasta 3,500 de salida más el razonamiento. Con `gemini-3.5-flash` ($1.50 / $9.00 por millón) eso es del orden de $0.03 a $0.08 USD por convocatoria (estimado, a medir con `call_review_live`), **una sola vez por convocatoria**: el resultado se reutiliza para todas las instituciones y proyectos.

## Primera corrida real (2026-10-01, `gemini-3.5-flash`, convocatoria de NMP 2026)
- **Funcionó:** 1 llamada, 13 páginas enviadas (23,860 caracteres), 8,384 tokens de entrada y 5,905 de salida (4,560 de razonamiento). **44 segundos** y **$1.22 MXN (≈ $0.066 USD)**, dentro de la estimación de $0.03 a $0.08.
- **Qué hizo bien:** agregó 2 criterios y 2 documentos que las reglas no tenían (p. ej. que la agenda de acceso a derechos la ejecuta directamente el albergue o asilo) y escribió un perfil útil: quién convoca, el objetivo, las dos líneas de apoyo con su población, cómo se evalúa, lo que no se permite, y la duda correcta (el monto no está en el documento). Ninguna cita fue rechazada.
- **Lo que enseñó (por qué hay una v2 del prompt y más reglas en el código):**
  1. **Una cita verificada no hace fiel a la paráfrasis.** De 10 elementos del perfil, 2 cambiaron términos: «mayor pobreza multidimensional» se volvió «pobreza extrema», y «fines políticos, partidistas o religiosos» se volvió «proselitismo político o religioso». Ahora el código marca `paraphrase_suspect` cuando el texto usa dos o más palabras largas que la cita no tiene (se probó que atrapa ambos casos y no marca una paráfrasis normal), el prompt prohíbe cambiar términos, y **la cita literal siempre acompaña al texto y es lo que vale**.
  2. **Los documentos agregados eran condicionales** (los estados financieros 2024 de la etapa «Actividad anual» solo los piden a organizaciones nuevas) y salían como requisitos bloqueantes. Ahora el esquema trae `applies_to` para criterios y documentos, y **todo lo que agrega el modelo es no bloqueante** hasta que una persona lo confirme.
  3. **44 s es mucho para una sola llamada**, casi todo razonamiento. No es un problema porque corre una vez en silencio al subir el archivo, pero conviene una barra de avance y probar menos esfuerzo de razonamiento para esta tarea.
- **Lo que no demuestra:** que el revisor encuentre errores de verdad. En este documento las reglas ya habían dado 36 de 36, así que el revisor no tenía errores graves que corregir (0 quitas). La prueba que falta es NMP 2024, donde las reglas dan 0 criterios y 0 documentos.

### Segunda medida: NMP 2024 con el modelo real
- Duró 273 s con 5 intentos fallidos y costó $0.4162 MXN (`gemini-3.6-flash`). Los fallos eran tiempos agotados de 120 s que se clasificaban como «sin conexión», se reintentaban en el mismo modelo y no contaban para el tope. Ahora son `Timeout`, con tiempo por tarea (revisión 180 s), pasan al siguiente modelo de la cadena y se detienen tras 2.
- El modelo acertó: sesión informativa 2024-04-10 11:00 y quitó una fecha falsa (publicación en el DOF). Las citas verificadas fueron correctas; 3 marcas de paráfrasis, probablemente falsos positivos cuando el modelo une texto cercano.
- Huecos de las reglas que mostró: la p. 8 (5 criterios con «•», sin título de participación) no se leía ni se enviaba, y el calendario «Abril 01 al 22» no se entendía. Corregido: se toma la página donde están los años mínimos o las figuras legales, esa página se envía al modelo, y hay una regla «Mes DD al DD». Sin IA, las reglas dan ahora 5 criterios y 4 fechas.
- 2024 ya no es un conjunto reservado puro: se leyeron p. 8 y 24–27 para verificar. Queda NMP 2025 y la repetición con las correcciones.

## Consecuencias
- La IA no relee la convocatoria en la conversación: recibe el resumen estructurado (límites, fechas, criterios, perfil) que dejó la ingesta.
- El perfil de la convocatoria es lo que el diagnóstico (ADR-010) usa como marco desde la primera pregunta.
- Falta persistir el borrador (`grant_call`, `call_template`, `requirement` con origen y confirmación; el modelo de datos ya tiene los campos), guardar los fragmentos del PDF para búsqueda (`document_chunk`) y la pantalla para subir el archivo.
- La calidad real del revisor (¿encuentra errores de verdad?, ¿el perfil sirve para la primera pregunta?) no está medida: solo el comportamiento del código ante respuestas buenas y malas.

> **Actualización (ADR-013):** la revisión de este ADR (modelo que propone y cita, código que verifica citas) fue reemplazada por el riel de convocatorias: el código corta la convocatoria en candidatos y el modelo solo etiqueta. Lo anterior queda como historia de por qué se cambió.
