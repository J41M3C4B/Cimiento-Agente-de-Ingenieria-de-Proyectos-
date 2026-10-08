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

-- La tabla `facility` (lista de espacios del perfil) se eliminó en la migración 0018: las instalaciones viven en su
-- módulo (ADR-030, tablas fac_*; ver más abajo).

-- Ingresos escritos a mano (ADR-026). Las cuotas de los beneficiarios del padrón NO se guardan aquí: se calculan.
CREATE TABLE income_source (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  label TEXT NOT NULL,             -- "Padrinos", "Fundación X 2025", "Eventos"
  amount_mxn INTEGER,              -- como lo escribió la persona, según `period` (antes annual_amount_mxn)
  period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual')),
  kind TEXT NOT NULL DEFAULT 'other'
    CHECK (kind IN ('fee_estimate','recurring_donor','occasional_donation','project_grant','other')),
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

-- Módulo de Personal (ADR-027, migración 0015). Tablas propias, aparte del perfil; solo salen agregados (hr::api).
-- hr_modality      modalidades propias de la institución: code, title, behaves_as (una modalidad del código)
-- hr_position      puestos: title (único), area, duties, default_modality, default_schedule, reference_pay_mxn,
--                  authorized_seats, reports_to, active
-- hr_person        datos personales, domicilio, contacto, banco (clabe), fiscales (curp, rfc, nss, tax_regime, tax_zip),
--                  extra (datos propios en JSON), account_id (para los perfiles de acceso)
-- hr_job           trabajo de la persona: position_id, modality, start_date (+ start_date_approx), end_date, schedule,
--                  shift, work_days, weekly_hours, status (active|vacation|sick_leave|leave|left), left_date, left_reason,
--                  pay_amount_mxn, pay_period (weekly|biweekly|monthly), pay_method; current = 1 para el vigente
-- hr_emergency_contact  hasta 2 por persona
-- hr_custom_field  datos propios del formulario
-- staff_group.relation  tipo de relación de cada línea anónima (dónde cuenta su dinero)

-- Módulo de Beneficiarios (ADR-029, migración 0017; elimina roster_entry y roster_field).
-- care_person         identificación (birth_date + birth_date_approx, sex, curp, origen, lengua, estudios o escuela,
--                     group_id), ingreso (entry_date + approx, stay_mode, referred_by, admission_reasons JSON, status
--                     active|hospitalized|discharged|deceased, status_date, discharge_reason), salud por categorías
--                     (dependency, mobility, disabilities JSON, chronic_conditions JSON, continence, orientation,
--                     psych_care, vaccines_up_to_date), familia (visits, legal_status), aportación (monthly_fee_mxn,
--                     fee_payer, programs JSON), consentimiento (consent_date, consent_signer), extra, hidden
-- care_contact        hasta 2 responsables por persona (legal_guardian)
-- care_group          grupos propios (título único)
-- care_custom_field   datos propios del formulario (hidden mientras su borrado espera)
-- care_waitlist       solicitudes de ingreso: requested_on, name/phone opcionales, sex, approx_age, dependency, reason,
--                     status waiting|admitted|declined|withdrawn, person_id

-- Primer inicio (ADR-031, migración 0019).
-- institution  state (código de 32 estados), municipality, founded_year, legal_form (ac|iap|ibp|sc|abp|religious|other),
--              authorized_donee y cluni (yes|in_progress|no), onboarded_at (una sola vez, al terminar el asistente)
-- institution_profile  served_estimate, staff_paid_estimate, staff_volunteer_estimate: cifras rápidas mientras no hay
--              fichas en los módulos (las fichas mandan)
-- app_user.welcomed_at  la persona ya vio la bienvenida

-- Módulo de Instalaciones (ADR-030, migración 0018; elimina facility).
-- fac_site       un inmueble (la pantalla maneja uno): name, land_m2, built_m2, floors, floor_access JSON
--                (ramp|elevator|stair_lift|none), built_year, tenure (own|loan|rent|borrowed|other), tenure_until,
--                tenure_documented; servicios (water_sources JSON, water_shortage never|sometimes|often,
--                water_storage_liters, power_outages, gas, drainage, internet); seguridad (extinguishers,
--                extinguishers_current, smoke_detectors, marked_exits, emergency_lights, first_aid_kit,
--                internal_program yes|in_progress|no, civil_protection_opinion, opinion_year, drills_per_year), notes
-- fac_space      un grupo de espacios de un tipo en un piso: kind (catálogo), label, floor (-1 sótano, 0 planta baja…),
--                count, good, fair, poor, unusable (suman count o menos: el resto está «sin revisar»), problems JSON,
--                accessible, beds, hospital_beds (dormitorio y enfermería), grab_bars, accessible_shower (baño), notes,
--                origin, source_ref
-- fac_equipment  un grupo de equipo: kind (catálogo), label, count, good, fair, poor, unusable, notes, origin, source_ref

-- Perfiles de acceso (ADR-028, migración 0016).
-- app_user        cuentas: username (único, minúsculas), display_name, person_id (ficha del personal), role (admin|manager),
--                 active, password_hash (Argon2id), must_change_password, failed_attempts, locked_until (unix), last_login_at
-- access_request  borrados que esperan al administrador: kind, target_id, target_label, requested_by, status
--                 (pending|approved|rejected), resolved_by, resolved_at; una sola pendiente por cosa
-- audit_log.actor_id  quién hizo cada cosa (de la sesión)
-- hidden          en document, project, roster_entry, roster_field, hr_person, hr_custom_field: oculto mientras espera
-- app_settings 'access.recovery'  hash del código de recuperación del administrador

-- Egresos por concepto (ADR-026). La nómina no se escribe aquí: se calcula del padrón con prestaciones.
-- `institution_profile.annual_budget_mxn` es el «gasto anual aproximado» (modo exprés): cuenta mientras esta
-- lista esté vacía.
CREATE TABLE expense_item (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  label TEXT NOT NULL,             -- "Alimentos", "Luz y agua", "Medicinas"
  amount_mxn INTEGER CHECK (amount_mxn IS NULL OR amount_mxn >= 0),
  period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual')),
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

## Dueño de cada tabla (ADR-032)

Cada tabla tiene un solo dueño. Solo su dueño la lee y la escribe; los demás le piden datos por su `api`.

| Dueño | Tablas |
|---|---|
| Base | `schema_migrations`, `app_settings`, `audit_log`, `ai_usage` |
| Núcleo | `institution`, `institution_profile`, `population_group`, `staff_group` (líneas anónimas que se calculan de los módulos), `document`, `document_chunk`, `app_user`, `access_request`; `roster_field` y `roster_entry` (padrón viejo del ADR-020, vacío tras los traslados) |
| Personal (`hr_*`) | `hr_modality`, `hr_position`, `hr_person`, `hr_job`, `hr_emergency_contact`, `hr_custom_field` |
| Beneficiarios (`care_*`) | `care_group`, `care_person`, `care_contact`, `care_custom_field`, `care_waitlist` |
| Instalaciones (`fac_*`) | `fac_site`, `fac_space`, `fac_equipment` |
| Finanzas (`fin_*`) | `fin_income`, `fin_expense`, `fin_settings` |
| Proyectos | `project`, `call_reading`, `call_reading_file`, `conversation_turn`, `conversation_root`, `diagnosis_summary`, `need`, `project_section`, `budget_item`, `schedule_activity`, `drafting_plan`, y las de la primera especificación: `grant_call`, `call_template`, `requirement`, `diagnosis_answer`, `diagnosis_pending`, `diagnosis_dimension`, `questionnaire`, `questionnaire_field` (conservan su nombre) |

- **0020 `fin_income`, `fin_expense`, `fin_settings`:** el dinero sale de las versiones del perfil y pasa al módulo de Finanzas (ADR-032).
  - La migración copia las líneas de la versión vigente del perfil y su «gasto anual aproximado» (`fin_settings.annual_budget_mxn`, una sola fila).
  - Cada línea conserva su `origin`, su `source_ref` y su confirmación.
  - `income_source`, `expense_item` e `institution_profile.annual_budget_mxn` quedan como historia de las versiones anteriores y ya no se escriben.
