# ADR-029 · Módulo de Beneficiarios e inteligencia de datos

**Estado:** aceptada (2026-10-07). Reemplaza lo que quedaba del ADR-020: el padrón genérico desaparece.

## Contexto

Los beneficiarios (adultos mayores en un asilo, niñas y niños en una casa hogar) eran el último uso del padrón genérico del ADR-020: una ficha en JSON con 8 datos (nombre, grupo, edad, nivel de apoyo, cuota, año de ingreso, teléfono y correo). A la IA solo le llegaba cuántas personas había por grupo.

La institución pidió tres cosas:

- Desacoplar los beneficiarios como se hizo con Personal (ADR-027), para migrarlos después a un módulo especializado.
- Recoger información básica pero relevante para la IA y para la operación.
- **Empezar a introducir al personal en la inteligencia de negocios y de datos**: que vean cómo datos que parecen insignificantes mejoran la calidad de los proyectos y de los presupuestos.

## Decisión

1. **Un módulo aparte** (`src-tauri/src/care/`, tablas `care_*`, migración 0017) con la misma frontera que Personal:
   - Usa solo la bitácora y `common`, y una prueba lo vigila.
   - La app le habla por `care::api` (`population_lines`, `indicators`, `ai_summary`) y por `care::service`.
   - La coordinación con el perfil vive en `care_service.rs`. El frontend vive en `src/features/care/`.
   - El tipo de institución lo pasa la app (`Flavor`): el módulo no lee el perfil.
2. **`common/`**: los validadores de CURP, RFC, NSS y CLABE, las fechas y la forma del teléfono y del correo salen del módulo de Personal a un módulo compartido sin dependencias. Así los dos módulos los usan sin depender uno del otro, y una prueba vigila que `common` no use nada de la app.
3. **Una ficha en cinco pasos** que se adapta al tipo de institución. Para guardar basta con el nombre y la fecha de nacimiento, o una edad aproximada que Rust convierte en el 1 de enero del año que da, marcado como aproximado. Cada paso muestra su avance.
   - **Identificación:** nombre, apellidos, fecha de nacimiento, sexo, CURP (validada y tapada), municipio y estado de origen, y lengua indígena.
     - Asilo: escolaridad y si sabe leer y escribir.
     - Casa hogar: si va a la escuela, el grado y si tiene rezago escolar.
     - Además, un grupo propio opcional.
   - **Ingreso y estancia:** fecha de ingreso, modalidad de estancia, quién la canalizó y motivos de ingreso (varios). La situación puede ser en atención, en el hospital, egresó o falleció, con su fecha y motivo. **Un egreso no borra la ficha.**
   - **Atención y salud, solo categorías:** nivel de apoyo, movilidad, discapacidad por tipo y condiciones crónicas de una lista.
     - Asilo: control de esfínteres y orientación.
     - Casa hogar: vacunas al día y atención psicológica.
     - Nunca diagnósticos escritos, medicinas ni médicos.
   - **Familia y responsable:** hasta 2 personas responsables (con la marca de tutor legal) y la frecuencia de visitas. En casa hogar, la **situación legal solo como categoría**, sin expediente ni números de juicio.
   - **Aportación y apoyos:** cuota mensual y quién la paga (familia, pensión propia, beca o exenta), pensiones y programas sociales, y el **aviso de privacidad firmado** (fecha y quién lo firmó).
4. **Grupos automáticos** por sexo y rango de edad según el tipo de institución: asilo 60–69, 70–79, 80–89 y 90 o más; casa hogar 0–5, 6–11, 12–17 y 18 o más. Por ejemplo, «Mujeres de 80 a 89 años» o «Niñas de 6 a 11 años». La institución puede crear **grupos propios**, y sus nombres pasan por el escáner porque llegan a la IA.
5. **Lista de espera** (`care_waitlist`):
   - **Qué guarda:** fecha, sexo, edad aproximada, nivel de apoyo y motivo; el nombre es opcional.
   - **Cuenta como demanda** en el tablero y en la ficha de la IA.
   - **«Darle ingreso»** crea la ficha (pide la edad).
   - **Quitar** una solicitud es solo del administrador: dirección y contaduría la marcan «Ya no lo necesita».
