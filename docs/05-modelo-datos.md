# 05 · Modelo de datos

SQLite cifrada (SQLCipher). Identificadores tipo texto con prefijo (`inst_`, `proj_`, `doc_`…) generados con ULID para que sean ordenables y migrables. Fechas en ISO 8601 UTC.

## Convención de origen (obligatoria)

Toda tabla que guarde un dato que pudo venir de la IA lleva:

| Columna | Tipo | Valores |
|---|---|---|
| `origin` | TEXT | `user`, `document`, `ai_assumption`, `computed` |
| `source_ref` | TEXT NULL | JSON: `{"document_id": "...", "page": 4}` o `{"ai_call_id": "..."}` |
| `confirmed_at` | TEXT NULL | Fecha en que una persona lo confirmó |
| `confirmed_by` | TEXT NULL | Rol que confirmó |

Regla: nada con `origin = 'ai_assumption'` y `confirmed_at IS NULL` puede exportarse.

## Esquema inicial (migración 0001)

```sql
-- Configuración y metadatos
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Institución (una por instalación en v1, pero modelado para varias)
CREATE TABLE institution (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('elderly_home','children_home','other')),
  mission TEXT,
  legal_rfc TEXT,                 -- RFC de la institución (persona moral)
  contact_phone TEXT,             -- contacto institucional, nunca a la IA
  contact_email TEXT,             -- contacto institucional, nunca a la IA
  legal_rep_name TEXT,            -- solo si la convocatoria lo exige; nunca a la IA
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Perfil versionado: cada confirmación crea una versión
CREATE TABLE institution_profile (
  id TEXT PRIMARY KEY,
  institution_id TEXT NOT NULL REFERENCES institution(id),
  version INTEGER NOT NULL,
  capacity_total INTEGER,
  annual_budget_mxn INTEGER,       -- centavos NO; pesos enteros
  notes TEXT,
  confirmed_at TEXT,
  created_at TEXT NOT NULL,
  UNIQUE (institution_id, version)
);

-- Población atendida, SOLO agregada
CREATE TABLE population_group (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  label TEXT NOT NULL,             -- "Adultos mayores con dependencia alta"
  age_min INTEGER, age_max INTEGER,
  count INTEGER NOT NULL CHECK (count >= 0),
  dependency_level TEXT CHECK (dependency_level IN ('low','medium','high','total')),
  notes TEXT,
  paying_count INTEGER,            -- migración 0008: cuántos del grupo pagan cuota de estancia
  monthly_fee_mxn INTEGER,         -- migración 0008: cuota mensual por persona que paga
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE staff_group (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  role TEXT NOT NULL,              -- "Enfermería", "Cocina", "Religiosas", "Voluntariado"
  count INTEGER NOT NULL CHECK (count >= 0),
  shift TEXT,                      -- "matutino", "24h", etc.
  paid INTEGER NOT NULL DEFAULT 1, -- 0 = voluntario
  -- migración 0008: una línea por puesto, sin nombre (la app escribe count = 1; el sueldo es por persona)
  monthly_salary_mxn INTEGER, contract TEXT CHECK (contract IN ('permanent','temporary','fees')),
  start_year INTEGER, notes TEXT,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE facility (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,              -- "dormitorio", "baño", "cocina", "enfermería"
  count INTEGER NOT NULL DEFAULT 1,
  condition TEXT CHECK (condition IN ('good','fair','poor','critical')),
  accessible INTEGER,              -- 0/1/NULL
  notes TEXT,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE income_source (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  label TEXT NOT NULL,             -- "Cuotas", "Donativos JAP 2025", "Eventos"
  annual_amount_mxn INTEGER,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

-- Documentos (solo versión limpia)
CREATE TABLE document (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('call','questionnaire','template','internal','quote','photo','other')),
  display_name TEXT NOT NULL,
  mime TEXT NOT NULL,
  data_level TEXT NOT NULL CHECK (data_level IN ('green','yellow')),  -- 'red' no existe
  clean_hash TEXT NOT NULL,        -- hash de la versión limpia
  blob BLOB,                       -- archivo limpio (si se necesita conservar, p. ej. plantillas)
  extracted_text TEXT,             -- texto extraído y tapado
  redactions_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE TABLE document_chunk (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES document(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  page INTEGER,
  text TEXT NOT NULL
);

CREATE VIRTUAL TABLE document_chunk_fts USING fts5(text, content='document_chunk', content_rowid='rowid');
-- Vectores (sqlite-vec). Dimensión según el modelo de embeddings elegido en Fase 3.
-- CREATE VIRTUAL TABLE document_chunk_vec USING vec0(embedding float[384]);

-- Convocatorias
CREATE TABLE grant_call (
  id TEXT PRIMARY KEY,
  funder TEXT NOT NULL,            -- "JAP", "Nacional Monte de Piedad"
  title TEXT NOT NULL,
  year INTEGER,
  opens_at TEXT, closes_at TEXT,
  call_template_id TEXT REFERENCES call_template(id),
  created_at TEXT NOT NULL
);

-- Plantilla reutilizable por convocatoria (se procesa una sola vez)
CREATE TABLE call_template (
  id TEXT PRIMARY KEY,
  funder TEXT NOT NULL,
  name TEXT NOT NULL,
  sections_json TEXT NOT NULL,     -- [{key, title, max_chars, required, guidance}]
  word_template_document_id TEXT REFERENCES document(id),
  version INTEGER NOT NULL DEFAULT 1,
  confirmed_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE requirement (
  id TEXT PRIMARY KEY,
  call_template_id TEXT NOT NULL REFERENCES call_template(id) ON DELETE CASCADE,
  code TEXT NOT NULL,              -- "REQ-03"
  text TEXT NOT NULL,
  kind TEXT NOT NULL,              -- max_amount, min_amount, allowed_category, required_document, deadline,
                                   -- max_duration_months, min_beneficiaries, cofunding_percent,
                                   -- section_required, max_length_chars, manual
  value_json TEXT,
  blocking INTEGER NOT NULL DEFAULT 0,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

-- Proyectos
CREATE TABLE project (
  id TEXT PRIMARY KEY,
  institution_id TEXT NOT NULL REFERENCES institution(id),
  profile_id TEXT NOT NULL REFERENCES institution_profile(id),
  grant_call_id TEXT REFERENCES grant_call(id),
  title TEXT NOT NULL,
  stage TEXT NOT NULL CHECK (stage IN ('PROFILE','DIAGNOSIS','PRIORITIZATION','CALL_SELECTION','DRAFTING','REVIEW','READY')),
  requested_amount_mxn INTEGER,
  archived_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE diagnosis_answer (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  dimension INTEGER NOT NULL CHECK (dimension BETWEEN 1 AND 7),
  question TEXT NOT NULL,
  answer TEXT NOT NULL,
  followup_index INTEGER NOT NULL DEFAULT 0,  -- 0 = pregunta base, 1-2 = repreguntas
  created_at TEXT NOT NULL,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE diagnosis_summary (
  project_id TEXT PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
  summary_json TEXT NOT NULL,      -- salida estructurada del diagnóstico (ver 02-flujo-funcional.md)
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE need (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  description TEXT,
  scores_json TEXT,                -- {"beneficiaries":4,"severity":5,...}
  total_score REAL,                -- calculado por código
  selected INTEGER NOT NULL DEFAULT 0,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE project_section (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  section_key TEXT NOT NULL,
  content TEXT NOT NULL,
  needs_review INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL,
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT,
  UNIQUE (project_id, section_key)
);

CREATE TABLE budget_item (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  category TEXT NOT NULL,          -- "material", "mano de obra", "equipo", "capacitación"
  description TEXT NOT NULL,
  quantity REAL NOT NULL CHECK (quantity > 0),
  unit TEXT,
  unit_price_mxn REAL NOT NULL CHECK (unit_price_mxn >= 0),
  vat_included INTEGER NOT NULL DEFAULT 1,
  funded_by TEXT NOT NULL CHECK (funded_by IN ('requested','institution','other')),
  quote_document_id TEXT REFERENCES document(id),
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE schedule_activity (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  start_month INTEGER NOT NULL CHECK (start_month >= 1),
  end_month INTEGER NOT NULL,
  CHECK (end_month >= start_month)
);

-- Cuestionarios Excel
CREATE TABLE questionnaire (
  id TEXT PRIMARY KEY,
  project_id TEXT REFERENCES project(id) ON DELETE CASCADE,
  call_template_id TEXT REFERENCES call_template(id),
  source_document_id TEXT NOT NULL REFERENCES document(id),
  created_at TEXT NOT NULL
);

CREATE TABLE questionnaire_field (
  id TEXT PRIMARY KEY,
  questionnaire_id TEXT NOT NULL REFERENCES questionnaire(id) ON DELETE CASCADE,
  sheet TEXT NOT NULL,
  cell TEXT NOT NULL,              -- "C14"
  question TEXT NOT NULL,
  field_type TEXT NOT NULL CHECK (field_type IN ('text','number','currency','date','list','boolean','formula')),
  allowed_values_json TEXT,        -- para listas desplegables
  required INTEGER NOT NULL DEFAULT 0,
  profile_mapping TEXT,            -- ruta a un dato del perfil, p. ej. "population.total" (se llena sin IA)
  answer TEXT,
  origin TEXT, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

-- Bitácora y uso de IA
CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  at TEXT NOT NULL,
  event TEXT NOT NULL,
  entity TEXT,
  entity_id TEXT,
  details_json TEXT                -- conteos y decisiones; NUNCA contenido
);

CREATE TABLE ai_usage (
  id TEXT PRIMARY KEY,
  at TEXT NOT NULL,
  task TEXT NOT NULL,              -- "diagnosis.next_question", "drafting.section", ...
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  cached_tokens INTEGER NOT NULL DEFAULT 0,
  estimated_cost_mxn REAL,
  project_id TEXT,
  success INTEGER NOT NULL
);
```

