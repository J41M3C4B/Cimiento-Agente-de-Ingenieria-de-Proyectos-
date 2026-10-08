# 09 · Roadmap

Cada fase termina con un criterio verificable. No se avanza sin cumplirlo. Marca `[x]` al completar.

## Fase 0 · Cimientos técnicos

Objetivo: proyecto que compila, base cifrada y las pruebas técnicas de riesgo resueltas.

- [x] Crear proyecto Tauri 2 + React + TypeScript + Vite + Tailwind con pnpm.
- [x] Estructura de carpetas de `01-arquitectura.md` (módulos vacíos con sus traits).
- [x] SQLite con SQLCipher; llave generada y guardada en el llavero del SO.
- [x] Sistema de migraciones; aplicar `0001` de `05-modelo-datos.md`.
- [x] Archivo `src/i18n/es-MX.ts` y wrapper tipado de `invoke()`.
- [x] **Prueba técnica Excel** (ver `06-documentos-office.md`).
- [x] **Prueba técnica Word** (ver `06-documentos-office.md`).
- [x] **Prueba técnica PDF:** extraer texto de un PDF con tablas y uno escaneado (el escaneado debe detectarse y avisar).
- [x] **Prueba técnica sqlite-vec** cargado junto con SQLCipher.
- [x] Actualizar comandos en `docs/agents/comandos-y-pruebas.md`.
- [x] ADRs actualizados con los resultados (ADR-002 y ADR-004; PDF abajo).
- [x] Revisión visual de los archivos de `src-tauri/target/office-out/` en Excel y Word reales (hecha; los formatos reales son más grandes, con muchas páginas y hojas con fórmulas ya armadas: revisar en Fases 4 y 5).

**Terminado cuando:** `pnpm tauri dev` abre una ventana, la base está cifrada (no se abre sin llave) y las 4 pruebas técnicas tienen resultado documentado.

## Fase 1 · Perfil, escáner y bitácora

- [x] Escáner completo según `04-escaner-datos-sensibles.md` con fixtures de positivos y negativos.
- [x] Flujo de cuarentena en la UI.
- [x] Bitácora (`audit_log`) con los eventos mínimos. Ya se registran `document.uploaded`, `scanner.quarantine`, `scanner.override`, `emergency.delete` y `profile.confirmed`; los demás (`scanner.leak_prevented`, `ai.call`, `export.created`, `stage.changed`, `backup.created`) existen en el código y se activan con sus fases.
- [x] Pantallas de perfil: institución, población, personal, instalaciones, ingresos. Versionado al confirmar.
- [x] Botón de borrado de emergencia (para documentos).
- [x] Cargar `fixtures/institucion-asilo.json` y `fixtures/institucion-casa-hogar.json` desde un comando de desarrollo.
- [x] **Revisión de «Mi institución», bloque 1: datos generales (2026-10-07, ADR-026).** Guardado revisado (montos con comas, tope de cifras, avisos de RFC, teléfono y correo). Ficha de la IA: «Gasto anual aproximado» en lugar de «Presupuesto anual», ingresos por tipo y egresos; nómina y cuotas del padrón nunca como cifra, y balance en palabras. Ingresos por tipo con cuotas calculadas del padrón, egresos en lista o con aproximado exprés, nómina con aguinaldo y prima vacacional (LFT 2023), balance. Migración 0014.
- [x] Diseño de la pantalla para el bloque 1 (sesión de diseño): ingresos por tipo y periodo, lista de egresos con modo exprés, tarjeta de balance, nómina con prestaciones.
- [x] **«Mi institución», bloque 2: Personal como base de RH (2026-10-07, ADR-027).** Módulo aparte (`hr/`, tablas `hr_*`, migración 0015 con traslado del padrón), modalidades con reglas y modalidades propias, catálogo de puestos con plazas autorizadas, formulario en 4 pasos con avance, CURP/RFC/NSS/CLABE validados y tapados, baja distinta de borrado, aportaciones y personal externo como egresos aparte, y a la IA solo agregados (atributos personales con grupos de 3 o más).
- [x] Diseño fino del módulo de Personal (sesión de diseño).
- [x] **Perfiles de acceso (2026-10-07, ADR-028).** Cuentas con contraseña (Argon2id), administrador y dirección/contaduría, permiso declarado y revisado en Rust en cada comando (con prueba que impide comandos sin permiso), borrados de dirección/contaduría como solicitudes con el registro oculto, panel de administración (cuentas, solicitudes, bitácora, código de recuperación), bloqueo por inactividad, bitácora con autor; el PIN se retira. Migración 0016.
- [x] Diseño fino de las pantallas de acceso y del panel de administración (sesión de diseño).
- [x] **«Mi institución», bloque 3: Beneficiarios e inteligencia de datos (2026-10-07, ADR-029).** Módulo aparte (`care/`, tablas `care_*`, migración 0017 con traslado del padrón, que desaparece), ficha en 5 pasos según asilo o casa hogar, salud solo por categorías, lista de espera, tablero con indicadores y hallazgos que cruzan beneficiarios con espacios, dinero y personal, aviso de privacidad por persona; a la IA solo conteos y hallazgos sin dinero. `common/` con los validadores compartidos.
- [ ] Diseño fino del módulo de Beneficiarios y su tablero (sesión de diseño).
- [x] **«Mi institución», bloque 4: Instalaciones (2026-10-07, ADR-030).** Módulo aparte (`facilities/`, tablas `fac_*`, migración 0018 con traslado de la lista del perfil, que desaparece). El inmueble con m², pisos y cómo se sube, tenencia y papeles, servicios, y seguridad y protección civil. Espacios y equipo por grupo con conteo por estado y lo no contado «sin revisar», fallas de una lista, camas, barras y regadera accesible. Tablero con indicadores (m² por persona, personas por baño, camas) y hallazgos que cruzan la casa con las personas atendidas; a la IA todo, salvo conteos de personas con atributos de menos de 3.
- [ ] Diseño fino del módulo de Instalaciones y su tablero (sesión de diseño).
- [ ] Umbrales de la NOM-031-SSA3 y la NOM-032-SSA3 para calificar las proporciones de Instalaciones (confirmar en el texto oficial).
- [x] **Primer inicio (2026-10-08, ADR-031).** Bienvenida por cuenta, puesta en marcha del administrador y asistente obligatorio de datos de la institución en 6 pasos con revisión; cifras rápidas de personal y beneficiarios; ubicación, año de fundación, figura jurídica, donataria y CLUNI en el perfil y en la ficha de la IA. Migración 0019. `CIMIENTO_DATA_DIR` para probar desde cero sin tocar los ejemplos.
- [ ] Diseño fino del primer inicio (sesión de diseño) y lista de «Siguientes pasos» en Inicio.
- [ ] Módulo remoto para aprobar solicitudes a distancia (sobre `access_request`).

