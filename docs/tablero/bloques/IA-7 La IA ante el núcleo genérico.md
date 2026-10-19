---
tipo: bloque
id: IA-7
carril: ia
estado: backlog
prioridad: media
fase: MVP-3
rama:
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-19
cerrado:
---

## Qué y por qué

**Anotado desde ERP-2** (el núcleo genérico, decidido por el líder el 2026-10-19). El catálogo de «Mi institución» ya sirve a cualquier organización civil, no solo a un asilo o una casa hogar. Esto cambia lo que ve la IA. El ERP ya hizo su parte: el catálogo, las palabras de cada opción en `es-MX.ts`, el manual y un tercer ejemplo. **Lo que la IA hace con eso le toca a este carril.**

**Lo que cambió en el catálogo** (`core/institution/catalog.rs` y `forms.rs`):

| Campo | Opciones nuevas (código «palabras de la pantalla») |
|---|---|
| `institution.populations` | `general_public` «Población en general (sin un grupo de edad)», `organizations` «Otras organizaciones» |
| `institution.modalities` | `remote` «A distancia o en línea», `support` «Con apoyos: dinero, especie o becas» |
| `institution.care_areas` | `environment` «Medio ambiente», `culture` «Cultura y arte», `human_rights` «Derechos humanos», `community_development` «Desarrollo comunitario», `employment` «Empleo y capacitación» |

**Lo que cambió de sentido:**
- **Dos cifras de personas** (decisión del líder al probar ERP-2):
  - `institution.served_estimate`, «¿Cuántas personas atienden hoy?», solo con residencia o estancia de día (`catalog::LODGING`);
  - `institution.served_last_year` (**campo nuevo**, un detalle de `institution`), «¿A cuántas personas atendieron en el último año?», con las demás formas (`catalog::BY_PROGRAM`);
  - una institución mixta da las dos.

  **La ficha de la IA (`ai_sheet::render_people`) solo lee `served_estimate`:** para una A.C. comunitaria hoy dice que falta «a quiénes atiende» aunque la cifra del año esté guardada. Además no dice de qué periodo es la cifra.
- **`institution.capacity_total`** solo aplica con residencia o estancia de día. Su condición mira otra ventana. El Capturista describe la condición solo (`Solo aplica si institution.modalities es residential o day_care`).
- **`institution.sex_served`, `age_min`, `age_max`** solo aplican si se marcó un grupo de edad: con «Población en general» u «Otras organizaciones» solas, no.
- **«¿Qué piden para que alguien ingrese?»** ahora es «¿Qué piden para atender a alguien?».
- **El Expediente:** el Registro de Centros de Asistencia Social se pide solo si viven menores en la institución. La licencia de uso de suelo y el programa de protección civil, solo con residencia, estancia de día o consulta.

**Un arreglo que el ERP hizo en código de este carril, para no romperlo:** `capturist::still_to_tell` evaluaba las condiciones solo con los datos de su ventana. Con la capacidad mirando «Su institución», nunca la habría listado. Ahora le pasa también lo que dicen las otras ventanas (`profile::forms::context`). Es una línea: revísenla.

**Algo que ya pasaba y ahora se nota más:** `reader::fill::asked_fields` (y por él `upload`) le pregunta al Lector por los campos vacíos sin ver si aplican. Por ejemplo, el folio de la Junta a una A.C., y ahora la capacidad a quien no es residencia. Lo sugerido de un campo que no aplica nunca se ve ni se guarda, pero gasta en la llamada. La prueba `a_written_field_is_not_asked_and_a_full_window_asks_nothing` ahora dice que la institución es residencia para seguir probando lo mismo. Las condiciones con datos de otras ventanas están en `profile::forms::with_context` y `context`.

**Qué hay que revisar** (sin decir cómo):
- La ficha de la IA (`ai_sheet.rs`): el periodo de la cifra de personas, y las opciones nuevas en sus palabras.
- Los oficios del Asistente y del Capturista: ejemplos de asilo o casa hogar, y que entiendan las opciones nuevas.
- Las palabras del Lector para llenar los campos (`reader::fill::SEARCH`, `upload::SPEAKS_OF`): `served_estimate` busca «viven», «residentes» y «estancia de día». Para que la suite siga pasando, el ERP puso las primeras palabras de `served_last_year` y su línea en `lector.json` («nada», con 48 aceptable). Hay que medirlas.
- Las baterías (`fixtures/capturista.json`, `fixtures/expediente/asistente.json`, `lector.json`): hoy todas son de residencias. Hay un ejemplo nuevo que no lo es, `fixtures/institucion-red-comunitaria.json`.

## Terminado cuando

(Por definir.)

- [ ] Con «Red Comunitaria Ejemplo, A.C.», el Asistente y el Capturista hablan de ella sin suponer una residencia, y la cifra de personas se dice como lo que es (hoy o en el último año).
- [ ] Las baterías siguen pasando en seco.

## Plan

## Cómo probarlo

## Resultado
