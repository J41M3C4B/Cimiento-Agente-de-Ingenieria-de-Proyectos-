# ADR-015 — Schema canónico de convocatorias (contrato en lugar de riel)

**Estado:** Aceptada y conectada al producto (2026-10-03). Schema v1.1.0 y pipeline en `src-tauri/src/documents/canonical/`; pantalla «Convocatorias»; riel archivado; validación a ciegas hecha (ver «Conexión al producto y cierre de la iteración»).

## Contexto
El riel (ADR-013/014) fallaba por ser específico: el código decidía qué era relevante según el formato de los documentos que teníamos. Ver `docs/huella-riel-de-convocatorias.md`.

## Decisión
Un **contrato universal**, `schemas/canonical_call.schema.json` (JSON Schema 2020-12). Nosotros fijamos qué información necesitamos de toda convocatoria; el modelo la busca y la rellena; el código verifica y normaliza. Cualquier modelo, barato o caro, responde al mismo contrato.

Flujo: documento → ingesta (texto, tablas y páginas; solo PDF con texto, ADR-011) → estructura documental → recuperación por campo → extracción de candidatos (LLM pequeño, sección por sección) → normalización (regex de fechas, montos, porcentajes, duraciones) → validación determinista → resolución de conflictos → JSON canónico.

## Reglas del contrato
1. **Cada dato es su cita:** `cita` literal + `pagina` + `documento`. El código verifica que exista; el texto lo pone la cita, no el modelo.
2. **`no_aparece` y `ambiguo` son respuestas válidas.** Un campo vacío siempre dice por qué. Las reglas `if/then` hacen cumplir: `encontrado` exige evidencia o elementos; `no_aparece` los prohíbe.
3. **El modelo señala, el código normaliza.** Los campos `normalizado` (fecha, monto, porcentaje, duración) los llena el código desde la cita (`x-llena: codigo`); `valor_texto` debe ser subcadena exacta de la cita.
4. **Listas exhaustivas.** Un calendario es una lista de `hitos` con tipo cerrado, no «la fecha de cierre»; los montos viven en `modalidades[]` cuando cambian por línea o categoría.
5. **Un paquete de documentos, no un PDF.** `documento` en cada evidencia; `conflictos[]` guarda todas las versiones de un dato contradictorio y el código no elige.
6. **Válvula de escape:** `otros_hallazgos[]` recoge lo relevante que ningún campo prevé (plazo único, solicitudes incompletas…). Texto libre del modelo solo en `nota` y `dudas`, marcado como supuesto de la IA.
7. **Agnóstico por diseño:** ninguna descripción de campo nombra una institución, formato ni palabras de un convocante. Las descripciones son la instrucción que lee el modelo.

## Qué contiene (v1.0.0)
`metadatos` (llena el código), `identidad`, `temporalidad`, `elegibilidad`, `financiamiento`, `proyecto`, `documentacion`, `evaluacion`, `entrega`, más `conflictos`, `otros_hallazgos` y `dudas`. Parte del borrador de Jaime (`canonical_schema.txt`) con estos cambios: fechas como `hitos[]`, `modalidades[]`, evidencia y estado por campo, `condiciones_del_apoyo`, `limites_participacion`, `prioridades` y la válvula de escape.

## Cómo se mide
Mismos hechos verificados de antes (`referencias/alsea.json` 42, `bienestar.json` 22) con **Flash-Lite, temperatura 0**. El código no se escribe contra esos documentos: ninguna regla de recuperación, normalización o validación puede depender del formato de uno. Primero en seco con modelo simulado; criterio de éxito fijado antes de correr. Preview queda fuera hasta que se reinicie su cuota.

## Decisiones abiertas
- Recuperación por campo (embeddings) o texto completo por sección: se mide primero con texto completo como techo y luego con recuperación.
- El esquema completo pesa ~26 KB: se envía por sección, y un generador quita las anotaciones `x-*` y los `if/then` para proveedores con subconjunto de JSON Schema (Gemini).
- El código del riel se archiva cuando el pipeline nuevo lo sustituya.

