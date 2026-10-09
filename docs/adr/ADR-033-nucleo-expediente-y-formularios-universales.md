# ADR-033 · El núcleo como fuente de la institución: formularios universales, expediente, datos institucionales protegidos e historial ligero

**Estado:** aceptada (2026-10-09); se aplica por bloques junto con el ADR-034. El dibujador genérico se valida primero solo con «Mi institución». Sale de la auditoría `docs/14-auditoria-nucleo.md`. Va de la mano del ADR-034 (la IA en todo el ERP), que usa el catálogo de campos de este ADR.

## Contexto

La auditoría encontró que «Mi institución» guarda lo básico, pero no alcanza para alimentar al ERP. Faltan datos legales y fiscales, los documentos no alimentan nada, hay reglas en la pantalla y un defecto que vuelve borrador el perfil. Además, los formularios están escritos para tres casos fijos: asilo, casa hogar u «otra».

- **El tipo de institución decide los campos.** Cada módulo convierte `institution.kind` en su `Flavor` y muestra unos campos u otros: escolaridad o grado escolar, bandas de edad, puestos sugeridos.
- **Una institución mixta o distinta queda fuera** (discapacidad, mujeres, calle, adicciones, comedor comunitario). Agregar cada caso obligaría a tocar todos los formularios.
- **Los campos propios** (`hr_custom_field`, `care_custom_field`) ya son un formulario descrito por datos, pero cada módulo lo resolvió por su cuenta.

Decisiones de la institución (2026-10-09):
1. Los integrantes del patronato **se guardan con su nombre**, sin llegar nunca a la IA.
2. La cuenta bancaria de la institución **es dato institucional**, distinto de los datos bancarios del personal.
3. Se quiere un **historial ultraligero** para medir el progreso.
4. **Varias sedes:** se prepara el modelo, pero se construye después.
5. **Obligaciones ante la Junta, el SAT y otros:** se dejan para una actualización posterior.
6. **Todos los formularios deben ser universales** desde ahora.

## Decisión

### 1. Formularios universales: el catálogo de campos

**Cada formulario se describe como datos en Rust y la pantalla lo dibuja.** Es la misma fuente para la pantalla, la validación, el manual del asistente y las propuestas de la IA (ADR-034).

- **Tipos en la base** (`common/forms`): `FormSpec` (secciones y campos) y `FieldSpec`. Cada campo lleva:
  - `id` estable (`institution.legal_name`, `care.person.mobility`) y tipo (`text`, `long_text`, `number`, `money`, `year`, `date`, `select`, `multi_select`, `yes_no`, `email`, `phone`, `catalog`);
  - opciones (códigos), obligatorio o no, validación (reglas con nombre, en Rust);
  - **`applies_when`**: cuándo aparece, según el perfil de atención (abajo) o según otro campo;
  - **sensibilidad**: `public`, `internal`, `institutional_private` (nunca a la IA, la inserta el código al exportar) o `personal` (fichas de personas);
  - **`ai`**: si llega a la IA y cómo (`as_is`, `aggregate_only` o `never`);
  - **quién lo usa**: qué módulos y procesos lo consumen, para el «¿para qué sirve?».
- **Los textos no viven en Rust:** la etiqueta corta, en `es-MX.ts` con la clave del `id`. La explicación larga, en el manual del ERP (ADR-034). Una prueba exige que cada `id` tenga las dos.
- **Cada dueño describe sus formularios:** el núcleo los suyos y cada módulo los de sus fichas, en su `domain/`, como datos puros. La base da los tipos y el validador genérico. El núcleo junta los catálogos por el `api` de cada módulo para el asistente.
- **Los campos propios** de Personal y Beneficiarios pasan a ser `FieldSpec` con origen `custom`. Mismo dibujo, misma validación.
- **La pantalla tiene un solo dibujador** (`FormRenderer`, con los componentes de `src/components/ui/`). El acomodo fino por sección (columnas, agrupación) va en el `FormSpec`, para conservar el diseño de nivel empresarial sin escribir cada formulario a mano.
- **Se migra formulario por formulario.** Primero «Mi institución» (sus campos nuevos nacen en el catálogo); después Beneficiarios y Personal, al tocarlos. Nunca un cambio masivo.

### 2. El perfil de atención reemplaza al tipo fijo

La institución declara **a quién atiende y cómo**, en lugar de elegir entre tres tipos:

| Dato | Opciones (códigos) | Varias |
|---|---|---|
| Poblaciones | primera infancia, niñez, adolescencia, juventud, personas adultas, personas mayores | sí |
| Sexo que atiende | mujeres, hombres, ambos | no |
| Modalidades | residencial, estancia de día, ambulatoria o consulta, comunitaria, en domicilio | sí |
| Áreas de atención | cuidado, salud, discapacidad, educación, alimentación, violencia, adicciones, calle, migración, salud mental, otra | sí |

- **`applies_when` se escribe contra este perfil:** «grado escolar» aparece si atiende niñez o adolescencia; «continencia», si atiende personas mayores o discapacidad. Una institución mixta ve los dos grupos de campos.
- **`institution.kind` se queda** por compatibilidad: se deduce del perfil (solo personas mayores y residencial = `elderly_home`). Los `Flavor` de los módulos pasan a construirse desde el perfil de atención, no desde el tipo. Los datos actuales se trasladan en la migración: asilo = personas mayores, residencial, cuidado.
- **Las bandas de edad y los catálogos sugeridos** (puestos, espacios) salen de las poblaciones marcadas, no de un `match` de tres casos.

### 3. Datos institucionales protegidos

