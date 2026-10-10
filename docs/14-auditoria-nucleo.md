# 14 · Auditoría del núcleo: «Mi institución» y su expediente

**Fecha:** 2026-10-09. **Estado:** borrador para decidir; de aquí sale el ADR-033.

## 1. Qué debe hacer el núcleo

«Mi institución» ya no es solo el contexto de la IA para armar proyectos: es la **fuente única de lo que la institución es**, y de ahí se alimenta todo lo demás. Cumple su función cuando:

1. **Cada módulo encuentra ahí lo que necesita de la institución** (tipo, ubicación, régimen fiscal, capacidad) sin leer tablas ajenas ni pedírselo a la persona otra vez.
2. **La IA recibe una ficha completa y confiable:** cada dato con su origen, sin borradores falsos.
3. **El expediente legal está completo y vigente:** se sabe qué documentos hay, cuáles faltan y cuáles vencen.
4. **Los documentos alimentan los datos:** lo que dice el acta o la constancia fiscal llega al perfil con `origin = 'document'` y su página.
5. **Las reglas viven en Rust:** la pantalla solo muestra lo que el núcleo ya compuso.

Las dos instituciones base son I.A.P. (un asilo y una casa hogar). Todo lo que sigue está pensado primero para ellas, sin cerrar la puerta a A.C. y otras figuras.

## 2. Lo que hay hoy

| Pieza | Dónde | Qué guarda o hace |
|---|---|---|
| `institution` | migraciones 0001 y 0019 | nombre, tipo (asilo, casa hogar, otra), misión, RFC, teléfono, correo, representante, estado, municipio, año de fundación, figura jurídica, donataria (sí/en trámite/no), CLUNI (sí/en trámite/no), `onboarded_at` |
| `institution_profile` (versionado) | 0001, 0019 | capacidad total, notas, cifras rápidas; cuelgan las líneas anónimas `population_group` y `staff_group` |
| Primer inicio | `core/onboarding` | 6 pasos obligatorios; los catálogos (estados, figuras, registro) viven aquí |
| Ficha de la IA | `core/ai_sheet.rs` | identidad, ubicación, figura, donataria, CLUNI, dinero, personal, beneficiarios e instalaciones como agregados |
| Documentos | `core/archive`, `src/core/documents` | texto pegado (txt, md, csv) con escáner, fragmentos y FTS5; columna de donantes sin conectar |

## 3. Hallazgos

> **Avance:** D1, D2, D3 y D7 corregidos en N1 y D4 en N2 (2026-10-09; también las reglas de Inicio: ocupación, lugares vacíos y avance). La prueba de fronteras ahora revisa el SQL y encontró dos accesos más, también corregidos: el núcleo borraba `fac_site` al cargar el ejemplo y Proyectos leía `document` y `document_chunk` de su convocatoria.

### 3.1 Defectos y deuda de desacoplamiento (se arreglan primero)

| # | Hallazgo | Evidencia | Efecto |
|---|---|---|---|
| D1 | **Un cambio en Personal o Beneficiarios vuelve el perfil borrador.** `sync_profile` guarda con `profile_store::save`, que abre una versión nueva sin confirmar si la última estaba confirmada. | `core/profile/sync.rs:58`, `core/profile/storage.rs:139` | La ficha de la IA dice «BORRADOR» después de dar de alta a una persona; se crea una versión por cada cambio en los módulos; los proyectos siguen leyendo la última confirmada, que se va quedando vieja. |
| D2 | **Proyectos lee `institution_profile` con SQL directo.** La tabla es del núcleo; la prueba de fronteras no lo ve porque revisa `use`, no SQL. | `modules/projects/storage/projects.rs:69` y `:95` | Rompe la regla de ADR-032 («Proyectos usa solo `core::api`»). |
| D3 | **Los catálogos de la institución viven en el primer inicio.** `profile::domain` importa `STATES`, `LEGAL_FORMS`, `REGISTRY` y `OLDEST_YEAR` de `onboarding::domain`. | `core/profile/domain.rs:325-340` | El perfil depende del asistente; al agregar campos crece el enredo. |
| D4 | **Reglas de negocio en la pantalla.** «Las fichas mandan sobre las cifras rápidas» y las brechas de personas y espacios se deciden en TypeScript. | `src/core/profile/ProfilePage.tsx` (`peopleApprox`, `staffApprox`), `src/core/profile/gaps.ts` | Contradice el principio 8 (la ficha se compone en Rust) y duplica reglas que ya están en `onboarding::domain`. |
| D5 | **La nómina y las cuotas se calculan en el perfil.** `ProfileTotals` suma sueldos, prestaciones y cuotas a partir de las líneas anónimas. | `core/profile/domain.rs:186-295` | El núcleo hace cuentas que son de Personal y Beneficiarios. ADR-032 §8 lo dejó así a propósito; cambiarlo necesita su propio ADR. |
| D6 | **La identidad no tiene origen ni historial.** `institution` se sobrescribe en cada guardado, sin `origin`, `source_ref` ni confirmación; solo se versionan capacidad, notas y líneas anónimas. | migración 0001; `storage.rs:146` | Incumple el principio 4: no se sabe si el RFC lo escribió la persona o salió de un documento. |
| D7 | **Las líneas de beneficiarios se guardan como `origin = 'user'`** aunque las calcula el módulo (el personal sí usa `computed`). | `core/profile/storage.rs:205` | Origen falso. |
| D8 | **El ejemplo del asilo es A.C.,** pero las dos instituciones reales son I.A.P. | `fixtures/institucion-asilo.json` | No se prueba el caso real (JAP, patronato, obligaciones de I.A.P.). |