## Lo construido (2026-10-02)
`documents/canonical/`, sin una sola regla de formato:
- `contract.rs`: carga el schema, deriva la vista del modelo (sin `x-llena: codigo`, `if/then`, formatos ni patrones; `$ref` o en línea) y las consultas por campo a partir de las descripciones del propio schema.
- `package.rs`: un paquete de PDF, Word y Excel como páginas numeradas en un solo conteo (`[Página N | archivo]`), mapa de páginas (primera línea que no se repite en el documento) y año dominante.
- `retrieve.rs`: qué páginas lee cada bloque. Cada campo es una consulta; se ordenan las páginas por significado (embeddings `gemini-embedding-001`) o por TF-IDF léxico (línea base sin modelo), se fusionan los rankings (RRF) y se toman las mejores hasta un presupuesto (un tercio del paquete, entre 8,000 y 40,000 caracteres) con sus vecinas y la primera página de cada documento.
- `normalize.rs`: lee montos, porcentajes, duraciones, enteros y fechas (rangos entre meses, «Mayo 16 al 23», ISO, numéricas) de la cita. Lector de fechas propio: el del extractor de reglas, hecho para el Monte de Piedad, no leía rangos entre meses ni «Mayo 16 al 23» y asignaba mal el año.
- `assemble.rs`: verifica cada cita contra su página (la reubica si está en otra, la descarta si no está en ninguna), exige que el valor esté escrito en la cita, deja que la evidencia gane a la declaración, conserva contradicciones, rellena `documento` y `normalizado` y valida contra el schema completo.
- `run.rs`: las estrategias a comparar: `oneshot` (todo en una llamada), `block_all` (una llamada por bloque con todo el texto, control), `block_lexical` y `block_embedding` (una llamada por bloque con las páginas recuperadas).
- `live_tests.rs`: banco de medición (manifiesto, reanudable, `resumen.md`).
- Tarea `call.canonical` y `pipeline::run_with` (esquema y tope de salida por llamada); prompt `canonical_read.v1`.

## Hallazgos técnicos
1. **Gemini no acepta el schema completo con salida estructurada.** Cada bloque solo (con `$ref` o en línea) da 200; tres bloques dan 200; cuatro o más dan 400 «invalid argument», aun con 10 a 17 KB (es un límite de complejidad, no de tamaño). La lectura de una sola llamada usa por eso el schema descrito en las instrucciones (`GeminiProvider::with_schema_in_prompt`) y el código lo valida después; el resto usa salida estructurada forzada.
2. Los modelos de embeddings del servicio ofrecen `embedContent` pero no `batchEmbedContents`: el cliente cae a una petición por texto.
3. Los paquetes reales son pequeños (6,500 a 16,400 tokens): con recuperación cada bloque lee ≈ 45 % del paquete y los ocho bloques suman 3.4 a 4.4 veces el paquete. La recuperación solo ahorra de verdad con paquetes de cientos de páginas.
4. Un documento que no es convocatoria (aviso electoral de la JAP) no queda vacío: el modelo coloca su contenido en los campos que encajan (convocante, calendario, requisitos) y deja vacíos los de dinero y proyecto. Falta un campo `tipo_de_documento` que diga qué es el documento sin cerrar la puerta (no se vuelve a hacer una puerta de clasificación, ADR-014).

## Plan de pruebas
Siete paquetes con todos los archivos disponibles: `jap` (control: no es convocatoria), `alsea` (PDF + Word), `bienestar` (cartel, reglas, carta compromiso y formato en Excel), `nmp2024`, `nmp2025`, `nmp2026` y `nmp2026_paquete` (convocatoria + anexo de rubros + guía + indicadores, 50 páginas). Solo `gemini-3.5-flash-lite`, temperatura 0 y semilla 7, 4 llamadas por minuto. Cuatro estrategias por paquete. Se mide: llamadas, tiempo, tokens, costo, citas verificadas (alucinación), citas reubicadas, campos encontrados / ambiguos / no aparece, errores de schema, hechos verificados de Alsea (42) y Bienestar (22) y acuerdo entre estrategias. Nada se afina contra un paquete.

