# ADR-026 · Ingresos por tipo, egresos con modo exprés y nómina con prestaciones de ley

**Estado:** aceptada (2026-10-07). Las reglas de pago de la ley se movieron al módulo de Personal (`hr/domain/payroll.rs`, ADR-027); las aportaciones a la congregación, las becas y el personal externo cuentan como egresos aparte de la nómina.

## Contexto

La revisión del bloque «datos generales» de «Mi institución» encontró que el dinero no cuadraba:

- Las cuotas de los beneficiarios se veían dos veces: el padrón las calcula (quiénes pagan × cuota), pero también podían escribirse a mano como una fuente de ingreso más («Cuotas de recuperación»). Las dos cifras se mostraban sin relacionarse.
- El único egreso era la nómina (sueldo × 12). El «gasto anual aproximado» era un número suelto, sin comparar con nada, y a la IA le llegaba como «Presupuesto anual», que para ella puede querer decir lo que la institución tiene y no lo que gasta.
- El aviso «la suma de los ingresos no coincide con el gasto» aparecía con cualquier diferencia, aunque fuera de un peso. Que los ingresos no igualen el gasto no es un error: es el balance.
- La nómina anual no incluía las prestaciones de ley, así que el costo real del personal quedaba corto.

La institución aclaró cómo entra su dinero: las **cuotas de los beneficiarios** (una mensualidad o colegiatura fija), los **donantes fijos** (personas o empresas que dan cada mes o cada año), los **donativos ocasionales** (personas y empresas que donan cuando quieren) y los **donativos por proyecto** (convocatorias ganadas).

## Decisión

1. **Ingresos por tipo.** Cada ingreso escrito a mano lleva su tipo (`fee_estimate`, `recurring_donor`, `occasional_donation`, `project_grant`, `other`) y su periodo (`monthly` o `annual`). La persona escribe la cifra como la conoce y Rust la pasa a anual.
2. **Las cuotas de los beneficiarios salen del padrón.** Son una línea calculada (`beneficiary_fees`) que nadie edita. Mientras el padrón no tenga cuotas, se acepta una cuota aproximada escrita a mano (`fee_estimate`). En cuanto el padrón tiene cuotas, la aproximada deja de sumar (`counted: false`) y aparece el aviso `fee_estimate_ignored`.
3. **Egresos: lista por concepto o, para empezar, un aproximado.** La tabla `expense_item` guarda los conceptos (alimentos, servicios…) con su periodo. El `annual_budget_mxn` de siempre pasa a ser el **gasto anual aproximado** (modo exprés, todo incluido).
   - Mientras la lista esté vacía, el total de egresos es el aproximado (`expenses_basis: estimate`).
   - En cuanto la lista tiene una partida, el total es la lista más la nómina calculada (`list`), y el aproximado queda como referencia.
   - Sin lista ni aproximado, los egresos no se conocen (`unknown`).
4. **Nómina con prestaciones de ley**, como regla fija en `domain/finances.rs`:
   - Sueldo diario = mensual ÷ 30.
   - Aguinaldo = 15 días.
   - Prima vacacional = 25 % de los días de vacaciones según el artículo 76 de la LFT reformado en 2023: 12 días el primer año, +2 por año hasta 20 al quinto, y luego +2 cada cinco años.
   - La antigüedad sale del «Año en que entró» del padrón.
   - Honorarios no lleva prestaciones. Si falta el contrato o el año, se toma la opción segura para un presupuesto (con prestaciones, primer año) y se cuenta en `benefits_assumed`.
   - Las cuotas del IMSS y del INFONAVIT quedan fuera por ahora.
5. **El padrón conserva contrato y año de ingreso al agrupar** (`derive_staff`), porque sin ellos no se pueden calcular las prestaciones. El contrato se lee de la opción del selector aunque la persona la haya renombrado («De planta», «Base», «Eventual», «Honorarios»…).
6. **Balance** = ingresos contados − egresos. Rust lo calcula solo cuando los dos lados se conocen (`balance_annual_mxn`). El aviso estricto `income_differs_from_budget` desaparece. Los avisos nuevos (no impiden guardar):
   - `payroll_over_estimate`: la nómina sola supera el aproximado.
   - `expense_looks_like_payroll`: se escribió la nómina como gasto, y contaría dos veces.
   - `fee_estimate_ignored`: hay una cuota aproximada y el padrón ya tiene cuotas.
   - `rfc_format`, `phone_format` y `email_format`: el RFC, el teléfono o el correo no tienen la forma esperada.

   Bloquea guardar una cifra mayor a cien mil millones (`amount_too_large`).
7. **Lo que lee la IA** (`institution_context.rs`):
   - Recibe el gasto anual aproximado con ese nombre, cada ingreso escrito con su tipo y su cifra al mes y al año, cada egreso escrito, y la suma de lo escrito a mano.
   - De las cuotas del padrón solo sabe cuántas personas pagan. De la nómina solo sabe que existe y que lleva prestaciones. **Ninguna de las dos va como cifra, ni siquiera sumada**: con una sola persona con sueldo, el total sería su sueldo (ADR-020).
   - Por la misma razón, el balance llega en palabras («los ingresos alcanzan / NO alcanzan a cubrir los egresos») y nunca como cifra, porque de la cifra se podría despejar la nómina.
8. **Las cuentas son de Rust.** `ProfileView.finances` trae cada línea con su monto anual, los totales por tipo, fijos y variables, los egresos y el balance. La pantalla no suma; solo muestra.
9. Los montos se aceptan como la gente los escribe: «1,800,000», «$1 800 000» (`parse_pesos` en Rust y su gemelo en el formulario). En el padrón se guardan como dígitos.

## Consecuencias

- Migración 0014: `income_source.annual_amount_mxn` pasa a llamarse `amount_mxn` y se le agregan `period` y `kind`. Lo que ya existía queda como anual, y lo que decía «cuota» se marca como `fee_estimate`. Se crea `expense_item`.
- El JSON del perfil cambia: los ingresos llevan `kind`, `amount_mxn` y `period`, y aparecen `expenses` y `finances`. El frontend se ajustó lo mínimo para seguir funcionando; el diseño de ingresos por tipo, de la lista de egresos con el modo exprés y de la tarjeta de balance lo hace la sesión de diseño.
- El perfil guarda una línea de personal más fina (cargo, sueldo, contrato y año de ingreso). No lleva nombres, pero se vuelve más fácil reconocer a la persona en el perfil. El perfil no sale del equipo y la ficha de la IA no muestra el año de ingreso.
- Las sumas dependen del año en curso (antigüedad), que sale del reloj de la base (`current_year`) y viaja en `StoredProfile.as_of_year`. Las pruebas del dominio fijan el año.

## Alternativas descartadas

- **Dejar las cuotas como un ingreso escrito a mano:** el padrón ya tiene el dato exacto, y escribirlo dos veces es la forma más segura de contarlo dos veces.
- **Solo la lista de egresos, sin aproximado:** al empezar, casi ninguna institución tiene el desglose a la mano, y sin egresos no hay balance.
- **Mandar la nómina total a la IA cuando hay varias personas con sueldo:** necesita un umbral de anonimato y su propio ADR. Por ahora no se manda.