- [x] Prueba manual en la app: cargar un ejemplo, pegar una CURP ficticia en las notas, ver la cuarentena y tapar; agregar un documento de texto y borrarlo con el botón de emergencia.

**Terminado cuando:** se capturan ambos perfiles ficticios, todas las pruebas del escáner pasan y pegar un texto con CURP ficticia activa la cuarentena.

## Fase 2 · Diagnóstico con IA

- [x] Trait `AiProvider` + `AnthropicProvider` + configuración de modelos por nivel (ADR-006).
- [x] Llave de API en llavero; pantalla para capturarla («Ayuda automática»).
- [x] Registro `ai_usage` y contador de gasto con tope (aviso al 80 %, pausa al 100 %).
- [x] Máquina de etapas (`domain/stage.rs`) con pruebas.
- [x] Diagnóstico de 7 dimensiones con repreguntas acotadas (máx. 2 por dimensión, el código lleva la cuenta).
- [x] Resumen del diagnóstico con validación de esquema, revisión de cifras y confirmación.
- [x] Priorización con puntaje calculado por código.
- [x] ~~Modo sin IA (preguntas base fijas y borrador del resumen armado con las respuestas)~~ → retirado el 2026-10-03: Cimiento funciona solo con IA (ADR-017).
- [x] `GeminiProvider` (ADR-007), selector de proveedor, límites por modelo con ritmo, métricas en tiempo real (migración 0003) y comprobación de llave sin gastar llamadas.
- [x] Prueba en seco del caso dorado completo contra un servicio Gemini simulado por HTTP real (`cargo test dry_run`).
- [x] Primera corrida real: Flash-Lite respondió bien (9 llamadas, 0.76 s en promedio); `gemini-3.8-flash` dio 503 por alta demanda (8.3 s hasta el error). Se agregó la cadena de respaldo 3.8 → 3.7 → 3.6 (ADR-008) y la prueba en seco del 503.
- [x] Sonda real (`gemini_probe_live`): la petición fuerte del resumen es válida (200 en `gemini-3.5-flash` y en Flash-Lite, con `anyOf`+`null` y `thinkingLevel`). `gemini-3.8/3.7/3.6-flash` seguían en 503, así que el modelo fuerte principal pasó a `gemini-3.5-flash` (ADR-009).
- [x] `golden_case_live` con `gemini-3.5-flash` al frente (2026-10-01): resumen escrito por la IA, sin cifras inventadas ni datos personales. Métricas reales: 11 llamadas (10 ligeras, 1 fuerte), 0 fallidas; ligeras 0.5–0.8 s (una de 12.5 s); resumen 12.4 s, 1,405 de entrada y 2,735 de salida (2,077 de razonamiento); costo a precio de pago ≈ $0.55 MXN (el resumen ≈ $0.49). Criterios: 11 de 14 con la vara anterior (falta «a ras de piso» literal, «puerta amplia» y «capacitación»; el criterio «silla de baño» se cumplía en falso con «silla de ruedas» y ya se corrigió).
- [x] Simulación a mano con una convocatoria real de Nacional Monte de Piedad 2026 (`docs/11-simulacion-nmp-2026.md`): el método llegó a la raíz en la vuelta 3 y al objetivo firme en la 5. De ahí salieron cambios a la propuesta (`10`, §13), entre ellos adelantar el extractor determinista de convocatorias de la Fase 3.
- [ ] ~~Versión 2 de los prompts~~ → reemplazada por la **propuesta** de conversación guiada con bitácora y convocatoria primero (`docs/10-metodologia-conversacion.md`, ADR-010). Pendiente de aprobación; si se aprueba, cambia el orden de etapas y adelanta parte de la Fase 3.
- [ ] Primera llamada real a Gemini: `cargo test gemini_smoke_live -- --ignored --nocapture` (una llamada). Confirma la forma de la petición escrita desde la documentación (ADR-007, lista de pendientes).
- [ ] Probar en la app con una llave real: guardarla, probar la conexión, hacer el diagnóstico completo y revisar el resumen.