## Resultados (2026-10-02, 27 corridas completas con Flash-Lite)
Detalle en `D:\Agente Proyectos\pruebas-schema\conclusiones.md` y `resumen.md`. Promedio por paquete: una llamada ($0.49, 27 s, 47 elementos, Alsea 33/42, Bienestar 15/22), por bloque con puntaje léxico ($0.86, 58 elementos, 32/42 y 17/22), por bloque con embeddings ($0.90, 67 elementos, **38/42 y 17/22**), por bloque con todo el texto como control ($1.27, 83 elementos, 38/42 y 18/22). Citas verificadas de 87 a 100 %; 0 errores de esquema. Una llamada falló en 1 de 7 paquetes (JSON cortado, reproducible con temperatura 0).

**Decisión:** lectura por defecto = por bloque con embeddings (respaldo léxico sin modelo); `oneshot` solo como vista previa barata; `block_all` descartado. Pendiente: agotar las listas (segunda pasada), `identidad.tipo_de_documento`, validación a ciegas con convocatorias nuevas y con un paquete de cientos de páginas, archivar el riel y conectar el pipeline al producto.

## Diagnóstico y mejoras del proceso (2026-10-03)
Jaime fijó el rumbo: no se busca un modelo más caro, se mejora el proceso para que uno barato baste. Los modelos fuertes (`gemini-3-flash-preview`, `gemini-3.8-flash`) devolvieron 503 «high demand» el 2026-10-02 y no hubo lectura con ellos. El diagnóstico se hizo sobre las 27 corridas ya guardadas, sin gastar cuota: los hechos que faltaban en las cuatro estrategias tenían causas de proceso.

| Hecho que falta | Causa |
|---|---|
| Calendario de Alsea | Tabla aplanada por el extractor de PDF: etiquetas y valores desacoplados; una cita literal no puede unirlos. |
| Obligaciones de Bienestar (lista a–e) | Se recuperaba por página y la página mezcla temas; con todo el texto, el modelo se daba por satisfecho tras los primeros elementos. |

Mejoras (`documents/canonical/`, `documents/pdf_rows.rs`), todas agnósticas:
1. **Cobertura medida por el código** (`assemble.rs`): páginas con alguna cita verificada, proporción de texto citado y páginas «delgadas» (menos de 60 % de su texto dentro de una cita). No necesita hechos de referencia, así que mide cualquier paquete.
2. **Segunda lectura** (`run.rs`, `Options::residual`): cada bloque vuelve a leer solo las páginas delgadas, con los pasajes ya citados tapados con `[…ya extraído…]` y con lo que el bloque ya tiene como contexto. Lo nuevo se mezcla con `Answers::merge` (las listas crecen; un valor único solo llena un campo vacío) y pasa por las mismas verificaciones. El código lleva la cuenta de lo cubierto; el modelo decide qué significa lo demás (la diferencia con el riel). Una página citada una sola vez no cuenta como leída.
3. **Filas de tabla por posición** (`pdf_rows.rs`): se agrupan las letras por bandas verticales y se parten columnas por el hueco; las filas se agregan al final de la página en un bloque propio, sin cambiar el texto. Descarta prosa en columnas, listas con inciso y encabezados repetidos.
4. **Respaldo léxico automático** cuando el servicio de embeddings falla (429, caída): el bloque se elige por TF-IDF y la lectura sigue; el informe lista qué bloques cayeron.
5. Embeddings `gemini-embedding-2` (100 por minuto; acepta lotes: dos textos en una solicitud).

Resultado con Flash-Lite, por bloque con embeddings, 4 llamadas por minuto (hechos verificados a mano):

