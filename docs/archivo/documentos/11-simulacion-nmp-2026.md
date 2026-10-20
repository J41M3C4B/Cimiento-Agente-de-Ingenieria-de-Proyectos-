# 11 · Simulación con una convocatoria real (Nacional Monte de Piedad 2026)

**Fecha:** 2026-10-01. **Qué fue:** un ensayo a mano de la metodología de `10-metodologia-conversacion.md`, sin el sistema. Jaime hizo de directora de «Casa Hogar Amparo IAP» (institución ficticia; algunos detalles se inventaron sobre la marcha) y Claude hizo el trabajo de la IA: leer la convocatoria, llevar la bitácora y conducir la conversación.

**Insumo:** la convocatoria real «Convocatoria 2026 · Inversión Social» (30 páginas, con texto extraíble; se extrajo con `pypdf`). No es un caso escaneado.

## 1. Recorrido

| Vuelta | La IA | La persona aporta | Se cierra |
|---|---|---|---|
| 1 | Resume el marco de la convocatoria como bitácora y hace 3 preguntas (compuerta, quiénes y cuántos, pregunta de fondo) | Es IAP con 20 años; 22 niñas de 3 a 18; la idea del comedor digno | Marco; afectados (con una discrepancia) |
| 2 | Hace 2 preguntas y propone la raíz para confirmar; avisa de un riesgo de seguridad | Come tres veces al día; la comida sabe a óxido; el gas flamea; mantenimiento por descuido; un supuesto sobre un filtro | Problema, consecuencias |
| 3 | Pregunta quién se encarga del mantenimiento y si hay prueba | No hay encargado; hay un fondo extraordinario; el profesional dijo «que sí» | **Causa raíz** (mantenimiento solo reactivo) |
| 4 | Pide detalle del profesional y propone objetivo, alcance e indicadores | Baños en mal estado (2 caídas, 1 corte, 2 de 5 regaderas), el profesional confirma el tubo degradado | Consecuencias, efectos secundarios; descarta la hipótesis del filtro |
| 5 | Plan de pruebas y punto de control «lo que entendí» | Contrato anual con técnico después de 12 meses | Alcance e indicadores propuestos; sostenibilidad a medias |
| 6 | Cierre | Contador y directora coordinan; dinero del fondo extraordinario, más revisión de donantes | Sostenibilidad; objetivo firme |

Cada respuesta de la IA fue a la vez la conversación y su registro etiquetado (respondió, punto clave, objetivo, punto a revisar, decisiones abiertas) con cita y solidez.

## 2. Resultado: el diagnóstico que salió de la bitácora

Formato de `02-flujo-funcional.md`. El número de niñas lo da el perfil de la institución, no la conversación.

```json
{
  "problem_statement": "El comedor, la cocina y los baños están deteriorados (agua con óxido, gas con llama inestable, regaderas y losetas en mal estado) porque la casa no tiene responsable, plan ni fondo de mantenimiento: solo se repara cuando algo falla.",
  "affected": { "group": "Niñas de 3 a 18 años en situación de vulnerabilidad", "count": null, "description": "Usan a diario el comedor, la cocina y los baños. Cifra: del perfil." },
  "current_consequences": [
    "La comida sabe a óxido, sobre todo la que lleva agua (referido: las niñas).",
    "La estufa flamea con llama naranja; según un profesional, por el óxido en la conexión de gas (referido).",
    "2 caídas en regaderas con azulejo resbaloso y 1 corte con losetas sueltas.",
    "2 de 5 regaderas funcionan (40 %).",
    "Llaves sueltas y sin agua por mala instalación."
  ],
  "root_causes": [
    "No hay responsable, plan ni fondo de mantenimiento; la directora atiende cuando le avisan.",
    "Falta de conocimiento técnico sobre el mantenimiento necesario.",
    "El tubo de conexión del agua y del gas está degradado (referido: profesional)."
  ],
  "reframed_need": "Comida segura y nutritiva en un comedor digno y baños seguros, con un sistema de mantenimiento para que la casa no vuelva a deteriorarse.",
  "alternatives": [
    { "title": "Solo reparar cuando falle", "pros": ["Sin inversión inicial"], "cons": ["Es lo que ha pasado; el daño se repite"] },
    { "title": "Obra sin plan de mantenimiento", "pros": ["Resuelve lo urgente"], "cons": ["Se volvería a deteriorar"] },
    { "title": "Obra más sistema de mantenimiento (agenda, responsable, fondo)", "pros": ["Resuelve lo urgente y evita que se repita", "Genera costos reales para decidir entre contrato anual o agenda por servicio"], "cons": ["Requiere responsable interno y fondo pasados los 12 meses"] }
  ],
  "suggested_indicators": [
    "Instalaciones de agua y gas conformes (hoy no conformes)",
    "Regaderas funcionando (hoy 2 de 5)",
    "Áreas con mantenimiento preventivo (hoy no hay plan)",
    "Caídas y cortes por instalaciones (hoy 2 y 1)"
  ],
  "open_questions": [
    "Monto real de la plataforma (se supuso que cubre los gastos).",
    "Informe y cotización escritos del profesional.",
    "Análisis del agua (el olor a azufre no lo explica solo el óxido).",
    "Quién paga el contrato pasados los 12 meses: se propone reasignar del fondo extraordinario, sin monto ni compromiso."
  ]
}
```