### 3.2 Datos que faltan

**Prioridad:** P1 = lo piden los módulos que siguen o casi todas las convocatorias; P2 = lo piden los documentos y Donantes; P3 = conveniente.
**IA:** si llega a la ficha de la IA. Lo que no llega se inserta por código al exportar, como el contacto hoy.

#### Identidad

| Dato | Prio | Lo usa | IA |
|---|---|---|---|
| Nombre legal completo («Asilo X, I.A.P.») aparte del nombre corto | P1 | documentos, recibos, Donantes | sí |
| Objeto social (lo que dicen los estatutos), aparte de la misión | P1 | convocatorias (elegibilidad), proyectos | sí |
| Visión y valores | P3 | proyectos | sí |
| Población que atiende por regla: sexo, edad mínima y máxima, criterios de ingreso | P1 | Beneficiarios (validar ingresos), convocatorias | sí |
| Servicios o programas que ofrece (residencia, estancia de día, comedor, terapia…) | P1 | proyectos, Beneficiarios (programas), Donantes | sí |
| Logotipo | P2 | documentos y guía en Word | no |

#### Ubicación y contacto

| Dato | Prio | Lo usa | IA |
|---|---|---|---|
| Domicilio completo (calle, número, colonia, código postal) | P1 | recibos deducibles, documentos, convocatorias | solo municipio y estado |
| Página web y redes sociales | P3 | Donantes, documentos | no |

El estado y el municipio ya existen y también los necesitan Personal y Finanzas: el impuesto sobre nómina cambia por estado y el salario mínimo es otro en la zona libre de la frontera norte.

#### Legal y fiscal

| Dato | Prio | Lo usa | IA |
|---|---|---|---|
| Régimen fiscal y código postal fiscal (de la constancia de situación fiscal) | P1 | Finanzas, recibos deducibles, Donantes | régimen sí |
| Constitución: fecha, número de escritura y notaría | P2 | convocatorias, documentos | fecha sí |
| Registro ante la Junta de Asistencia Privada: cuál Junta y folio (para I.A.P.) | P1 | Finanzas (obligaciones con la Junta), convocatorias | que está registrada sí; el folio no |
| Donataria: fecha y número del oficio, rubro autorizado | P1 | Donantes (recibos), convocatorias | rubro y vigencia sí |
| CLUNI: la clave (hoy solo sí/no) | P2 | convocatorias federales | que la tiene sí |
| Representante legal: además del nombre, vigencia del poder | P2 | convocatorias, documentos | no |
| Órgano de gobierno (patronato o consejo): cuántas personas y qué cargos | P2 | convocatorias (gobernanza), documentos | conteo sí |
| Registros del giro: Registro Nacional de Centros de Asistencia Social (casa hogar), aviso o licencia sanitaria (asilo), licencia de uso de suelo | P1 | convocatorias, cumplimiento | si tiene cada uno, sí |

#### Operación

| Dato | Prio | Lo usa | IA |
|---|---|---|---|
| Capacidad por grupo o sección (hoy solo un total) | P2 | Beneficiarios, Instalaciones (camas contra capacidad) | sí |
| Política de cuotas de recuperación (fija, por estudio socioeconómico, gratuita) | P2 | Beneficiarios, Finanzas | sí |
| Año fiscal y mes de cierre | P2 | Finanzas | no |

#### Lo que hay que confirmar antes de modelarlo

Varias obligaciones dependen del estado y de la Junta: el presupuesto anual ante la Junta, el informe de actividades, los estados financieros dictaminados, la cuota a la Junta, el informe de transparencia de donatarias y el informe anual del Registro Federal de OSC. Se confirman con las dos instituciones y con el texto oficial antes de escribir reglas, igual que las NOM de Instalaciones.

### 3.3 Documentos de la institución: hoy son un callejón sin salida

| Hoy | Problema |
|---|---|
| Solo texto pegado o archivos .txt, .md y .csv. La ventana acepta PDF y Word, pero los rechaza. | Los documentos base (acta, constancia, oficios) son PDF. La base ya lee PDF, Word y Excel para las convocatorias (`documents/`). |
| Todo se guarda como `kind = 'internal'` («Textos»). | No se sabe qué documento es cuál. Además, `document.kind` tiene un `CHECK` cerrado. |
| **Nadie los lee.** Ni la ficha de la IA ni Proyectos consultan los documentos de la institución; solo los de convocatorias. | Se guardan, se trocean e indexan, y no alimentan nada. |
| Sin vigencia ni fecha de emisión. | No se puede avisar que la opinión de cumplimiento o el poder están por vencer. |
| Sin relación con los datos. | El RFC del perfil y el de la constancia pueden no coincidir y nadie se entera. |
| La columna de donantes es un `TODO` en la pantalla. | Se resuelve con el módulo de Donantes, no ahora. |