- [ ] Correr el caso dorado en vivo (`cargo test golden_case_live -- --ignored --nocapture`) y registrar aquí las métricas reales (tiempos, llamadas, costo equivalente). Con Gemini usa unas 8 llamadas ligeras y 2 fuertes de las del día.

**Terminado cuando:** el caso dorado (`fixtures/caso-dorado-bano.md`) produce un replanteamiento equivalente al esperado, y se reporta el costo real del diagnóstico.

## Fase 3 · Convocatorias

- [x] **Extractor determinista de convocatorias, primera versión** (`documents/call_extract.rs`, adelantado tras la simulación `docs/11`): PDF con texto → JSON `call_extraction.v1` con límites, fechas, criterios, documentos, rubros, enlaces, indicadores y borradores de requisito, cada dato con su página y fragmento; lo que no resuelve (tablas, valores externos como el monto) queda en `needs_review` / `external`. Sin IA y sin costo. Medido contra la convocatoria real de NMP 2026: 36 de 36 hechos de referencia por reglas (ver `docs/11` §5, con sus límites: es el mismo documento con el que se escribieron las reglas).
- [x] Extractor, segunda ronda con 7 PDFs más (`docs/11` §5.1): clasifica el tipo de documento (8 de 8), detecta páginas repetidas, generaliza duración, figuras jurídicas, fechas y criterios sin viñetas, y marca para confirmar lo condicionado a una categoría. NMP 2024 se reservó sin ajustar y sigue sin sacar criterios ni documentos por reglas: esos casos irán a la IA leyendo solo las páginas etiquetadas, con confirmación humana.
- [x] **Alcance acordado (ADR-011):** solo **PDF con texto**. Todo lo que llega (convocatorias, reglas, anexos, guías) es PDF; no hay convocatoria de donativos de la JAP (el PDF era un aviso electoral). OCR, formularios y otros formatos quedan diferidos hasta que aparezca uno real.
- [x] **Revisión con IA de la convocatoria, una sola vez** (`documents/call_review.rs`, tarea `call.review`, prompt `call_review.v1`): el modelo recibe lo que sacaron las reglas y las páginas relevantes, corrige y completa, y escribe el perfil de la convocatoria. **Cada cita se verifica en el código** contra la página; lo que no aparece se descarta y se lista. Lo que agrega queda como supuesto de la IA. Nunca bloquea la subida si falla. Probada con un servicio simulado por HTTP real (respuestas buenas, citas inventadas, modelo caído, todos caídos).
- [x] Primera corrida real de la revisión (NMP 2026, `gemini-3.5-flash`): 1 llamada, 44 s, ≈ $1.22 MXN; agregó 2 criterios y 2 documentos y un perfil útil con 0 citas rechazadas. Mostró que una cita verificada no garantiza una paráfrasis fiel (2 de 10 cambiaron términos) y que lo agregado era condicional: prompt v2, `applies_to`, `paraphrase_suspect` y lo agregado por la IA ya no bloquea (ADR-011).
- [x] Primera corrida real de `call_review_live` sobre **NMP 2024** (273 s, 5 intentos fallidos, $0.4162 MXN con `gemini-3.6-flash`). El modelo acertó (sesión informativa 2024-04-10 11:00 y quitó una fecha falsa), pero reveló tres huecos que ya se corrigieron: (1) los reintentos por tiempo agotado se contaban como «sin conexión» y se repetían en el mismo modelo → ahora `Timeout` propio, tiempo por tarea (pregunta 60 s, resumen/necesidades 120 s, revisión 180 s), pasa al siguiente modelo y se detiene tras 2; (2) la elegibilidad no se leía en la p. 8 (sin título de participación) → ahora también se toma la página donde las reglas encuentran los años mínimos/figuras legales, y esa página se envía al modelo; (3) el calendario «Abril 01 al 22» no se leía → regla nueva «Mes DD al DD». Sin IA, las reglas dan ahora 5 criterios y 4 fechas en esa convocatoria. Nota: 2024 deja de ser un conjunto reservado puro (leí p. 8 y 24–27 para verificar).
- [x] Ajustes por los 503 de Google (ADR-008): los 5xx no cuentan para el cupo local y la revisión de convocatoria puede terminar en Flash-Lite. Gemma 4 31B probado y descartado.
- [x] **Riel de convocatorias, parte 1: segmentador de candidatos** (`call_rail.rs`, ADR-013): 27/27 de lo que las reglas hallan en NMP 2026, 19/20 en 2025, 17/17 en 2024; ≈ 3,600 a 5,300 tokens por convocatoria.
- [x] **Riel, parte 2: etiquetado en una sola llamada** (prompt `call_review.v3`, ADR-013): vocabulario cerrado, preguntas fijas con «no aparece», el texto lo pone el código. Probado en seco con el servicio simulado. El extractor aprendió horas de 24 h y fechas con hora sin año.
- [x] **Riel, parte 3 (código)**: dos corridas reales sobre NMP 2024 con `gemini-3-flash-preview` (22 s, ≈ $0.35 MXN); ajustes: calendario por etapa, etiquetas `project_requirement` y `funding_condition`, contrato de `priority`, lectura laxa de «15 Julio», extractor con horas de 24 h. Prueba de acuerdo entre modelos escrita (`call_review_agreement_live`).
- [x] Primera prueba de acuerdo en NMP 2024 (Preview contra Flash-Lite): 76.5 % de etiquetas iguales, preguntas fijas 8/9; Flash-Lite 10 s y $0.24, Preview 17 s y $0.31. Respuesta: contratos por etiqueta en código (ADR-013).
- [x] Acuerdo entre Preview y Flash-Lite con los contratos: 2025 84.1 %, 2026 87.0 % (2024 sin comparar por un 503). Detrás: `applies_to` (57 %), índice y títulos tomados como reglas, oraciones cortadas, esquema numerado. Corregido: `applies_to` lo deduce el código, el segmentador descarta índice y títulos, une oraciones cortadas y parte el esquema numerado.
- [x] Tercera prueba de acuerdo: 2024 88.4 %, 2025 83.2 %, 2026 86.5 %, aplicabilidad 100 %, producto idéntico en 2024 y 2025. Cambios: `procedure` fuera del vocabulario, anexo de documentos fuera del envío, medida `critical_rate` (lo que alimenta el producto).
- [x] Cuarta prueba de acuerdo: etiqueta 91.6 % (2024), 81.2 % (2025), 94.1 % (2026); preguntas fijas 8/9, 9/9, 9/9; el producto difiere como máximo en 1 criterio y 1 documento. `date` y `limit` salen del vocabulario (11 etiquetas): las fechas y topes los dan las reglas y las preguntas fijas.
- [x] Quinta prueba de acuerdo: 87.4 % / 80.2 % / 91.9 %; Preview dio 5 y luego 9 criterios en 2024 con el mismo código: el ruido de un solo modelo es del tamaño de la diferencia entre modelos. Añadidos `with_sampling` (opt-in) y `call_review_consistency_live`.
- [x] Ruido de Flash-Lite contra sí mismo en 2024: 96.8 % por defecto, **100 %** con temperatura 0 y semilla 7. `call.review` ahora fija ese muestreo (`AiTask::sampling`). El ruido grande era de Preview.
- [x] Preview con temperatura 0: 3 de 3 `truncated` (bucle de razonamiento). El muestreo fijo queda solo para los modelos medidos (`sampling_models`: Flash-Lite) y `Truncated` pasa al siguiente modelo de la cadena.
- [x] Preview con su muestreo natural en 2024: 3 corridas `used`, producto idéntico (5, 10, 10, 4), criterios y documentos 100 % consigo mismo, etiqueta 95.8 %.
- [x] **Riel de convocatorias cerrado** (ADR-013): sexta prueba de acuerdo Preview contra Flash-Lite: 92.6 % (2024) y 84.2 % (2025) por etiqueta, producto final con diferencias de 1 a 2 criterios y 1 documento; 2026 sin comparar por un 503 de Preview. Contrato de `criterion` apretado (una línea solo bajo un título de requisitos debe estar escrita como requisito).
- [x] Modelo del riel: Flash-Lite primero (temperatura 0, semilla 7) y Preview de respaldo (`AiTask::light_model_first`, ADR-013).
- [x] **¿El riel oculta la convocatoria?** Medido (`call_full_reading_vs_rail_live`): 74 % (2026) y 62 % (2024) de las conclusiones de una lectura completa están en el producto; casi todo lo que no está es secundario para decidir si participar. Pérdidas reales: requisitos del proyecto en `not_relevant`, población por tipo de institución (asilos 60+), descripciones del anexo, estrategias. Ver ADR-013. Plan original de la medida: (lectura completa por un modelo fuerte contra el producto del riel, conclusión por conclusión) en NMP 2026 y 2024. De ahí sale qué se pierde y dónde (etiquetado, segmentador o conclusiones que nacen de juntar partes) y si la búsqueda por significado debe ser la vuelta al texto original.
- [ ] Medida de calidad del riel: omisiones y sobrantes contra una referencia verificada por convocatoria (lista de requisitos y documentos de 2024 y 2025, como ya hay para 2026). El acuerdo entre modelos no mide corrección (ADR-013).
- [ ] Riel, dos pendientes pequeños: etiqueta `population` («a quién atiende cada programa») y descripción de cada documento del anexo junto a su nombre.
- [x] Construida la lectura completa estructurada (`call_sheet`, tarea `call.sheet`) y el banco del duelo (`call_duel_live`) con referencias verificadas de Alsea (42 hechos) y Bienestar (22). Primer vistazo sin gastar: las reglas dan 0 criterios en Alsea y descartan las Reglas de Operación de Bienestar como «no convocatoria».
- [x] Primer duelo (Alsea 42, Bienestar 22): riel 25 y 3; lectura completa con Preview 34 y 11; con Flash-Lite 8 y 11. El riel perdía hechos por los contratos (48 % de las etiquetas descartadas en Alsea) y por la puerta de clasificación, hechos a la medida del Monte de Piedad. La lectura completa detectó la contradicción 30 %/20 % entre el PDF y el Word de Alsea.
- [x] Segundo duelo: el riel libre con Flash-Lite dio 33/42 y 12/22, igual que la lectura completa con Preview (34 y 11) a la mitad del costo. **Decisión (ADR-014): riel v2** (sin contratos ni puerta, 15 etiquetas, prompt v4).
- [x] Tercer duelo (riel v2, Flash-Lite): Alsea 37/42, Bienestar 18/22, Alsea + Word 37/42, ≈ $0.2 y 8–10 s. Los 5 faltantes de Alsea eran dos defectos del segmentador (listas numeradas tomadas por títulos; viñetas de «Rubros Financiables» descartadas), corregidos sin modelo.
- [ ] **PUNTO DE DECISIÓN (ver ADR-014, autocrítica del 2026-10-02):** Jaime señala que el riel solo se ajusta a lo que tenemos y no es lo bastante inteligente para cubrir cualquier convocatoria. Mañana: congelar la arquitectura y medir a ciegas con 2 o 3 convocatorias nuevas de otras instituciones; candidatos sin suposiciones de formato (riel mínimo sin filtros / lectura completa por secciones). Sin más ejecuciones hasta decidir. Los embeddings se aplazan.
- [ ] (Aplazado) embeddings y RAG, usando como preguntas de referencia las 100 conclusiones de las lecturas completas (cada una con su página) (búsqueda por significado sobre los PDFs y la memoria de la institución). Empieza con preguntas de referencia y la línea base de FTS5, sin gastar.
- [x] Cadena: 3.5-flash apagado pero principal, 3-flash-preview como único respaldo fuerte, 3.5-flash-lite ligero. Familia 2.x y 3.6/3.7/3.8 fuera (ADR-012).
- [ ] Repetir `call_review_live` sobre NMP 2024 con las correcciones y sobre NMP 2025. Pendiente: las fechas del calendario se leen bien pero algunas quedan con tipo `other`; que el modelo las clasifique o afinar la regla.
- [ ] Decidir el esfuerzo de razonamiento para la revisión (hoy 44 s, 77 % razonamiento) y mostrar un avance mientras corre.
- [x] **Riel archivado y lectura canónica conectada al producto (2026-10-03, ADR-015):** el código y los prompts del riel quedaron en `docs/codigo-archivado/` y `docs/prompts-archivados/`; la tarea de IA es una sola (`call.canonical`). Pantalla «Convocatorias»: se suben el PDF y los Word/Excel que lo acompañan, se guardan por página (`document`, `document_chunk`, FTS) y la lectura corre en segundo plano (`call_reading`, migración 0004): por bloque con embeddings, segunda lectura de lo poco citado, citas verificadas por el código. Estados: guardada, leyéndola, lista, leída en parte, no se pudo leer; «Leer otra vez» sin gastar de más; «Esto es lo que entendimos» con archivo y página de cada línea, contradicciones, lo que no aparece y qué tan completa fue la lectura. Sin llave, sin internet o sin cupo la convocatoria queda guardada y esperando.
- [x] **Tablas, una última pasada (2026-10-03):** `pdf_grid` lee las líneas del PDF (rectángulos delgados rellenos, segmentos, cajas) y arma la rejilla: celdas largas, de varias líneas o combinadas salen bien; se distingue una tabla de datos de las franjas de diseño de la página (pie de página, marcos); el texto girado de las etiquetas verticales se lee como palabra, aunque se pase de su celda. Sin bordes, `pdf_rows` lee por la posición del texto, y las columnas de texto corrido se leen columna por columna. Probado en los PDF reales de Alsea, Bienestar y Monte de Piedad.
- [x] `identidad.tipo_de_documento` (schema 1.1.0): el documento dice cómo se llama a sí mismo y el código lo clasifica (convocatoria, reglas de operación, lineamientos, aviso, guía, formato, anexo, otro). Solo describe; la pantalla avisa si no parece convocatoria pero no bloquea nada.
- [x] Validación a ciegas con las convocatorias de Monte de Piedad 2024, 2025 y 2026 y dos documentos que no son convocatoria (ver ADR-015, «Validación final»).
- [ ] El chat arranca con el resumen «esto es lo que entiendo de la convocatoria», pregunta solo lo dudoso y formula la primera pregunta con ese contexto. (Siguiente paso: el documento canónico y el resumen ya están guardados por convocatoria; falta elegir la convocatoria del proyecto y llevarla al diagnóstico.)
- [x] Agregar documentos de convocatoria (PDF, Word, Excel) → texto por página, fragmentos y FTS (hecho). Vectores: la recuperación por significado se hace al leer, con embeddings que no se guardan.
- [x] **El proyecto nace de su convocatoria (ADR-016, 2026-10-03):** un solo alta («Mis proyectos») con los archivos de la convocatoria y el carácter de cada uno, nombre, quién convoca y año; crea proyecto + lectura en una transacción. Migración 0005 (`project.kind`, `project.call_reading_id`, `call_reading.funder/year`, `call_reading_file.role`), `project_create_from_call`, `project_delete`, panel «La convocatoria de este proyecto» con aviso si lo escrito no coincide con lo leído. La pestaña «Convocatorias» desaparece; «Documentos» queda solo para los de la institución. Proyectos sin convocatoria (`internal`): puerta abierta, apagada.
- [x] **Diagnóstico como conversación guiada con IA (ADR-017, 2026-10-03):** se quitan las 7 preguntas fijas. Etapas reordenadas (`PROFILE → CALL_SELECTION → DIAGNOSIS → …`); `CALL_SELECTION` = confirmar la convocatoria (condición calculada desde la lectura, `call_reading.confirmed_at`); apertura IDEA–OBSTRUCCIÓN–BENEFICIO con el contexto de la convocatoria y el perfil, hasta 5 porqués, causa de fondo propuesta y confirmada por la persona; solo con IA (si falla, «Intentar otra vez»); pantalla de chat. Migración 0006, `domain/conversation.rs`, `conversation_service.rs`, tarea `conversation.turn`, prompts `conversation_turn.v1` y `diagnosis_summary.v2`. Probado en seco (servicio Gemini simulado y batería de personas simuladas); 292 pruebas de Rust y 41 del frontend.
- [ ] **SIGUIENTE:** (1) una corrida real con tope de llamadas (`cargo test golden_case_live -- --ignored --nocapture`, hasta 7 ligeras y 1 fuerte) y una prueba piloto con una directiva real; (2) priorización como «elegir el objetivo» a partir de la causa de fondo; (3) requisitos tipificados desde el documento canónico para cruzarlos con el proyecto; (4) bitácora con solidez por hecho (ADR-010 §4–5).
- [ ] Extracción de requisitos → confirmación humana → `call_template`: los requisitos tipificados salen del documento canónico que ya se guarda (monto máximo y mínimo, contrapartida, duración, fecha de cierre, documentos que piden, quién puede participar), sin otra llamada al modelo; la persona los confirma y queda `origin` y `confirmed_at`.
- [ ] Checklist de requisitos tipificados (el código evalúa monto, contrapartida, duración, fecha y documentos contra el proyecto y el perfil).
- [ ] **Ideas tomadas de la comparación con NotebookLM (2026-10-03)** — no se hace ingeniería inversa; se aprovechan estas tres y se usa NotebookLM solo como segunda opinión manual:
  - [ ] **Guardar los embeddings** de las páginas de cada convocatoria en la base (`sqlite-vec` ya está). Hoy se recalculan en cada lectura. *Cuándo:* junto con la pregunta libre; es su requisito.
  - [ ] **Preguntarle libremente a la convocatoria**, con respuesta y cita verificada por el código (la recuperación por significado y la verificación de citas ya existen). *Cuándo:* con el chat que arranca con el resumen (rebanada 4 de `docs/10`); sirve también para resolver las dudas que quedan en el resumen.
  - [ ] **Citas que se pueden abrir:** en lugar de «archivo, página N», ver el pasaje resaltado en su página. *Cuándo:* con la pantalla del checklist, donde cada requisito debe mostrar su fuente.
  - [ ] **Segunda opinión manual:** subir NMP 2026 a NotebookLM, hacerle las preguntas del schema y comparar con lo que se leyó (en especial `nombre` y las listas largas). *Cuándo:* antes de la demostración a la directora; cuesta una tarde y no necesita código.