Pendientes de evidencia (plan de pruebas): informe y cotización del profesional, revisión de la instalación de gas, detector de monóxido, análisis del agua, fotos con fecha, registro de incidentes sin nombres, 2 o 3 cotizaciones por concepto.

## 3. Medidas

| Medida | Valor |
|---|---|
| Vueltas de la IA hasta objetivo y raíz firmes | 5 más un cierre |
| Preguntas explícitas hechas | 11, más 3 confirmaciones |
| Preguntas que el perfil de la institución habría contestado | 3 de 11 (compuerta, quiénes y cuántos, el 22 contra 20) |
| Preguntas respondidas a medias o sin responder | 6: por qué no se ha podido hacer (mitad); informe escrito del profesional; alcance A o B (implícito); confirmación de objetivo e indicadores (silencio); monto (supuesto); dinero del contrato (mitad) |
| Hallazgo no previsto | La raíz (falta de sistema de mantenimiento) apareció en la vuelta 3, no estaba en el planteamiento inicial |
| Contradicción atrapada | Hipótesis «viene de la calle, hace falta filtro» frente a «tubo degradado» del profesional |

## 4. Lo aprendido y cambios a la metodología

1. **El perfil de la institución debe ser la fuente de verdad** y mucho más completo. La compuerta, el tipo de institución y las cifras base no se preguntan: se confirman. El código detecta si lo dicho en la conversación contradice al perfil. Hoy el formulario no tiene: datos legales (figura, regulador, años como donataria, autorización vigente, Anexo 14); instalaciones con detalle (agua, gas, electricidad, último mantenimiento, responsable, inventario de mobiliario); ingresos por donante con tipo (operación o inversión); alianzas; línea base (peso y talla); vigencia de documentos; fecha de última confirmación por dato.
2. **La pregunta de apertura debe ser fija y de dos partes:** qué cambiaría (resultado) y por qué no se ha podido (obstáculo). De la respuesta parcial salieron las dos preguntas que encontraron la raíz.
3. **Solidez de cada hecho:** observado, referido (lo dijo otra persona), supuesto o **verificado** (con documento). Solo lo observado o verificado puede mover presupuesto o indicadores. Evitó que un filtro supuesto entrara al presupuesto.
4. **Plan de pruebas generado desde la bitácora:** el código lista los hechos críticos (seguridad, salud, costos) por debajo de «verificado»; la IA solo redacta cada prueba con **qué es, quién la hace y qué documento queda**, en palabras sencillas (el detector de monóxido es un aparato de ferretería, no una consulta a un químico). La convocatoria exige evidencia y cotizaciones desglosadas a precio de mercado.
5. **Formulario progresivo en tiempo real.** La IA ya produce preguntas estructuradas; deben presentarse como campos tipados (texto, opciones, número, sí o no, «no sé», «después») de uno en uno, sin abrumar. El código registra cada pregunta hecha con su estado (respondida, parcial, saltada, declinada) y la vuelve a mostrar hasta cerrarla. La persona olvidó contestar cosas; el sistema no debe depender de su memoria. Las confirmaciones (objetivo, indicadores, resumen) son explícitas; el silencio no confirma.
6. **Proponer para confirmar funciona mejor que preguntar en abierto:** la raíz y el objetivo se cerraron antes por proponerlos.
7. **La convocatoria debe llegar como JSON extraído de forma determinista, no leída por la IA.** Hacerlo con IA a mano cuesta mucho. Hay que cubrir varios casos: PDF con texto, PDF escaneado (se necesita OCR), formularios (DOCX, XLSX, formularios web) y enlaces a complementos (aquí, un ZIP de ejemplos de documentos). El extractor trae por regla lo estructurable (fechas, montos, porcentajes, duraciones, listas de documentos, rubros, población) con su página y fragmento, y segmenta el resto por secciones etiquetadas para que la IA lea solo la que necesita. La IA queda para lo ambiguo, con confirmación humana.
8. **Las tablas son lo difícil.** En esta convocatoria, las matrices de documentos por etapa (Anexo 1) y de indicadores de producto y resultado (Anexo 3) salieron desordenadas con extracción de texto simple: hace falta extracción con posición de palabras y confirmación humana.
9. **Datos que no vienen en el documento:** el monto lo da la plataforma, así que es un campo obligatorio que la persona o el código completa; hasta entonces, un supuesto marcado.
10. **Fechas:** según el PDF, la postulación fue del 8 al 15 de abril de 2026. El código debe comparar las fechas con hoy y avisar si la ventana ya pasó, antes de gastar una sola vuelta.
11. **Alertas de seguridad:** cuando aparezcan señales de riesgo (gas, llama, monóxido, instalación eléctrica), la IA lo avisa aparte de la convocatoria. Aquí sirvió para sugerir una revisión del gas y un detector.