| | Alsea (42) | Bienestar (22) | costo MXN (Alsea / Bienestar) |
|---|---|---|---|
| línea base (una pasada) | 38 | 17 | 0.90 / 0.91 |
| + segunda lectura | 39 | (corrida inválida: 429 de embeddings) | 1.27 / — |
| + filas de tabla y cobertura por proporción | 41 | 18 | 1.54 / 1.85 |
| + pasajes ya extraídos tapados | **41** | **19** | 1.40 / 2.05 |

Alsea recuperó el calendario y el Atlas de Riesgos; falta una frase («decisión final inapelable» o el Atlas, según la corrida). Bienestar recuperó el CFDI; faltan tres elementos de la misma lista de obligaciones (evidencias, preferencia por empresas mexiquenses, facilidades para las visitas): el modelo los deja aunque la página esté a la vista y las citas no se rechazan. Una sola corrida por combinación: la diferencia de uno o dos hechos no es concluyente.

Pendiente: comprobar si dividir los campos de lista en llamadas más pequeñas agota las listas, medir a ciegas con las NMP (solo cobertura, no hay hechos de referencia), `identidad.tipo_de_documento`, archivar el riel y conectar el pipeline al producto.

## Conexión al producto y cierre de la iteración (2026-10-03)
Se cerró esta etapa: la lectura canónica ya vive en el producto y el riel quedó archivado. Lo que sigue es lo que cambió y por qué.

### Del riel al producto
- **Riel archivado.** `call_extract`, `call_rail`, `call_review` y `call_sheet` pasaron a `docs/codigo-archivado/riel-*.rs.txt`; sus prompts a `docs/prompts-archivados/`; las pruebas manuales del riel salieron de `diagnosis_service_tests.rs` y `dry_run_tests.rs`. Lo que la lectura canónica usaba de ellos (limpieza de texto de PDF, comparar frases) quedó en `documents/text.rs`. La tarea de IA es una sola: `call.canonical`.
- **Subir y leer en silencio** (ADR-011). Pantalla «Convocatorias»: se eligen el PDF y los Word/Excel que lo acompañan. Al subirlos solo se revisan, se limpian y se guardan (2 a 6 s por paquete, medido): texto por página en `document` y `document_chunk` (con buscador), y una fila en `call_reading` (migración 0004). La lectura corre después en segundo plano y nunca bloquea; la pantalla pregunta cada 3 s mientras hay una en curso. Estados: guardada, leyéndola, lista, leída en parte, no se pudo leer. Sin llave, sin internet, sin cupo o sin presupuesto, la convocatoria queda guardada y esperando, con el motivo en palabras sencillas y «Leer otra vez». Un reintento que no lee nada no borra lo que una lectura anterior sí leyó.
- **Qué lee y con qué.** Por bloque con embeddings (`gemini-embedding-2`, solo si el proveedor elegido es Gemini; con otro proveedor o sin servicio de embeddings, puntaje léxico), segunda lectura de lo poco citado, Flash-Lite primero. El código verifica cada cita contra su página y ensambla el documento canónico, que se guarda completo por convocatoria. Costo y tokens de cada lectura quedan en `ai_usage` con la etiqueta `call:<id>`.
- **Qué se muestra** (`documents/canonical/summary.rs`, sin modelo): título, tipo de documento, quién convoca, fechas por etapa, dinero (montos, contrapartida, tope administrativo, tipos de apoyo), 24 listas (quién puede participar, qué se paga y qué no, documentos, criterios de calificación…), contradicciones entre documentos, lo que no se encontró y qué tan completa fue la lectura. Cada línea trae archivo y página.
- **Datos de personas.** El escáner de la institución es demasiado estricto para un documento público del convocante: taparía su teléfono, su correo y los nombres de a quién escribir, que son parte de lo que dice la convocatoria. `PublicDocScanner` tapa solo lo que identifica a una persona (CURP, RFC de persona, clave de elector, CLABE, tarjeta, NSS) y es el mismo escáner antes de guardar y antes de enviar a la IA, para que las citas sigan coincidiendo con el texto guardado. Un archivo que parece una lista de personas se rechaza entero. Medido en las 5 convocatorias reales: 0 tapados.
- **Borrado de emergencia.** Borrar un archivo de una convocatoria borra la lectura que lo cita; borrar la convocatoria borra todos sus archivos y lo que salió de ellos.
- **Compilación de entrega.** `panic = "abort"` en el perfil de release impedía atrapar el fallo de un PDF dañado (`catch_unwind`) y habría cerrado la aplicación; ahora es `unwind`.