- [ ] Explicación de brechas en lenguaje sencillo.
- [ ] Convocatoria ficticia en `fixtures/` basada en la estructura pública de convocatorias reales.

**Terminado cuando:** con la convocatoria ficticia, el checklist detecta correctamente un monto excedido y un documento faltante.

> **Cambio de alcance (ADR-011):** Word y Excel pasan a **solo lectura**. La IA los usa para guiar a la persona y esta los llena a mano; así la responsabilidad del resultado es suya y el trabajo pesado (conceptualizar el proyecto) ya queda hecho en el chat. Quedan en pausa: la plantilla propia y el marcado de plantillas del donante, la generación del `.docx` final (Fase 4) y el llenado de cuestionarios Excel (Fases 5 y 5.5). El código de lectura y escritura de `documents/` se conserva.

## Fase 4 · Redacción y guía en Word (ADR-018)

- [x] Requisitos tipificados de la convocatoria desde el documento canónico (`requirements.rs`).
- [x] Objetivo: la priorización parte de la causa de fondo y de lo que financia la convocatoria.
- [x] Secciones del proyecto: plan por código (base fija o las que pide la convocatoria), redacción con IA, edición con escáner y confirmación por sección.
- [x] Presupuesto y cronograma con cálculos y validaciones en código (`budget.rs`, `schedule.rs`), con confirmación que se pierde si algo cambia.
- [x] Revisión automática (`checklist.rs`): errores que bloquean y avisos que no.
- [x] Guía en Word armada por el código (`docx::build`, `guide_service`) con escaneo previo, sin metadatos de plantilla; se abre con `python-docx`.
- [x] Recorrido completo en seco contra el servicio Gemini simulado: conversación → resumen → objetivo → redacción → revisión → guía.
- [ ] Abrir la guía en Word y revisar cómo se ve (`CIMIENTO_SAMPLE_DIR=… cargo test write_a_sample_guide -- --ignored --nocapture`).
- [ ] Cotizaciones ligadas a partidas (`budget_item.quote_document_id`).