## 5. Extractor determinista: primera medición (2026-10-01)

Primera rebanada del cambio 7 de §4: `documents/call_extract.rs`. Entra el texto por página que ya produce `read_pdf` y sale un JSON `call_extraction.v1`. Cada dato lleva su página y el fragmento de donde sale; lo que ninguna regla puede resolver se lista aparte y no se adivina.

**Qué hace por reglas:** repara el texto (ligaduras, espacios raros, enlaces cortados, letras mal decodificadas); lee límites (duración máxima, tope de gastos administrativos, años mínimos de operación, montos con «máximo» o «mínimo»), figuras jurídicas, fechas (rangos, fecha con hora, «a partir de <mes>») y su estado frente a un día dado; criterios de participación y documentos listados; el Anexo 1 numerado; rubros y conceptos financiables; enlaces (formulario, sesión, plataforma, descarga) y correos; indicadores con su agenda; etiquetas por página; y borradores de `requirement` para que una persona los confirme.

**Medición contra la convocatoria real (30 páginas), comparada con lo que se leyó a mano:** 36 de 36 hechos de referencia, y 201 pruebas de Rust pasan. No pide OCR, no inventa ningún monto (el PDF no trae) y tarda unos segundos.

**Lo que enseñó la medición, y por qué el «36 de 36» pide cuidado:**
- **Los conteos solos engañaron.** La primera versión pasaba «21 documentos en el Anexo 1», pero los números 1 a 3 eran los pasos del índice («Postulación», «Registro», «Entrega de Recursos») y habían desplazado a los documentos reales. Y «18 criterios» incluía 7 pasos de procedimiento de la página 9. Se vio al **leer el JSON**, no los números. Ahora la medición compara contenido (qué dice el documento 3, el 11, el 21) y hay pruebas que fijan cada error.
- **Es el mismo documento con el que se escribieron las reglas.** Eso mide que las reglas funcionan ahí, no que generalicen. Hace falta una segunda convocatoria de otro donante, sin mirarla antes.
- **Los dos extractores de PDF fallan distinto:** `pdf-extract` deja letras mal decodificadas en algunas fuentes (`æ` por ñ, `Ø` por é, `Æ` por á, `œ` por ú, `˝` por Í), que se reparan por regla porque no existen en español; `pypdf` decodifica bien los acentos pero deja `/f_i`. El extractor del proyecto, con la reparación, queda mejor.

### 5.1 Segunda ronda: siete PDFs más (barrido sin tocar reglas, después ajustes)