Nueva sensibilidad `institutional_private`: es de la institución, **nunca llega a la IA ni a las herramientas de los agentes** y la inserta el código al exportar. El escáner la reconoce como propia (como hoy el contacto y la representante).

- **Patronato u órgano de gobierno** (`core_board_member`): nombre, cargo (código y texto), desde cuándo, activo. Hacia la IA solo sale cuántas personas y qué cargos.
- **Cuentas bancarias de la institución** (`core_bank_account`): banco, CLABE (validada con su dígito verificador, como en ADR-027), titular = la institución, para qué se usa (donativos, operación) y si se muestra en documentos. Se ven tapadas, y mostrarlas queda en la bitácora con el campo y nunca el valor.
- **Diferencia con el personal:** la CLABE de una persona (`hr_person.clabe`) es dato personal del módulo de Personal y sigue sus reglas. El escáner no deja de marcar una CLABE en un texto libre **salvo que sea exactamente una cuenta de la institución**.

### 4. Historial ultraligero para medir el progreso

Dos tablas pequeñas y solo de agregar:

- **`core_change`:** cada cambio de un dato de la institución con sensibilidad `public` o `internal`. Guarda el campo, el valor nuevo, el `origin`, cuándo y quién. Basta para reconstruir cómo estaba antes. Los `institutional_private` registran **solo que cambiaron**, sin el valor.
- **`core_snapshot`:** una foto por mes (`YYYY-MM`) de los indicadores que ya calculan los módulos (personas atendidas, lista de espera, personal, espacios por estado, balance en palabras, porcentaje de llenado, expediente vigente). Son cifras agregadas: con eso se arman las gráficas de progreso sin guardar fichas viejas. Se toma al abrir la app si el mes no tiene foto.
- **Resuelve el D6 de la auditoría:** los datos de `institution` pasan a llevar `origin`, `source_ref` y confirmación por campo. La versión del perfil deja de ser la única historia.

### 5. El expediente

Como en `docs/14-auditoria-nucleo.md` §4: `core_record` con un catálogo de tipos en código, estado (`have`, `missing`, `in_progress`, `not_applicable`), documento opcional, fechas de emisión y vigencia, folio, origen y confirmación.

- **Qué tipos le tocan a cada institución** lo decide Rust según la figura jurídica y el perfil de atención.
- **Del documento al dato:** el Lector propone (ADR-034) y la persona confirma.
- **Las reglas de obligaciones y vencimientos por Junta o por el SAT quedan para después** (decisión 5). Por ahora, solo la vigencia escrita en cada documento.

### 6. Preparado para varias sedes, sin construirlo

- Las tablas nuevas que dependen de un inmueble llevan `site_id` opcional desde el inicio. Por ejemplo, `core_record` para la licencia sanitaria, el uso de suelo o la protección civil.
- Ningún código nuevo supone «un solo inmueble». Lo que hoy lo supone (la pantalla de Instalaciones, `main_site` del primer inicio) se marca y se resuelve en su propio ADR.

## Orden de trabajo (junto con el ADR-034)

| Bloque | Qué |
|---|---|
| **N1** | Defectos de la auditoría (D1 borrador, D2 SQL de Proyectos, D3 catálogos, D7 origen) |
| **F1** | `common/forms` (`FormSpec`, `FieldSpec`, validador) y el perfil de atención con su migración; los `Flavor` salen del perfil |
| **N2** | La pantalla ya no decide: `institution_overview` en Rust |
| **IA1–IA2** | Bucle de agentes, `ai_proposal`, manual y «?» por campo (ADR-034) |
| **N3** | Formularios de «Mi institución» en el catálogo, con los datos nuevos (P1 de la auditoría), patronato, cuentas, origen por campo, `core_change` y `core_snapshot`; ejemplo I.A.P. |
| **IA3–IA4** | Asistente en toda la app y Capturista |
| **N4** | Expediente (`core_record`) y subir PDF y Word |
| **IA5–IA6** | Embeddings guardados, búsqueda híbrida y Lector de documentos |
| **N5** | El expediente hacia Proyectos, la IA e Inicio |
| **Después** | Formularios de Beneficiarios y Personal en el catálogo; varias sedes; obligaciones y vencimientos por regla |

## Consecuencias

- **Agregar un tipo de institución deja de tocar formularios:** se marcan sus poblaciones y áreas, y los campos aparecen solos.
- **El dibujador genérico es una apuesta de diseño:** se valida contra `docs/13-sistema-visual.md` con «Mi institución» antes de pasar los módulos.
- **Migraciones nuevas:**
  - perfil de atención;
  - `core_board_member` y `core_bank_account`;
  - `core_change` y `core_snapshot`;
  - origen por campo de `institution`;
  - `core_record`;
  - `ai_proposal` (ADR-034).

  Ninguna edita una aplicada.
- **El escáner** aprende a reconocer las cuentas y los nombres del patronato como propios de la institución, con pruebas positivas y negativas (principio de pruebas obligatorias).

## Alternativas descartadas

- **Seguir con un `match` por tipo de institución:** cada tipo nuevo tocaría todos los formularios y los catálogos.
- **Formularios escritos por la institución desde cero (constructor libre):** perdería las reglas, los agregados y lo que llega a la IA. Los campos propios se quedan para lo que no está en el catálogo.
- **Versionar copias completas del perfil para el historial:** pesado y con datos que no hacen falta. Una línea por cambio y una foto mensual de indicadores bastan.
- **Guardar el patronato y las cuentas sin protección o no guardarlos:** la institución los necesita para sus documentos. El nivel `institutional_private` los guarda sin exponerlos.