**Terminado cuando:** el caso dorado genera una guía que abre bien en Word, con cifras correctas y sin datos sin confirmar.

## Fase 5 · Excel

~~Lectura y llenado de cuestionarios Excel~~ → fuera de alcance (ADR-011 y ADR-018): el formato del donante no se llena en Cimiento. El código de lectura de `documents/xlsx.rs` se conserva.

## Fase 6 · Listo para instalar (ADR-019)

- [x] Respaldo cifrado con contraseña y restauración (conserva lo anterior).
- [x] PIN opcional de pantalla.
- [x] Pantalla de ayuda sencilla.
- [x] Escaneo de la base completa (solo conteos).
- [x] Revisión de textos nuevos contra `08-estilo-redaccion.md` (la prueba de jerga del frontend los cubre).
- [x] Instalador para Windows: `pnpm tauri build --bundles nsis` compiló (2026-10-03): `src-tauri/target/release/bundle/nsis/Cimiento_0.1.0_x64-setup.exe`, 7.5 MB. Falta instalarlo en una computadora limpia.
- [x] **Diseño visual aplicado (2026-10-03):** sistema claro con azul #2B63E0, Nunito, tarjetas, navegación lateral, indicador «paso N de 6», chat con burbujas, barras de acción fijas y textos más cortos en usted. Se ajusta con lo que salga de las pruebas. Falta regenerar el instalador con este diseño.
- [ ] **Instalar en una computadora limpia** y que una persona no técnica complete el caso dorado sin ayuda.
- [ ] Probar la restauración de un respaldo hecho en otra computadora.
- [ ] **Lectura de convocatorias en paralelo para cuando haya API de paga:** hoy los bloques se leen uno tras otro (4 a 8 minutos por convocatoria con el ritmo de prueba). Los 8 bloques de la primera pasada y los de la segunda lectura se pueden lanzar a la vez, con un tope de llamadas simultáneas y con los límites por modelo subidos en la configuración; bajaría a cerca de un minuto sin cambiar el costo. Verificar antes que el contador de uso y el control de ritmo aguanten llamadas simultáneas; probar en seco con el modelo simulado.