Se agregaron a la carpeta: el anexo de rubros, las convocatorias de NMP 2025 y 2024, la guía de información general y financiera, los dos anexos de indicadores y un PDF de la JAP. **Los 8 tienen capa de texto: ninguno pidió OCR.** Método: primero un barrido con el extractor tal cual estaba; después se desarrolló con NMP 2025 y la JAP y se **reservó NMP 2024 como prueba sin ajustar reglas con su contenido** (solo se verificó que lo extraído fuera correcto).

**Hallazgos:**
- **El PDF de la JAP no es una convocatoria de donativos.** Es el aviso del proceso electoral de representantes de las IAP ante el Consejo Directivo de la Junta (postulación de candidaturas, votación el 19 de agosto de 2026). Falta una convocatoria de apoyo de la JAP, si existe. Además, el PDF arrastra el mismo texto en cada página (páginas 2, 3 y 4 repiten la 1; la 6 repite la 5).
- **Los indicadores sueltos coinciden con los del anexo de la convocatoria 2026** (36 y 22), y el anexo de rubros trae los mismos 62 conceptos: dos documentos distintos, una misma verdad.
- **Lo que generaliza y lo que no.** Las reglas de una frase (duración, años mínimos, tope administrativo, figuras jurídicas, enlaces, correos, rubros, indicadores) funcionan en los tres NMP. Las que dependen del formato de la página (criterios, documentos, fechas) fallaron en cuanto cambió el formato: NMP 2025 no lleva viñetas (el símbolo se perdió) y escribe las fechas como «19 MAYO-6 JUNIO» o «A PARTIR DEL 23 DE JUNIO*».

**Errores de corrección que apareció al revisar el contenido de NMP 2024 (reservado) y que ya se corrigieron:**
- Una fecha suelta («DOF publicado en 18 enero 2024», un requisito sobre un documento) salía como ventana de postulación, por haber la palabra «formulario» unas líneas antes. Ahora una ventana es un rango, o una fecha con la señal pegada.
- «Duración máxima de 6 meses» y «monto máximo de $350,000» valen solo para una categoría. Ahora el límite guarda los otros valores (`others`) y se marca `conditional`; se ofrece a confirmar y nunca como requisito bloqueante.

**Se agregó:** reconocer el tipo de documento (convocatoria de donativos, anexo de rubros, anexo de indicadores, guía, aviso que no es convocatoria), con acierto en los 8 PDFs; detectar páginas repetidas; criterios escritos sin viñetas; más formatos de fecha (con aviso de que el año se tomó del documento y de que un calendario en gráfico no dice a qué etapa pertenece cada fecha); y revisión humana marcada para lo que no es un valor único.

**Todavía no sale por reglas en NMP 2024:** los criterios de participación y los documentos (0), y casi todas las fechas (1 de 1 correcta tras el ajuste). Esa es la medida honesta de cuánto generaliza hoy. Nota de costo: leer una convocatoria completa con IA **una sola vez** cuesta del orden de $0.05 a $0.08 USD (estimado, 12 a 19 mil tokens de entrada más la salida) y se reutiliza para todas las instituciones y proyectos: lo caro sería releerla en cada vuelta de conversación, no extraerla una vez.

**Lo que NO resuelve por reglas** (queda para una persona o para la IA leyendo solo las páginas etiquetadas): la tabla de documentos por etapa (las marcas «Sí» salen sin su fila), la dimensión y el tipo (producto o resultado) de cada indicador, el monto (lo da la plataforma), y el texto narrativo (criterios de evaluación, estrategias, población), que la IA leería por etiqueta.

**Pendiente:** segunda convocatoria de otro donante; PDF escaneado con OCR (hoy solo se detecta); formularios DOCX, XLSX y web; descargar los complementos enlazados; tablas con extracción por posición de palabras; y aprobar tres tipos de requisito nuevos (`allowed_legal_figure`, `min_operating_years`, `max_admin_percent`) en `05-modelo-datos.md`.

## 6. Límites de esta simulación

- Una sola persona, técnica, que conoce el método y colaboró; una directora real puede ser menos clara. Sigue pendiente la batería de personas simuladas y la prueba piloto con una persona real (`10`, §6.1).
- Parte de lo dicho se inventó para el ejercicio (por ejemplo, el filtro).
- No hubo llamadas a un modelo: los costos del método siguen siendo estimaciones.
- Las afirmaciones técnicas (color de la llama, óxido, combustión) son del profesional del ejercicio; la IA no las valida, solo marca el riesgo.
