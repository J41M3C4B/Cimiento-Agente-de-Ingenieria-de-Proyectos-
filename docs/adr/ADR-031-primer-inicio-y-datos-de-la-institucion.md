# ADR-031 · Primer inicio y datos obligatorios de la institución

**Estado:** aceptada (2026-10-08).

## Contexto

Al terminar de crear la cuenta del administrador (ADR-028), la app entraba directo y vacía. En «Mi institución» solo había un recuadro que invitaba a empezar y que se podía ignorar. Así, la IA trabajaba sin saber a qué se dedica la institución, dónde está, cuánto gasta o a cuántas personas atiende, y terminaba adivinando (la ficha decía «no capturado»).

El perfil tampoco guardaba lo que casi todas las convocatorias revisan primero:

- El estado y el municipio.
- Los años de operación.
- La figura jurídica.
- Si es donataria autorizada y si tiene CLUNI.

La institución pidió un primer inicio que obligue a llenar los datos básicos que necesita la IA, y decidir quién lo ve.

## Decisión

1. **Tres momentos:**
   - **Bienvenida** (una vez por cuenta, `app_user.welcomed_at`). Son tres pantallas: qué hace Cimiento, cómo cuidamos los datos y dónde pedir ayuda. Va después de cambiar la contraseña temporal.
   - **Puesta en marcha** (solo el administrador). Muestra si la ayuda automática tiene llave, cuántas cuentas de dirección y contaduría hay y cuántos pasos de datos faltan. Desde ahí llena los datos o **los deja a la dirección** y entra a la app (por esa sesión de la ventana). En desarrollo, ahí mismo se cargan los ejemplos.
   - **Datos de la institución** (obligatorios). Son **de la institución, no de la persona**. Mientras falte uno, dirección y contaduría ven el asistente en lugar de la app y no lo pueden saltar.
2. **Seis pasos y una revisión** (`domain/onboarding.rs`; Rust decide qué falta):

   | Paso | Obligatorio | Opcional |
   |---|---|---|
   | Su institución | nombre, tipo, a qué se dedica | |
   | Dónde y quiénes son | estado, municipio, un teléfono o correo, figura jurídica, año de fundación, donataria autorizada (sí/en trámite/no), CLUNI (sí/en trámite/no) | RFC, representante |
   | A quién atienden | capacidad, cuántas personas atienden hoy | |
   | Su equipo | cuántas con sueldo y cuántas voluntarias (0 es respuesta) | |
   | Dinero | gasto anual aproximado (o la lista de egresos), al menos una fuente de ingreso con monto | |
   | Su casa | pisos, de quién es el inmueble | m², hasta qué año, papeles |

3. **Cifras rápidas.** El personal y los beneficiarios viven en módulos con una ficha por persona (ADR-027 y ADR-029). Para no hacer eterno el primer inicio, el asistente pide solo cuántos son (`served_estimate`, `staff_paid_estimate` y `staff_volunteer_estimate` en `institution_profile`).
   - La IA las recibe como «cifra aproximada que dio la persona; todavía sin registros».
   - **Las fichas mandan:** cuando un módulo tiene personas, el paso se da por cumplido, la cifra no se pide y la IA no la recibe.
   - Las cuotas de los beneficiarios registrados cuentan como ingreso.
4. **Se guarda cada paso** (`onboarding_save`):
   - El perfil queda como borrador, pasa por el escáner y por las validaciones de siempre.
   - Lo del inmueble va al módulo de Instalaciones (ADR-030).
   - Se puede salir y seguir donde se quedó: el asistente abre en el primer paso incompleto.
   - Solo se avanza cuando Rust dice que el paso está completo. Si falta algo, la pantalla lo nombra.
5. **Al terminar** (`onboarding_finish`):
   - Exige todos los pasos completos.
   - Confirma el perfil, que queda como su primera versión.
   - Marca `institution.onboarded_at` **una sola vez**: el asistente no vuelve aunque después se borre un dato, porque lo que falte se ve en «Mi institución» y en la ficha de la IA.
   - Se registra `institution.onboarded` en la bitácora.
6. **Datos nuevos de la institución** (migración 0019): `state` (32 estados), `municipality`, `founded_year`, `legal_form` (A.C., I.A.P., I.B.P., S.C., A.B.P., asociación religiosa, otra), `authorized_donee` y `cluni`.
   - **No son datos personales y llegan a la IA:** ubicación, años de operación, figura jurídica, donataria y CLUNI.
   - **El contacto sigue sin llegar a la IA.**
7. **Permisos:**
   - `onboarding_status` y `onboarding_welcome_done` solo piden sesión.
   - `onboarding_save` y `onboarding_finish` piden `Use`.
   - Rust dice quién puede dejarlo para después (`can_postpone`, solo el administrador).
   - La pantalla hace de portero (`OnboardingGate`, entre `AccessGate` y la app): los demás comandos no se bloquean en Rust mientras no se termine.
8. **Prueba desde cero sin tocar los ejemplos:** solo en desarrollo, `CIMIENTO_DATA_DIR` abre la app con otra carpeta de datos. Los ejemplos cargados con los botones se marcan como institución terminada.

## Consecuencias

- **Una instalación existente sin los datos nuevos ve el asistente,** que abre en el primer paso incompleto (por lo general «Dónde y quiénes son»). Los ejemplos ya traen los datos nuevos.
- **«Mi institución» muestra y edita los datos nuevos:**
  - Estado y municipio, en «Contacto».
  - Figura jurídica, año de fundación, donataria y CLUNI, en «Datos legales».
  - Las cifras rápidas, en «Capacidad y gasto».
- **Frontend:** hay un frontend que funciona (bienvenida, puesta en marcha, asistente con revisión). El diseño fino queda para la sesión de diseño.
- **Pendiente:**
  - Una lista de «Siguientes pasos» en Inicio después de terminar (registrar personal, beneficiarios y espacios; conectar la IA).
  - Validar el municipio contra el catálogo del INEGI.
  - Que la IA cruce el estado y la figura jurídica con los requisitos de cada convocatoria.

## Alternativas descartadas

- **Que solo el administrador llene los datos:** él no conoce el día a día de la institución, y la dirección entraría a una app que no reconoce.
- **Exigir las fichas de cada persona en el primer inicio:** son decenas de fichas; el asistente se volvería eterno y se abandonaría.
- **Bloquear en Rust todos los comandos hasta terminar:** el administrador necesita entrar a configurar la IA y las cuentas; el portero en la pantalla, con reglas decididas en Rust, basta.
- **Que el asistente vuelva cada vez que falte un dato:** cansaría. Una vez terminado, lo que falte se ve donde corresponde.