## 4. Propuesta: el expediente de la institución

La idea central es que **el expediente sea una lista de lo que la institución debe tener, con estado, y el archivo sea opcional.** «Tenemos el acta, pero no la hemos subido» también es información útil.

### 4.1 Tabla propia del núcleo (en lugar de tocar `document`)

```
core_record_type      catálogo en código (no en tabla): acta_constitutiva, estatutos_reforma, poder_representante,
                      constancia_situacion_fiscal, oficio_donataria, cluni, opinion_cumplimiento_32d,
                      registro_junta, registro_cas, aviso_funcionamiento_sanitario, uso_de_suelo,
                      programa_proteccion_civil, aviso_privacidad, reglamento_interno, organigrama,
                      estados_financieros, informe_anual, presupuesto_junta, otro
core_record           id, type, status (have|missing|in_progress|not_applicable), document_id NULL,
                      issued_on, valid_until, reference (folio u oficio), notes, origin, source_ref,
                      confirmed_at, confirmed_by
```

- **Qué tipos le corresponden a cada institución** lo decide Rust según la figura jurídica, el tipo de institución y lo que ya se marcó (por ejemplo, el oficio solo si es donataria; el Registro de Centros de Asistencia Social solo para casa hogar). La pantalla muestra «Tiene 9 de 12» y lo que falta.
- **La vigencia** se calcula en Rust. Hay tipos que vencen por regla (la opinión de cumplimiento) y otros que tienen su fecha escrita (el poder, la protección civil).
- **`document` no se toca:** el `CHECK` de `kind` obligaría a reconstruir la tabla. El expediente apunta a un documento con `document_id`.

### 4.2 Del documento al dato (determinista primero)

1. Leer PDF y Word con lo que ya existe en `documents/`; los escaneados se avisan, como en las convocatorias.
2. **Reglas sin IA:** de la constancia sale el RFC, el régimen, el código postal y la fecha; del oficio, la fecha y el número; de la CLUNI, la clave. Son formatos oficiales y estables.
3. **IA solo para proponer** lo que no tiene formato fijo (el objeto social de los estatutos), con cita verificada por el código.
4. La persona confirma. El dato queda con `origin = 'document'` y su `source_ref` (`{document_id, page}`).
5. Si el dato del perfil y el del documento no coinciden, se avisa y no se decide solo.

### 4.3 Qué sale hacia los demás (`core::api`)

- **Proyectos:** qué documentos tiene vigentes, para cruzarlos con «documentos que pide la convocatoria» (el checklist pendiente de la Fase 3).
- **IA:** una línea por tipo («Opinión de cumplimiento: vigente hasta marzo de 2027»). Nunca el contenido ni los folios.
- **Inicio:** los vencimientos próximos, en la lista de pendientes.

## 5. Qué necesita cada módulo del núcleo

| Módulo | Ya recibe | Le falta del núcleo |
|---|---|---|
| Personal | tipo de institución | estado y municipio (impuesto sobre nómina, salario mínimo de la zona) |
| Beneficiarios | tipo | población por regla (sexo y edades), capacidad por sección, política de cuotas |
| Instalaciones | tipo | capacidad para comparar con camas (hoy cruza en el tablero) |
| Finanzas | nómina y cuotas sumadas | régimen fiscal, año fiscal, si es donataria, obligaciones con la Junta |
| Donantes (por crear) | — | nombre legal, RFC, domicilio fiscal, régimen, oficio de donataria, logotipo |
| Proyectos | ficha de la IA, nombre y cifras | expediente vigente; que deje de leer tablas del núcleo (D2) |

## 6. Orden de trabajo

El orden quedó en el ADR-033, junto con los bloques de la IA del ADR-034. Lo primero son los defectos (N1) y el catálogo de campos con el perfil de atención (F1), para que los datos nuevos nazcan universales.

## 7. Decisiones tomadas (2026-10-09)

1. **Patronato:** se guardan los nombres, sin llegar nunca a la IA (ADR-033 §3).
2. **Cuenta bancaria de la institución:** dato institucional protegido, distinto de la CLABE del personal (ADR-033 §3).
3. **Historial:** ultraligero, una línea por cambio y una foto mensual de indicadores (ADR-033 §4).
4. **Varias sedes:** se prepara el modelo y se construye después (ADR-033 §6).
5. **Obligaciones ante la Junta y el SAT:** para una actualización posterior.
6. **Nuevo:** formularios universales con un catálogo de campos y un perfil de atención en lugar del tipo fijo (ADR-033 §1 y §2), y la IA como equipo de agentes con herramientas, conocimiento recuperable y un asistente siempre a la mano (ADR-034).