**Terminado cuando:** se instala en una computadora limpia y un usuario no técnico completa el caso dorado sin ayuda.

## Fase 7 · Entrega y datos reales

**Avance (2026-10-05):** demostración y uso real con dos instituciones, con API de pago y solo agregaciones hacia la IA. Detalle, límites y cambios derivados en `12-validacion-con-instituciones.md`.

- [ ] **Cada porqué recapitula** la premisa anterior con contexto breve («Como la camioneta les consume gran parte del presupuesto, entonces… ¿por qué…?») y la pantalla explica por qué la causa de fondo pesa más que la primera respuesta (prompt `conversation_turn` nuevo y prueba con personas simuladas; retroalimentación de las instituciones).
- [ ] **Módulos para digitalizar procesos:** más detalle en el padrón de empleados y beneficiarios y tableros de cifras (ADR-020). El detalle por persona sigue sin salir del equipo.
- [ ] Título de la convocatoria inconsistente en 2 de 7 convocatorias de la JAP y la JAPEM (campo `nombre`, ya conocido en ADR-015): proponer el nombre con más de una fuente y mostrarlo siempre para confirmar.

- [ ] Revisión legal de privacidad (ver `03-gobernanza-datos.md`).
- [ ] Revisión de términos del proveedor de IA.
- [ ] Demo a directivos; ajustar parámetros (pesos de priorización, trato tú/usted, plantillas).
- [ ] Cargar convocatorias y formatos reales.

