# Comandos y pruebas

Mantén este documento actualizado.

```bash
pnpm install          # dependencias
powershell -ExecutionPolicy Bypass -File scripts/fetch-ocr.ps1   # una vez: Tesseract, sus modelos y Pdfium en src-tauri/resources/ocr (OCR, ADR-035); sin esto no compila el instalador ni pasan las pruebas de OCR
pnpm tauri dev        # desarrollo
# probar el primer inicio desde cero sin tocar los datos ni los ejemplos (solo en desarrollo):
#   $env:CIMIENTO_DATA_DIR="D:\Agente Proyectos\cimiento-vacio"; pnpm tauri dev
pnpm build            # compila el frontend (tsc + vite)
pnpm test             # pruebas del frontend (Vitest)
cargo test            # pruebas Rust (desde src-tauri/)
cargo test architecture                                # capas del monolito modular (ADR-032): cada capa solo mira hacia abajo y solo el dueño de una tabla escribe SQL sobre ella; listas de deuda que solo se achican
cargo test canonical                                   # lectura canónica (ADR-015): contrato, paquete, recuperación, normalización, ensamble, resumen y estrategias; sin red ni gasto
cargo test projects::calls                             # crear un proyecto desde su convocatoria y leerla en segundo plano, con modelo simulado (sin red ni gasto)
cargo test conversation                                # la conversación del diagnóstico (ADR-017): reglas puras, servicio con modelo simulado y batería de personas simuladas (sin red ni gasto)
cargo test card                                        # la ficha de la convocatoria para la persona y su resumen con IA (ADR-024), con modelo simulado
#   la ficha de una convocatoria real, sin llamadas (a ojo):
#   $env:CIMIENTO_CANON_FILE="D:\...\x.canonico.json"; cargo test print_card_of_a_real_call -- --ignored --nocapture
cargo test jobs                                        # un proceso de IA por proyecto a la vez (ADR-023)
cargo test ai::agent                                   # bucle de agentes (ADR-034): herramientas, rechazo de lo no permitido, topes, escáner y ai.run, con modelo simulado
cargo test core::agents::assistant                     # el Asistente (ADR-034 §4, IA3; C3): batería en seco por pantalla, contexto, el mapa del Expediente (por tipo, nunca el nombre del archivo), le pregunta al Lector y cita solo lo que el Lector entregó, «Llenar con mis documentos», cuarentena, sin IA responde el manual
cargo test core::reader::quote                         # la revisión de una cita, la misma para todo agente (C4): sin acentos ni signos, un desliz de copia perdonado en citas largas, nunca una cifra distinta, y se guardan las palabras del texto
cargo test core::reader::ask                           # el Lector contesta una pregunta (C3): pasajes por tipo, en un documento nombrado, la cita y la cifra revisadas por el código, sin documentos o sin IA
cargo test assistant_battery_live -- --ignored --nocapture > assistant_battery_live.txt   # la batería del Asistente (fixtures/expediente/asistente.json, 16 preguntas, tres con conversación previa) con el modelo más barato; cada paso, veredicto y costo. Con $env:CIMIENTO_ROUNDS="2" la repite (unos $0.50 MXN por ronda)
cargo test assistant_native_vs_json_live -- --ignored --nocapture > native_vs_json.txt   # la medición del ADR-036 §7: la misma batería con nuestro protocolo JSON y con las funciones nativas de Gemini (solo medición; no está en la app)
cargo test letters_live -- --ignored --nocapture > letters_live.txt   # la frase de la prueba de la institución («Nos representa la directora…») 8 veces al Asistente con el modelo más barato (ADR-042 §6): cada respuesta tal cual y las palabras con una cifra entre letras («dej3»). Con $env:CIMIENTO_ROUNDS lo repite otro número de veces
cargo test assistant_live -- --ignored --nocapture     # 4 preguntas reales desde 4 pantallas con la llave guardada en la app (hasta 5 pasos cada una); imprime respuesta, pasos y costo
cargo test core::agents::capturist                     # el Capturista (IA4, C4): ninguna sugerencia se guarda sin el clic; lo inventado, lo protegido, lo que no cabe y una cifra que no está en las palabras de la persona se descarta; lo aceptado pasa por validación, escáner e historial con su origen; la batería en seco tira todas sus trampas; los datos propios de la institución se copian letra por letra y fecha dicha (ADR-041), y lo que la persona aclaró ante el escáner le llega
cargo test capturist_battery_live -- --ignored --nocapture > capturist_battery_live.txt   # la batería del Capturista (fixtures/capturista.json, 15 casos en las cuatro ventanas) con el modelo más barato: lo que contestó el modelo, lo que el código dejó y tiró, el veredicto por campo y lo que el programa le dice a la persona. Con $env:CIMIENTO_ROUNDS="2" la repite (unos $0.28 MXN por ronda)
cargo test core::agents::proposals                     # el único punto de toda sugerencia: valor que cabe, escáner, la política de ADR-038 y cómo se escribe cada campo (ADR-042, fase 5: el municipio con su nombre del catálogo del INEGI, la calle sin «Calle»)
cargo test core::agents                                # herramientas del núcleo (estado de llenado, partes de la ficha), permisos, lo que nunca llega a la IA, y ai_proposal
cargo test core::manual                                # manual del programa (ADR-034): cada campo del catálogo con su entrada, lenguaje sencillo y búsqueda sin conexión
cargo test places                                      # el mapa de lugares (ADR-039): páginas desde el manual y la barra, ventanas desde los formularios, el «@» de cada duda, quién puede abrir cada página, y el botón que nunca lleva a donde ya está la persona
cargo test agent_tools_live -- --ignored --nocapture   # encargo real con las dos herramientas del núcleo (a lo más 4 pasos); imprime pasos, texto y costo
cargo test ai_sheet                                    # la ficha de «Mi institución» que lee la IA: qué lleva, qué nunca (ADR-023)
cargo test finance                                     # módulo de Finanzas (ADR-026, ADR-032): ingresos por tipo, egresos, balance, escáner y guardado; la nómina con prestaciones está en `cargo test profile`
cargo test hr::                                        # módulo de Personal (ADR-027): modalidades, CURP/RFC/NSS/CLABE, avance, agregados y su frontera
cargo test access                                      # perfiles de acceso (ADR-028): roles, permiso de cada comando, cuentas, bloqueo, recuperación y solicitudes de borrado
cargo test care                                        # módulo de Beneficiarios (ADR-029): ficha, grupos, lista de espera, indicadores, tablero y lo que llega a la IA
cargo test overview                                    # la institución de un vistazo (ADR-033, ADR-042): cifras de cada módulo, cifra rápida, ocupación, y una sola cuenta del avance (los indispensables primero, sin fichas de módulos)
cargo test core::history                               # origen por dato e historial ligero (ADR-033 §4): solo lo que cambia, lo protegido sin valor, cómo estaba en una fecha, solo crece
cargo test profile::forms                              # las cuatro ventanas de «Mi institución»: guardado, reglas, Junta, escáner e historial en la misma transacción
cargo test core::snapshot                              # foto mensual (ADR-033 §4): solo cifras, el mes en curso se actualiza, un mes pasado queda fijo y nada se borra
cargo test documents::reading                          # leer escaneos y fotos (OCR, ADR-035): páginas sin texto dibujadas y leídas, la «@» de un correo, una página desteñida «con dudas», las celdas de una factura separadas con « | », la letra chica de una captura y de una foto se agranda y se lee, una página con letra normal no se relee, un PDF, Word o Excel con contraseña lo dice, la carpeta temporal se borra
cargo test core::archive::search                       # búsqueda en los documentos (IA5): por palabras con el expediente ficticio, fusión, extractos, y la pregunta sale solo limpia
cargo test core::archive::vectors                      # vectores del expediente (IA5): una vez, cada solicitud en ai_usage, otro modelo los rehace, el tope, el respaldo y el borrado
#   la medición de IA5 con vectores reales (solo texto ficticio; ~170 textos, unas 7 solicitudes; con CIMIENTO_VEC_CACHE guarda los vectores y la siguiente vez no pide nada):
#   $env:CIMIENTO_VEC_CACHE="D:\...\vectores.json"; cargo test search_live -- --ignored --nocapture > salida.txt
#   lo que entiende el Lector de un archivo real, sin guardar nada ni imprimir su texto (páginas, tiempo, tipo y largo de cada dato, y qué datos protegidos halla el escáner por página y si lo toma por lista de personas):
#   $env:CIMIENTO_READER_FILE="D:\...\acta.pdf"; cargo test reader_on_a_file -- --ignored --nocapture > salida.txt
cargo test core::reader::kinds                         # el tipo de un documento por sus palabras y si un archivo es del lugar donde se subió (ADR-042 §4): cada documento de la batería solo en su lugar, el nombre del archivo nunca le gana al texto
cargo test kind_confidence_measure -- --ignored --nocapture   # la medición del umbral de ADR-042 §4: los archivos de la prueba completa (con OCR, cerca de un minuto) y de la batería, en su lugar y en todos los demás, con cada peso del nombre y umbral. Con $env:CIMIENTO_KIND_SHOW="08" muestra qué dice el archivo 08
cargo test core::reader                                # el Lector (IA6a) y la política de llenado (ADR-038): batería de los datos propios de 13 documentos, campo lleno, rechazado, autoridad, reemplazo, lugar equivocado; el tipo de cada documento ficticio por sus palabras, fechas escritas con palabras, los datos de la constancia, el oficio, el acta y el poder, el RFC aunque lo guardado esté tapado, y lo que dijo la persona no lo cambia una regla
cargo test core::archive                               # Documentos: texto pegado y archivos (PDF, Word, Excel, fotos) con escáner, solo el texto limpio y las páginas leídas de imagen
cargo test core::records                               # expediente (ADR-033 §5, N4): qué documentos tocan a cada institución, estado y vigencia, folio protegido, PDF y Word con escáner, escaneados y fotos, solo el texto limpio y dónde está el original
cargo test core::examples                              # los dos ejemplos I.A.P. cargan completos al 100 % y sin nombres, folios ni CLABE hacia la IA
cargo test core::protected                             # patronato y cuentas (ADR-033 §3): qué se pide, CLABE tapada y en bitácora, historial sin valor, lo que llega a la IA, escáner y lo que falta
cargo test forms                                       # formularios descritos como datos (ADR-033): validador, perfil de atención, «Su institución» y las palabras de cada campo en es-MX.ts
cargo test onboarding                                  # primer inicio (ADR-031, ADR-042): solo los 14 datos indispensables del catálogo (todo campo que decide lo que el programa pide lo es), cada paso por su ventana (reglas, escáner, historial), se cierra solo con el último dato venga de donde venga, cifras rápidas, bienvenida y lo que llega a la IA
cargo test facilit                                     # módulo de Instalaciones (ADR-030): inmueble, grupos con conteo por estado, traslado, tablero, escáner y lo que llega a la IA
cargo test bridge::staff                               # el personal con la base real: lo que llega al perfil y a la IA, identificadores tapados, puestos con escáner
cargo test projects::drafting                          # redacción, presupuesto y cronograma (ADR-018), con modelo simulado
cargo test projects::guide                             # revisión y guía en Word (ADR-018): escribe archivos en carpetas temporales
cargo test core::security                              # PIN, escaneo de la base, respaldo y restauración (ADR-019)
# una guía de muestra para abrirla en Word a mano:
#   $env:CIMIENTO_SAMPLE_DIR="D:\...\muestra"; cargo test write_a_sample_guide -- --ignored --nocapture
pnpm tauri build --bundles nsis                        # instalador de Windows (con Strawberry Perl antes que el de Git en el PATH)
cargo test pdf_grid                                    # tablas con bordes por las líneas del PDF (sin gasto)
cargo test documents::pdf                              # el texto de un PDF con Pdfium (ADR-037): párrafos por la distancia entre renglones, guion de corte, contraseña, tablas con bordes en un formulario anidado
cargo test core::records::background                   # una lectura en segundo plano por documento a la vez (ADR-038)
cargo test full_expediente -- --ignored --nocapture    # la prueba completa del Expediente en seco (C4): los 24 archivos de fixtures/expediente/prueba-completa en su documento, por el mismo camino que la pantalla (escaneos y fotos con OCR, cerca de un minuto); avisos, datos propios, tapados y sugerencias contra esperado.json, con los pendientes conocidos aparte. Los archivos se rehacen con `python generar.py` en esa carpeta
cargo test core::reader::upload                        # las sugerencias al subir un archivo (ADR-042 §5): cada documento ficticio se pregunta por los campos que escribe, el nombre de una opción cuenta, una llamada por archivo con solo sus pasajes, lo que ya espera no se vuelve a preguntar salvo a un archivo con más autoridad, sin IA o sin campos no se llama
cargo test upload_detection_measure -- --ignored --nocapture   # la medición del umbral de ADR-042 §5, en seco: los 13 documentos contra los campos que escribe cada uno, con cada umbral
cargo test upload_live -- --ignored --nocapture > upload_live.txt   # los 13 documentos ficticios subidos uno tras otro con el modelo más barato (a lo más 13 llamadas, unos $0.17 MXN): lo que llegó a cada ventana, su página, inventados, costo y tokens del caché
cargo test core::reader::fill                          # «Llenar con mis documentos» en seco (C2): palabras por campo, la página esperada entre los pasajes (21 de 21), lo que el código descarta, sin documentos o sin IA
cargo test reader_live -- --ignored --nocapture > reader_live.txt   # las 4 ventanas con el modelo más barato sobre el expediente ficticio (4 llamadas y los vectores de los fragmentos); valor, página, inventados y costo por campo. Con $env:CIMIENTO_ROUNDS="2" la repite, cada ronda con un expediente nuevo
cargo test pdf_rows                                    # tablas sin bordes por la posición de las letras (sin gasto)
# qué saca Pdfium de cada PDF de una carpeta (ADR-037): letras, palabras partidas, signos raros, páginas como imagen, párrafos y filas de tabla; sin imprimir el texto salvo que se pida una página (solo con PDF públicos):
#   $env:CIMIENTO_PDF_DIR="D:\...\pdf_pruebas"; cargo test pdf_report -- --ignored --nocapture > salida.txt   (opcional: $env:CIMIENTO_PDF_SHOW="archivo.pdf:3")
# subir en seco los paquetes de un manifiesto, como lo haría una persona (tiempos, páginas, datos tapados; sin llamadas a servicios):
#   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test call_ingest_real -- --ignored --nocapture
# qué tablas salen de un PDF REAL, por bordes y por texto, y qué rejillas se descartan por ser diseño de la página:
#   $env:CIMIENTO_ROWS_PDF="D:\...\archivo.pdf"; $env:CIMIENTO_ROWS_SHOW="1"; cargo test count_tables_by_kind -- --ignored --nocapture
# tamaños en seco (sin llamadas) y lectura real de los paquetes de un manifiesto JSON (ver documents/canonical/live_tests.rs):
#   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test canonical_sizes -- --ignored --nocapture
#   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test canonical_live -- --ignored --nocapture   (reanudable; resultados y resumen.md en la carpeta "out")
#   en el manifiesto: "residual": true (segunda lectura sobre lo poco citado), "timeout_secs", "max_calls" (tope duro de llamadas reales; sin reintentos tras el primer fallo)
#   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test canonical_rows -- --ignored --nocapture      (sin gasto: muestra las filas de tabla de cada página)
#   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test canonical_embed_probe -- --ignored --nocapture  (2 textos al modelo de embeddings)
cargo test dry_run                                     # caso dorado contra un servicio Gemini simulado (no gasta nada)
cargo test gemini_smoke_live -- --ignored --nocapture  # primera llamada real: 1 llamada + comprobación gratuita de llave y modelos
cargo test golden_case_live -- --ignored --nocapture   # caso dorado con IA real (proveedor y llave guardados en la app)
pnpm tauri build      # instalador
```

**Requisito en Windows:** SQLCipher compila OpenSSL desde fuente (`bundled-sqlcipher-vendored-openssl`) y necesita Strawberry Perl **antes** que el `perl` de Git en el `PATH`. La primera compilación tarda ~11 min; las siguientes son rápidas. Ver ADR-002.

## Pruebas con IA real

Todas son opt-in (`#[ignore]`) y llevan tope de llamadas. Antes de gastar cuota se prueba en seco contra el servicio simulado; después se hace una sola corrida real y se registran sus métricas (tiempos, llamadas, costo equivalente) en `docs/09-roadmap.md`.

Las baterías del Asistente, del Capturista y de «Llenar con mis documentos» nunca mandan más de **14 solicitudes por minuto** a cada modelo (`ai::paced`; el plan gratuito permite 15). Cuentan dentro de la corrida, porque cada caso usa una base nueva y el ritmo de la app (que cuenta en `ai_usage`) no ve los casos anteriores. Entre dos baterías seguidas, espera un minuto. El plan gratuito permite además 500 llamadas al día por modelo, y las baterías no las cuentan: una corrida de las tres, en dos rondas, usa unas 110.
