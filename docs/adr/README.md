# Registro de decisiones de arquitectura (ADR)

Cada decisión importante se documenta aquí. Formato: contexto, decisión, consecuencias, estado. Una decisión no se edita: si cambia, se crea un ADR nuevo que la reemplaza.

| ADR | Decisión | Estado |
|---|---|---|
| 001 | Tauri 2 como contenedor de escritorio | Aceptada |
| 002 | SQLite + SQLCipher local primero | Aceptada |
| 003 | La IA propone, el código decide y escribe | Aceptada |
| 004 | Office en Rust, con Python como plan B | Aceptada (falta revisión visual en Office) |
| 005 | Proveedor de IA intercambiable con niveles | Aceptada |
| 006 | Llamadas a la IA por HTTP directo con salidas estructuradas | Aceptada (falta llamada real) |
| 007 | Gemini como proveedor principal por ahora (escenario B), con límites y métricas | Aceptada (falta llamada real) |
| 008 | Cadena de modelos de respaldo (un 503 del modelo principal ya no deja sin resumen) | Aceptada (la cadena actuó con 503 reales; falta verla terminar en un 200 de respaldo) |
| 009 | `gemini-3.5-flash` como modelo fuerte principal (3.6, 3.7 y 3.8 devolvían 503) | Aceptada |
| 011 | Ingesta en silencio de la convocatoria (reglas + un modelo que revisa una vez, con citas verificadas por código) y Word/Excel solo de lectura | Aceptada (falta correr la revisión con el modelo real) |
| 012 | `gemini-3-flash-preview` como modelo fuerte principal (respondió en 8 s mientras los demás daban 503) | Aceptada (falta repetir la revisión de convocatoria con él) |
| 013 | Riel de convocatorias: el código encuentra los candidatos, el modelo solo los etiqueta (resultado igual con cualquier modelo) | Reemplazada por la 015 (código archivado en `docs/codigo-archivado/`) |
| 014 | Decisión estratégica: riel de convocatorias o lectura completa estructurada (y si los embeddings hacen falta) | Reemplazada por la 015 |
| 015 | Schema canónico de convocatorias: un contrato que un modelo pequeño rellena con citas y que el código verifica; lectura por bloque con embeddings, conectada al producto | Aceptada y conectada a la pantalla «Convocatorias» (2026-10-03) |
| 016 | El proyecto nace de su convocatoria (archivos con su carácter, nombre, donante y año en un solo paso); institución global, proyectos independientes; proyecto sin convocatoria con la puerta abierta pero apagada | Aceptada (2026-10-03); falta reordenar las etapas |
| 017 | El diagnóstico es una conversación guiada con IA (apertura IDEA–OBSTRUCCIÓN–BENEFICIO + cinco porqués), con la convocatoria confirmada antes; solo con IA; interfaz de chat | Aceptada (2026-10-03); falta corrida real y prueba piloto |
| 018 | Redacción, revisión y guía en Word como entregable: el formato del donante no se llena; el código arma un Word con las conclusiones, estructurado según lo que pida la convocatoria | Aceptada (2026-10-03); falta abrir la guía en Word |
| 019 | Respaldo cifrado con contraseña, PIN opcional de pantalla y escaneo de la base completa | Aceptada (2026-10-03); falta probar en una computadora limpia |
| 020 | Padrón de personal y de beneficiarios: fichas individuales que no salen de este equipo; la IA solo ve agregados | Aceptada (2026-10-04); implementada |
| 021 | El borrador lo prepara la IA; la persona revisa, pone costos y corrige | Aceptada (2026-10-04); implementada, falta probar con el modelo real |
| 022 | Espacio de trabajo: el chat al centro y la convocatoria como índice en un panel; los pasos siguientes son partes del chat y lo que hace avanzar vive en el pie del panel; objetivos: 3 ordenados por la IA, sin calificar | Aceptada (2026-10-04); hechas las fases A a E (completo) |
| 023 | Los procesos de IA sobreviven a la navegación y la IA lee una ficha completa de «Mi institución» (sin sueldos ni cuotas por persona) | Aceptada (2026-10-04); falta corrida real |
| 024 | Dos capas: lo que consume la IA no es lo que ve la persona (la «Ficha» de la convocatoria) | Aceptada (2026-10-04) |
| 025 | Identidad visual: bandejas sobre el fondo de la ventana, color con significado, tokens en `src/index.css` como única fuente y componentes únicos en `src/components/ui/`, con una prueba que impide valores sueltos (`docs/13-sistema-visual.md`) | Aceptada (2026-10-07); se aplica por capas |
| 026 | Ingresos por tipo (cuotas del padrón, donantes fijos, ocasionales, por proyecto), egresos en lista o con un aproximado exprés, nómina con aguinaldo y prima vacacional, y balance; la IA no recibe nómina ni cuotas como cifra | Aceptada (2026-10-07); backend hecho, falta el diseño de la pantalla |
| 010 | Conversación guiada con bitácora estructurada, convocatoria primero | **Propuesta**; la convocatoria primero y la conversación progresiva se adoptaron en el ADR-017, la bitácora completa queda pendiente (ver `docs/10-metodologia-conversacion.md`) |