### Tablas: una última pasada
Hasta aquí las tablas se reconstruían solo por la posición del texto (`pdf_rows`), lo que falla con celdas largas y con tablas cuyas columnas no se alinean como se espera. Ahora, en este orden:
1. **Tablas con bordes** (`pdf_grid`): se leen las líneas del PDF (rectángulos delgados rellenos —como los dibuja Word—, segmentos y cajas) con un intérprete mínimo de operadores de dibujo; las que se tocan forman una rejilla; cada letra pertenece a la celda que la contiene; los vecinos sin línea entre ellos son una celda combinada; una celda que abarca varias filas se repite en cada una si es corta. Celdas con párrafos enteros, de varias líneas o combinadas salen bien porque la geometría, y no el texto, decide qué es una celda.
2. **Tabla de datos o diseño de la página.** Una rejilla solo cuenta si al menos 4 celdas tienen texto, al menos una cuarta parte de sus celdas están escritas y hay al menos 8 letras (si no, es un pie de página, un marco alrededor de la hoja o el eje de una gráfica). Medido: sin este filtro, NMP 2025 salía con 38 «tablas» en 42 páginas, casi todas el pie de página.
3. **Texto girado** (etiquetas verticales de una columna): cada letra lleva su dirección; el texto girado se lee como palabra y, si es más largo que su celda y se pasa de la última línea, cuenta para la celda más cercana.
4. **Sin bordes**: `pdf_rows` por la posición del texto, ahora con una guarda para columnas de texto corrido (las líneas son largas y siguen la frase de la línea de arriba): se leen columna por columna, sin cruzarlas renglón por renglón. NMP 2024 p. 18 pasó de 17 renglones cortados a 3 celdas, una por categoría.
5. Las letras que ya cayeron en una tabla con bordes no se vuelven a tomar por el método del texto.

En los PDF reales: Alsea (calendario y esquema de cofinanciamiento), NMP 2026 (tabla de documentos requeridos con su nombre, descripción y la marca de cada etapa; tabla de indicadores con la primera columna vertical completa; los rubros), NMP 2024 (glosario, rubros) y NMP 2025 salen fila por fila.

### Verificación de citas, dos pérdidas encontradas al validar
La validación a ciegas mostró dos pérdidas reales en las listas que salen de tablas, corregidas en el código (no en el modelo):
- Elementos cortos («Software», «Seguros», «Becas») se descartaban por tener citas de menos de 12 letras. Ahora una cita corta vale si es **una línea entera o una celda entera** de su página; una palabra suelta dentro de una oración sigue sin probar nada.
- Citas correctas se rechazaban porque el modelo escribe entera la palabra que la página corta con guion al final del renglón («ga- rantizar»). Ahora la cita se compara también con la página con esas palabras unidas; se conserva la cita tal como la escribió el modelo, y una unión que la página no tiene sigue siendo invento.

### `identidad.tipo_de_documento` (schema 1.1.0)
Un campo descriptivo, no una puerta: el modelo copia cómo se llama el documento a sí mismo («Convocatoria», «Reglas de Operación») y el **código** lo clasifica (convocatoria, reglas de operación, lineamientos, aviso, guía, formato, anexo, otro). La pantalla avisa «Este documento no parece ser una convocatoria» cuando no lo es, pero no bloquea nada. Hallazgo de la validación: en 2 de las 3 convocatorias de NMP el título sale de la fuente con las letras separadas («CONV OC A T ORIA»); el clasificador lo lee también sin espacios, solo para palabras largas.