6. **Tablero** (`domain/insights.rs`):
   - **Indicadores**, calculados en Rust:
     - Ocupación contra capacidad y plazas libres.
     - Pirámide por edad y sexo.
     - Nivel de apoyo, movilidad, discapacidad y salud.
     - Permanencia, e ingresos, egresos y fallecimientos del año.
     - Visitas y personas sin responsable.
     - Pensiones y programas, exentos, y cuota promedio.
     - **Costo por persona al mes** (egresos de «Mi institución» ÷ 12 ÷ personas atendidas).
     - Lista de espera, avisos de privacidad que faltan y fichas incompletas.
   - **Hallazgos con reglas fijas** que cruzan datos de distintos lugares:
     - Personas en silla de ruedas o en cama contra espacios no accesibles.
     - Lista de espera contra plazas libres.
     - Ocupación alta.
     - Costo por persona contra cuota promedio: lo que cada persona necesita de donativos.
     - Personas con mucho apoyo contra personal de cuidado.
     - Pocas visitas.
     - Adultos de 65 o más sin pensión ni programa (podrían tramitar la Pensión para el Bienestar).
     - Rezago escolar.
     - Fichas sin aviso de privacidad o incompletas.
   - Cada hallazgo trae sus números; las palabras las pone la pantalla, o la ficha de la IA.
7. **Lo que llega a la IA** (`ai_summary` y los hallazgos marcados `for_ai`):
   - **Siempre:** conteos por grupo y total, ocupación, lista de espera, e ingresos y egresos del año.
   - **Solo con 3 o más personas:** nivel de apoyo, movilidad, discapacidad, salud, estancia, motivos de ingreso, visitas, pensiones, lengua indígena, escolaridad, rezago y situación legal. La edad promedio, solo con 3 o más.
   - **Nunca:** nombres, CURP, fechas de una persona, origen exacto, responsables y sus teléfonos, la situación legal de una niña, las cuotas de una persona, ni hallazgos con dinero (el costo por persona permitiría despejar la nómina, ADR-026).
8. **Perfil:** las líneas anónimas de población (grupo, nivel de apoyo, cuota: cuántas personas, edades mínima y máxima) salen del módulo (`profile_sync`), y con ellas se calculan las cuotas que ya usan las finanzas (ADR-026).
9. **Accesos (ADR-028):**
   - Borrar a una persona o un dato propio se vuelve solicitud para dirección y contaduría. Los códigos `beneficiary` y `roster_field` de `access_request` se conservan y ahora apuntan al módulo.
   - Consultar la CURP queda en la bitácora (`care.sensitive_viewed`), y borrar también (`care.person_deleted`).
10. **Traslado** (migración 0017 + `storage::migrations::move_roster_people`):
    - Cada ficha de beneficiario conserva **su id**, así que una solicitud de borrado pendiente la sigue encontrando, y se conserva si estaba oculta.
    - El grupo viejo da el sexo; la edad pasa a fecha aproximada y el año de ingreso a fecha aproximada.
    - El teléfono pasa a una persona responsable y el correo a un dato propio.
    - Nivel de apoyo, cuota y datos propios se conservan (también si estaban ocultos).
    - Se registra `care.imported` y se **eliminan** las tablas `roster_entry` y `roster_field`.

## Consecuencias

- **Ya no existe el padrón genérico.** Desaparecen `domain/roster.rs`, `storage/roster.rs`, `commands/roster.rs`, sus comandos `roster_*` y `RosterTab.tsx`, y `roster_service` pasa a llamarse `profile_sync`.
- **La ficha de la IA lee la población del módulo**, igual que el personal. Un perfil con líneas de población escritas a mano y sin personas en el módulo ya no las muestra; en una instalación real el perfil siempre se arma desde el módulo.
- **Privacidad:** son datos sensibles (salud, menores). Siguen solo en la base cifrada de este equipo. El aviso de privacidad por persona deja visibles en el tablero las fichas que no lo tienen. Hace falta la revisión legal de la Fase 7.
- **Frontend:** hay un frontend que funciona (lista, ficha en 5 pasos, tablero con indicadores y hallazgos, lista de espera); el diseño fino queda para la sesión de diseño.
- **Pendiente para el módulo especializado:**
  - Historial de ingresos y egresos (reingresos).
  - Expediente de documentos.
  - Bitácora diaria de cuidados.
  - Una escala funcional profesional (Barthel).
  - Visitas con fecha.
  - Exportar a Excel.

## Alternativas descartadas

- **Seguir con el padrón genérico:** no permite validar, cruzar datos ni migrar a un módulo especializado.
- **Grupos escogidos a mano:** el sexo y la edad ya dicen el grupo; escogerlos dejaba datos contradictorios.
- **Mandar a la IA el costo por persona:** con el resto de las cifras permitiría despejar la nómina.
- **Guardar diagnósticos o medicinas:** subía mucho el riesgo; las categorías bastan para los indicadores y los proyectos.