## Fase 8 · Mejoras (opcional)

- NER local para nombres.
- Proveedor Ollama para tareas Light.
- Tablero de gestión con indicadores del perfil (efecto indirecto en la gestión).
- Migración a la nube si se justifica.
- [x] (2026-10-02) Riel de convocatorias abandonado (ver `docs/huella-riel-de-convocatorias.md`). Schema canónico `schemas/canonical_call.schema.json` y pipeline `documents/canonical/` (contrato, paquete de documentos, recuperación de páginas por campo, normalización, verificación de citas, ensamble y validación). ADR-015.
- [x] Comparar las dos formas de lectura (una llamada o una por bloque, con o sin recuperación de páginas) sobre los siete paquetes reales con Flash-Lite; decidido: por bloque con embeddings; el código del riel se retiró (2026-10-03).

## Fase 9 · Monolito modular: núcleo y módulos (ADR-032)

Objetivo: tratar la app como un ERP de la institución. «Inicio» y «Mi institución» forman el núcleo; Personal, Beneficiarios, Instalaciones, Finanzas y Proyectos son módulos independientes que se conectan solo por su `api`. Un bloque por commit, con las pruebas en verde. Proyectos se separa sin lógica nueva (su desarrollo queda en pausa).

- [x] **B0** ADR-032, arquitectura, principios y glosario; prueba de fronteras (`cargo test architecture`) con la lista de deuda de hoy (28 dependencias que van al revés).
- [x] **B1** Revisar textos y anotarlo en la bitácora pasa a la base (`scanner::guard::screen_texts`); `core::institution::kind` y `Flavor::from_kind` (nadie más lee el tipo de la tabla); los ejemplos y los tipos de las pantallas por el `api` y el `service` de cada módulo. Quedan 21 dependencias en la lista de deuda.
- [x] **B2** `hr`, `care` y `facilities` a `src-tauri/src/modules/`; la prueba de frontera de cada módulo revisa su nueva carpeta.
- [x] **B3** Módulo de Finanzas (`modules/finance/`, tablas `fin_*`, migración 0020, comandos `finance_get` y `finance_save`); el núcleo le pasa la nómina y las cuotas ya sumadas (`finance_service`), el paso «Dinero» del primer inicio y la ficha de la IA leen el módulo y dicen lo mismo que antes; en «Mi institución» las tarjetas de dinero guardan en el módulo y el gasto aproximado tiene su propia ventana.
- [x] **B4** `core/` (acceso, primer inicio, perfil, documentos, seguridad, puentes con cada módulo, tableros, ficha de la IA, `error`, `screen` y `core::api`); la conexión compartida y el escáner de documentos públicos pasan a la base; el borrado aprobado de un proyecto lo hace el comando. Quedan 12 dependencias en la lista, todas de Proyectos (B5).
- [ ] **B5** Proyectos a `modules/projects/` con `ProjectsError`. La lista de deuda queda vacía.
- [ ] **B6** Frontend en `src/core/` y `src/modules/`, módulos en el riel, «Mi institución» con resúmenes; notas «Para cloud».

**Terminado cuando:** la lista de deuda de `architecture_tests.rs` está vacía, todas las pruebas pasan y cada módulo se abre desde el riel.