### Validación final, a ciegas (2026-10-03)
Tres convocatorias que no se usaron para escribir ninguna regla (NMP 2024, 2025 y 2026) y dos documentos que no son convocatoria (el anexo de rubros y la guía de información general y financiera de NMP). Flash-Lite, 4 llamadas por minuto, por bloque con embeddings y segunda lectura. No hay hechos de referencia: se mide lo que el código puede medir solo (citas que se encuentran en su página, páginas con alguna cita, parte del texto citado).

Primera corrida y segunda, después de las dos correcciones a la verificación de citas (arriba):

| | citas verificadas | páginas con cita | texto citado | elementos | citas rechazadas |
|---|---|---|---|---|---|
| NMP 2024 (29 pp.) | 88 → **98 %** | 86 → 86 % | 30 → 44 % | 76 → 98 | 14 → 3 |
| NMP 2025 (42 pp.) | 94 → **96 %** | 60 → 74 % | 29 → 32 % | 106 → 147 | 8 → 8 |
| NMP 2026 (30 pp.) | 93 → **99 %** | 47 → 57 % | 26 → 34 % | 200 → 247 | 19 → 2 |

- **Costo y tiempo por convocatoria:** de $1.3 a $2.2 MXN y de 4 a 8 minutos con el ritmo de prueba de 4 llamadas por minuto (16 a 24 llamadas); el límite del servicio para este modelo es mayor (15 por minuto), así que en uso normal será menos. 0 errores de esquema y ninguna caída al respaldo léxico en las dos corridas (la primera, 96 llamadas, todas bien).
- **Qué se arregló gracias a la validación:** las listas que salen de tablas perdían elementos cortos y las citas con palabras cortadas por guion; el título con letras separadas no se clasificaba. Las tres cosas se corrigieron en el código y se midieron.
- **`tipo_de_documento`:** las tres convocatorias salen como «convocatoria» (en 2024 y 2026 el título viene como «CONV OC A T ORIA»). La guía sale como «guía» (la pantalla avisa que no parece convocatoria) y el anexo de rubros no dice qué es a sí mismo, así que el campo queda sin dato; es lo correcto para un campo que solo describe.
- **Lo que no tiene cita:** en NMP 2026 son el índice, las portadas de anexos, el canal de denuncias, el glosario y la descripción de las estrategias de trabajo (el anexo 2): no hay campo del schema para ellos. El porcentaje de páginas con cita es un piso, no una calificación.
- **Lo que sigue sin resolverse (y se deja así a propósito):** (1) el campo `nombre` no es estable con un modelo pequeño cuando la portada es decorativa: con la descripción corregida NMP 2025 sale bien («Convocatoria de Inversión Social 2025»), NMP 2026 sale como «INVERSIÓN SOCIAL» y NMP 2024 sin dato; la persona lo ve y lo corrige; (2) el modelo no siempre agota las listas largas (tres elementos de la lista de obligaciones de Bienestar, una frase de Alsea); (3) no se ha medido con convocatorias de otras instituciones ni con un paquete de cientos de páginas.

### Pendiente después de esta iteración
- Elegir la convocatoria de un proyecto y llevar el resumen al diagnóstico: el chat arranca con «esto es lo que entiendo» y pregunta solo lo dudoso (Fase 3, último punto).
- Confirmación de la persona: hoy el resumen se muestra para revisar; falta guardar lo que confirma (`origin`/`confirmed_at`) cuando el checklist de requisitos lo necesite.
- Cuando llegue una convocatoria de otra institución, correrla con `call_ingest_real` y `canonical_live` antes de tocar nada; no se afina contra ninguna.
