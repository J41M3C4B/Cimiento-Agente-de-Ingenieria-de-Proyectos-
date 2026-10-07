# ADR-027 · Módulo de Personal: la base de un futuro módulo de RH

**Estado:** aceptada (2026-10-07). Reemplaza la parte de personal del ADR-020; los beneficiarios siguen en el padrón del ADR-020.

## Contexto

El personal se llevaba en el padrón genérico del ADR-020: una ficha en JSON compartida con beneficiarios, con nombre, cargo, contrato, horario, sueldo, año de ingreso y contacto. Eso alcanzaba para sumar la nómina, pero no para lo que la institución pidió:

- Un formulario por pasos que recoja de una sola vez los datos personales, los del trabajo y el puesto, el contacto de emergencia, y el pago y los datos fiscales.
- Modalidades de empleo bien definidas, y poder crear puestos que no estén en la lista.
- Una base sólida para que la IA reciba solo agregados.
- Una estructura que el personal y las religiosas vayan aprendiendo a usar.
- Que todo esto sea el **inicio de un módulo de RH independiente** que después crecerá (historial de puestos, incidencias, nómina) y se podrá separar del resto de la app.

## Decisión

1. **Un módulo aparte**:
   - En Rust vive en `src-tauri/src/hr/`, con su dominio, guardado, servicio, frontera (`api.rs`) y traslado del padrón viejo.
   - Tiene tablas propias con prefijo `hr_` (migración 0015) y sus propios errores (`HrError`).
   - Del resto de la app solo usa la bitácora. Una prueba (`the_module_does_not_reach_into_the_rest_of_the_app`) falla si `hr/` usa cualquier otra cosa de la app.
   - La app le habla por `hr::api`, que solo entrega agregados anónimos (`staff_lines`, `ai_summary`), y por `hr::service`, que da servicio a las pantallas.
   - La coordinación con el perfil y el escáner vive en la app (`staff_service.rs`). El frontend vive en `src/features/hr/`, con sus propios tipos y llamadas.
   - Separarlo después sería mover `hr/` a un crate y `features/hr/` a su propia ruta.
2. **Persona y trabajo separados**:
   - `hr_person` guarda los datos personales, de contacto, fiscales y bancarios.
   - `hr_job` guarda el puesto, la modalidad, las fechas, el horario, la situación y el pago. Hoy hay un trabajo vigente por persona; la tabla ya admite historial.
   - `hr_emergency_contact` guarda hasta 2 contactos por persona.
   - `hr_custom_field` guarda los datos propios de la institución, que se conservan del padrón viejo.
   - `hr_person.account_id` queda listo para enlazar a la persona con su cuenta cuando existan los perfiles de acceso (siguiente bloque).
3. **Modalidades con reglas fijas en código** (`hr/domain/catalog.rs`):
   - Por tiempo indeterminado, por tiempo determinado, por obra determinada y periodo de prueba: relación laboral con aguinaldo, prima vacacional e IMSS.
   - Honorarios y asimilados a salarios: entran en la nómina, sin prestaciones.
   - Religiosa o religioso de la congregación y servicio social o prácticas: sin relación laboral. Su aportación o beca, si la hay, cuenta como egreso aparte («Aportaciones y apoyos»).
   - Voluntariado: sin pago.
   - Personal de una empresa externa (REPSE): cuenta como egreso («Personal de empresas externas»), no como nómina.
   - La institución puede crear **modalidades propias**, y cada una «se comporta como» una de estas, así que los cálculos siguen siendo deterministas.
4. **Catálogo de puestos**: el puesto es la plaza, no la persona.
   - **Campos**: nombre, área, funciones, modalidad y jornada habituales, pago de referencia, **plazas autorizadas** y a quién le reporta.
   - **Plazas sin cubrir**: las plazas que faltan se calculan y llegan a la IA como necesidad.
   - **Puestos sugeridos**: el catálogo arranca con los de un asilo o una casa hogar, y desde el formulario de la persona se pueden crear puestos nuevos.
   - **Archivar**: un puesto con personas no se puede archivar.
   - **Escáner**: el nombre y las funciones pasan por el escáner, porque pueden llegar a la IA.
5. **Formulario en cuatro pasos**:
   - Para guardar basta con el nombre, el puesto y la modalidad. Rust calcula el avance de cada paso según lo que aplica a la modalidad: el voluntariado no tiene paso de pago, y honorarios no pide el NSS.
   - **Validación en Rust**: CURP, RFC, NSS y CLABE se revisan con su forma y su dígito verificador. La CURP se compara con la fecha de nacimiento y el RFC con la CURP (si no coinciden es un aviso, no un bloqueo). También se revisan las fechas reales, el teléfono, el correo, el código postal, la cédula y las horas a la semana (aviso arriba de 48, art. 61 de la LFT).
   - **Pago**: se escribe como lo pagan (por semana, quincena o mes) y Rust lo pasa a mes con una regla fija. Un día de pago es la semana entre 7, la quincena entre 15 o el mes entre 30.
   - **El banco** se deduce de la CLABE.
