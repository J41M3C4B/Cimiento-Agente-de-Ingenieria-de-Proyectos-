# ADR-020 · Padrón de personal y de beneficiarios (fichas individuales que no salen de este equipo)

**Estado:** Aceptada (2026-10-04), a petición de la persona dueña del proyecto. Implementada: `domain/roster.rs`, `storage/roster.rs`, `roster_service.rs`, `commands/roster.rs`, migración `0009_roster.sql` y las pestañas Personal y Beneficiarios de «Mi institución».

## Contexto
La regla «cuántos, nunca quiénes» (`docs/agents/principios-de-ingenieria.md`, `03-gobernanza-datos.md`) impedía registrar personas una por una. Para sacar la nómina y lo que aportan las cuotas, y para que la institución digitalice su operación, se pidió llevar un padrón con una ficha por persona (nombre completo, correo, teléfono, cargo, contrato, horario, sueldo; y lo equivalente para quien se atiende), con un formulario único y campos que la persona pueda agregar.

## Decisión
1. **El padrón es aparte del perfil.** Tablas nuevas `roster_entry` (una fila por persona, valores en JSON) y `roster_field` (los campos del formulario de cada tipo). Ninguna columna de identificación entra a `population_group` ni a `staff_group`: esas tablas **siguen sin poder identificar a nadie** y ahora se **calculan** a partir del padrón.
2. **La IA y los documentos solo ven agregados.** Al guardar una ficha, `roster_service` reescribe en el perfil vigente una línea por puesto (cargo, con o sin sueldo, mismo sueldo: cuántas personas) y una por grupo (grupo, nivel de apoyo, cuota: cuántas, edades mínima y máxima). El resumen que lee la IA (`profile_context`) y la guía en Word salen de eso; los sueldos no se envían. Hay una prueba que comprueba que ningún nombre, teléfono, correo, sueldo o cuota aparece en el perfil ni en lo que lee la IA.
3. **El padrón no pasa por el escáner de datos personales** (son datos personales a propósito) y **no está** en la lista de tablas del escaneo de la base. Vive en la base cifrada (SQLCipher), así que el respaldo cifrado lo incluye y se restaura igual.
4. **El formulario se configura.** Cada tipo (personal, beneficiarios) trae campos incluidos (los que la app calcula: cargo, sueldo, grupo, edad, nivel de apoyo, cuota) y la persona puede cambiar títulos y opciones de los selectores, y agregar datos propios: texto libre, selector con opciones o número. Los datos propios y los incluidos con título cambiado se guardan por clave; borrar un dato propio lo quita de las fichas. Las categorías de beneficiarios se sugieren según el tipo de institución (casa hogar o asilo) y se pueden editar.
5. **Totales en Rust.** Nómina mensual y anual, personas con sueldo y voluntarias, personas atendidas, quiénes pagan cuota y lo que suman al mes y al año se calculan en `domain` (`ProfileTotals`), nunca en pantalla ni por la IA.
6. **Guardar el perfil no borra los agregados:** `save_profile` los vuelve a calcular desde el padrón; la pantalla ya no envía personal ni grupos.

## Consecuencias y límites
- Esto **cambia el principio 5** de `docs/agents/principios-de-ingenieria.md`: «cuántos, nunca quiénes» sigue valiendo **para la IA, los documentos y el escáner**; el padrón guarda quiénes solo en este equipo. Antes de usar datos reales (Fase 7) hay que revisar el aviso de privacidad, el consentimiento (menores y datos de salud) y quién puede abrir la aplicación (el PIN es una cerradura, no protección de datos; ver ADR-019).
- Los datos viejos por grupo (versión anterior del perfil) no se convierten en personas: solo existían ejemplos ficticios. Los ejemplos de desarrollo se cargan como personas con nombres de relleno («Persona de ejemplo 01»).
- Pendiente por decidir: si la IA algún día debe leer algo de una ficha (hoy no), y exportar el padrón a Excel (hoy no existe).
