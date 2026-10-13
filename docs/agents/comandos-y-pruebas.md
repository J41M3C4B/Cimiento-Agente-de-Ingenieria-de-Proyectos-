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
cargo test core::agents::assistant                     # el Asistente (ADR-034 §4, IA3): batería en seco por pantalla, contexto, cuarentena, sin IA responde el manual, lugares y fuentes que el código conoce
cargo test assistant_live -- --ignored --nocapture     # 3 preguntas reales desde 3 pantallas con la llave guardada en la app (hasta 5 pasos cada una); imprime respuesta, pasos y costo
cargo test core::agents::capturist                     # el Capturista (IA4): ninguna sugerencia se guarda sin el clic; lo inventado, lo protegido y lo que no cabe se descarta; lo aceptado pasa por validación, escáner e historial con su origen
cargo test capturist_live -- --ignored --nocapture     # un párrafo real para «Su institución» (1 llamada); imprime las sugerencias y el costo. Guarde la salida en un archivo: no la filtre
cargo test core::agents                                # herramientas del núcleo (estado de llenado, partes de la ficha), permisos, lo que nunca llega a la IA, y ai_proposal
cargo test core::manual                                # manual del programa (ADR-034): cada campo del catálogo con su entrada, lenguaje sencillo y búsqueda sin conexión
cargo test agent_tools_live -- --ignored --nocapture   # encargo real con las dos herramientas del núcleo (a lo más 4 pasos); imprime pasos, texto y costo
cargo test ai_sheet                                    # la ficha de «Mi institución» que lee la IA: qué lleva, qué nunca (ADR-023)
cargo test finance                                     # módulo de Finanzas (ADR-026, ADR-032): ingresos por tipo, egresos, balance, escáner y guardado; la nómina con prestaciones está en `cargo test profile`
cargo test hr::                                        # módulo de Personal (ADR-027): modalidades, CURP/RFC/NSS/CLABE, avance, agregados y su frontera
cargo test access                                      # perfiles de acceso (ADR-028): roles, permiso de cada comando, cuentas, bloqueo, recuperación y solicitudes de borrado
cargo test care                                        # módulo de Beneficiarios (ADR-029): ficha, grupos, lista de espera, indicadores, tablero y lo que llega a la IA
cargo test overview                                    # la institución de un vistazo (ADR-033): cifras de cada módulo, cifra rápida, ocupación, avance y lo que falta
cargo test core::history                               # origen por dato e historial ligero (ADR-033 §4): solo lo que cambia, lo protegido sin valor, cómo estaba en una fecha, solo crece
cargo test profile::forms                              # las cuatro ventanas de «Mi institución»: guardado, reglas, Junta, escáner e historial en la misma transacción
cargo test core::snapshot                              # foto mensual (ADR-033 §4): solo cifras, el mes en curso se actualiza, un mes pasado queda fijo y nada se borra
cargo test documents::reading                          # leer escaneos y fotos (OCR, ADR-035): páginas sin texto dibujadas y leídas, la «@» de un correo, una página desteñida «con dudas», las celdas de una factura separadas con « | », la letra chica de una captura y de una foto se agranda y se lee, una página con letra normal no se relee, la carpeta temporal se borra
cargo test core::archive::search                       # búsqueda en los documentos (IA5): por palabras con el expediente ficticio, fusión, extractos, y la pregunta sale solo limpia
cargo test core::archive::vectors                      # vectores del expediente (IA5): una vez, cada solicitud en ai_usage, otro modelo los rehace, el tope, el respaldo y el borrado
#   la medición de IA5 con vectores reales (solo texto ficticio; ~170 textos, unas 7 solicitudes; con CIMIENTO_VEC_CACHE guarda los vectores y la siguiente vez no pide nada):
#   $env:CIMIENTO_VEC_CACHE="D:\...\vectores.json"; cargo test search_live -- --ignored --nocapture > salida.txt
#   lo que entiende el Lector de un archivo real, sin guardar nada ni imprimir su texto (páginas, tiempo, tipo y largo de cada dato, y qué datos protegidos halla el escáner por página y si lo toma por lista de personas):
#   $env:CIMIENTO_READER_FILE="D:\...\acta.pdf"; cargo test reader_on_a_file -- --ignored --nocapture > salida.txt
cargo test core::reader                                # el Lector (IA6a): el tipo de cada documento ficticio por sus palabras, fechas escritas con palabras, los datos de la constancia, el oficio, el acta y el poder, el RFC aunque lo guardado esté tapado, y lo que dijo la persona no lo cambia una regla
cargo test core::archive                               # Documentos: texto pegado y archivos (PDF, Word, Excel, fotos) con escáner, solo el texto limpio y las páginas leídas de imagen
cargo test core::records                               # expediente (ADR-033 §5, N4): qué documentos tocan a cada institución, estado y vigencia, folio protegido, PDF y Word con escáner, escaneados y fotos, solo el texto limpio y dónde está el original
cargo test core::examples                              # los dos ejemplos I.A.P. cargan completos al 100 % y sin nombres, folios ni CLABE hacia la IA
cargo test core::protected                             # patronato y cuentas (ADR-033 §3): qué se pide, CLABE tapada y en bitácora, historial sin valor, lo que llega a la IA, escáner y lo que falta
cargo test forms                                       # formularios descritos como datos (ADR-033): validador, perfil de atención, «Su institución» y las palabras de cada campo en es-MX.ts
cargo test onboarding                                  # primer inicio (ADR-031): pasos, cifras rápidas, cierre, bienvenida y lo que llega a la IA
cargo test facilit                                     # módulo de Instalaciones (ADR-030): inmueble, grupos con conteo por estado, traslado, tablero, escáner y lo que llega a la IA
cargo test bridge::staff                               # el personal con la base real: lo que llega al perfil y a la IA, identificadores tapados, puestos con escáner
cargo test projects::drafting                          # redacción, presupuesto y cronograma (ADR-018), con modelo simulado
cargo test projects::guide                             # revisión y guía en Word (ADR-018): escribe archivos en carpetas temporales
cargo test core::security                              # PIN, escaneo de la base, respaldo y restauración (ADR-019)
# una guía de muestra para abrirla en Word a mano:
#   $env:CIMIENTO_SAMPLE_DIR="D:\...\muestra"; cargo test write_a_sample_guide -- --ignored --nocapture
pnpm tauri build --bundles nsis                        # instalador de Windows (con Strawberry Perl antes que el de Git en el PATH)
cargo test pdf_grid                                    # tablas con bordes por las líneas del PDF (sin gasto)
cargo test pdf_rows                                    # tablas sin bordes por la posición de las letras (sin gasto)
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