## Notas

- Montos en pesos (`INTEGER` para totales, `REAL` solo en precio unitario). Los cálculos usan aritmética decimal en Rust (`rust_decimal`) y se redondean a 2 decimales al mostrar.
- `population_group` y `staff_group` no tienen ni pueden tener columnas de identificación individual: desde la migración 0009 se **calculan** a partir del padrón (`roster_entry`, ADR-020), que es una tabla aparte que nunca llega a la IA. Cualquier PR que ponga nombres o contacto en ellas se rechaza.
- `blob` en `document` solo se usa cuando el archivo se necesita después (plantillas Word, cuestionarios Excel). Los PDF de convocatoria se guardan como texto extraído.

## Cambios posteriores a esta especificación (migraciones 0004 a 0007)

Esta especificación se escribió antes de varias decisiones; el código vive en `src-tauri/migrations/`:

- **0004 `call_reading`, `call_reading_file`:** la lectura de una convocatoria (ADR-015); reemplaza a `grant_call` y `call_template` en la práctica.
- **0005:** `project.kind` (`call` o `internal`), `project.call_reading_id` (el proyecto nace de su convocatoria, ADR-016), `call_reading.funder/year` (lo que escribió la persona) y `call_reading_file.role`.
- **0006:** `conversation_turn` y `conversation_root` (la conversación del diagnóstico, ADR-017) y `call_reading.confirmed_at`. Las tablas `diagnosis_answer`, `diagnosis_dimension` y `diagnosis_pending` ya no se escriben.
- **0007:** `project.asks_for_proposal` y `budget_item.administrative` (ADR-018).
- Las etapas de `project.stage` ahora van en el orden `PROFILE → CALL_SELECTION → DIAGNOSIS → PRIORITIZATION → DRAFTING → REVIEW → READY`.
