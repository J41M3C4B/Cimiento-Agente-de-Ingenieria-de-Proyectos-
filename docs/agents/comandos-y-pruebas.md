# Comandos y pruebas

Mantén este documento actualizado.

```bash
pnpm install          # dependencias
pnpm tauri dev        # desarrollo
pnpm build            # compila el frontend (tsc + vite)
pnpm test             # pruebas del frontend (Vitest)
cargo test            # pruebas Rust (desde src-tauri/)
cargo test canonical                                   # lectura canónica (ADR-015): contrato, paquete, recuperación, normalización, ensamble, resumen y estrategias; sin red ni gasto
cargo test call_service                                # crear un proyecto desde su convocatoria y leerla en segundo plano, con modelo simulado (sin red ni gasto)
cargo test conversation                                # la conversación del diagnóstico (ADR-017): reglas puras, servicio con modelo simulado y batería de personas simuladas (sin red ni gasto)
cargo test card                                        # la ficha de la convocatoria para la persona y su resumen con IA (ADR-024), con modelo simulado
#   la ficha de una convocatoria real, sin llamadas (a ojo):
#   $env:CIMIENTO_CANON_FILE="D:\...\x.canonico.json"; cargo test print_card_of_a_real_call -- --ignored --nocapture
cargo test jobs                                        # un proceso de IA por proyecto a la vez (ADR-023)
cargo test institution_context                         # la ficha de «Mi institución» que lee la IA: qué lleva, qué nunca (ADR-023)
cargo test finances                                    # ingresos por tipo, egresos, nómina con prestaciones y balance (ADR-026)
cargo test drafting_service                            # redacción, presupuesto y cronograma (ADR-018), con modelo simulado
cargo test guide_service                               # revisión y guía en Word (ADR-018): escribe archivos en carpetas temporales
cargo test security_service                            # PIN, escaneo de la base, respaldo y restauración (ADR-019)
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