6. **Identificadores tapados**:
   - CURP, RFC, NSS y CLABE nunca salen del servicio en claro. La ficha los trae como `null` y en su forma tapada (`HEGG••••••••••••04`).
   - Al guardar, `null` deja lo guardado, `""` lo borra y un valor lo reemplaza.
   - «Mostrar» (`hr_person_reveal`) devuelve el dato y deja en la bitácora `hr.sensitive_viewed` con el campo y quién lo consultó, nunca el valor.
7. **Baja y borrado son distintos**:
   - **Baja**: marcar «Ya no trabaja aquí» conserva la ficha (la nómina se guarda años) y la saca de los totales.
   - **Borrado**: «Borrar a esta persona» elimina todo de esa persona y lo registra (`hr.person_deleted`). Sirve para cumplir cuando alguien pide que se borren sus datos.
8. **Lo que sale del módulo**:
   - **Al perfil**: una línea anónima por puesto con los datos de pago (`staff_group`, ahora con su `relation`), con la que se calculan la nómina con prestaciones, las aportaciones y el personal externo.
   - **A la IA** (`ai_summary`): por puesto, cuántas personas, área, funciones, tipo de relación, jornada y turno, plazas autorizadas y sin cubrir. Además, el total por tipo de relación y, **solo para grupos de 3 o más personas**, rangos de escolaridad y de antigüedad.
   - **Nunca**: nombres, identificadores, domicilio, fechas de una persona, contactos ni pagos.
   - La sección de personal de la ficha de la IA se arma con lo que dice el módulo en ese momento, no con la versión guardada del perfil.
9. **Las reglas de pago de la ley** (`vacation_days`, `annual_benefits`) se mueven de `finances.rs` a `hr/domain/payroll.rs`. El perfil las usa a través de `hr::api`.
10. **Traslado** (migración 0015 + `storage::migrations::move_roster_staff`):
    - Cada ficha de personal del padrón viejo se convierte en persona con trabajo.
    - El cargo pasa a ser un puesto, el contrato una modalidad, y el año de ingreso el 1 de enero de ese año, marcado como aproximado.
    - El nombre se separa en nombres y apellidos a la manera mexicana.
    - Los datos propios conservan su clave, y un horario que el catálogo no conoce se guarda como dato propio.
    - Todo ocurre en la misma transacción y se registra `hr.imported` con el conteo. El padrón queda solo para beneficiarios, y el servicio del padrón rechaza la entidad `staff`.

## Consecuencias

- **Base de datos**: llega a la versión 15, con un paso de datos en código después del SQL. El ejecutor de migraciones aprendió a correr código al terminar una versión (`after`).
- **Cambios en el JSON**:
  - Los comandos del padrón ya no sirven para el personal; se usan los nuevos `hr_*`.
  - `ProfileTotals` suma `staff_support_annual_mxn` y `external_staff_annual_mxn`.
  - Las finanzas tienen dos líneas calculadas nuevas: `staff_support` y `external_staff`.
- **Frontend**: hay un frontend funcional con los componentes del catálogo (lista, formulario por pasos, catálogo de puestos, modalidad propia). El diseño fino queda para la sesión de diseño.
- **Privacidad**: guardar CURP, RFC, NSS y CLABE sube el nivel de cuidado. Siguen solo en la base cifrada de este equipo y en el respaldo cifrado. Antes de usar datos reales hace falta la revisión legal pendiente (`03-gobernanza-datos.md`). Los datos de salud quedaron fuera a propósito.
- **Pendiente para el módulo de RH**:
  - Historial de puestos (varias filas en `hr_job`).
  - Incidencias (vacaciones tomadas, faltas, incapacidades con fechas).
  - Expediente de documentos por persona.
  - Exportar la plantilla a Excel.
  - Cuotas del IMSS e INFONAVIT del patrón.
  - Roles: quién puede ver y cambiar qué (siguiente bloque).

## Alternativas descartadas

- **Seguir con el padrón genérico en JSON**: no permite validar identificadores, separar persona y trabajo ni sacar el módulo después.
- **Validar CURP y RFC solo por forma**: un error de dedo pasaría. El dígito verificador es una regla pública y barata.
- **Mandar a la IA escolaridad y antigüedad de cualquier tamaño de grupo**: «1 enfermera con posgrado» señala a una persona.
