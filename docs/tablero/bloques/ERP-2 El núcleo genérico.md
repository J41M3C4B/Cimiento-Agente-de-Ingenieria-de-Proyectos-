---
tipo: bloque
id: ERP-2
carril: erp
estado: revision
prioridad: media
fase: MVP-3
rama: erp/erp-2-nucleo-generico
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

El núcleo todavía supone un asilo o una casa hogar I.A.P.: «a quién atiende», la figura legal, el folio de la Junta, el Expediente, los ejemplos y los textos. Debe servir a cualquier organización de la sociedad civil (ADR-043 §4).

## Terminado cuando

(Aprobado por el líder el 2026-10-19, con sus decisiones de la revisión.)

- [ ] Con el ejemplo nuevo (una A.C. que no es residencia), el primer inicio y las cuatro ventanas de «Mi institución» se llenan sin una opción, pregunta ni texto ajeno a ella: no le piden capacidad y le preguntan a cuántas personas atendieron en el último año. Una institución mixta (residencia y programas en la comunidad) da las dos cifras, y ninguna cifra cambia de sentido al cambiar cómo atiende.
- [ ] Su Expediente pide solo los documentos que le tocan: sin Junta, sin el registro de centros de asistencia social, sin aviso sanitario, sin uso de suelo ni protección civil.
- [ ] Los dos ejemplos de residencia piden y muestran lo mismo que hoy: capacidad, «¿Cuántas personas atienden hoy?», ocupación, Junta y sus documentos.
- [ ] Ningún texto del núcleo (pantallas y manual) habla de asilo, casa hogar, residentes o abuelitos, salvo lo que escribió la institución.
- [ ] Lo que una institución ya guardó no cambia ni se pierde.
- [ ] Todas las pruebas pasan (`cargo test`, `pnpm test`), y el bloque `ia` con lo que le toca a la IA está en su Backlog.

**Anotado desde ERP-1:** varios textos todavía prometen los módulos congelados. Por ejemplo, el paso «Cuántas personas son» del primer inicio dice «después las fichas de cada persona la reemplazan», y la bienvenida habla de los datos de cada persona. Entran en la revisión de textos de este bloque.

## Revisión: lo que está atado a un asilo o una casa hogar

(CTO del ERP, 2026-10-19. Revisados: el catálogo de campos y formularios, los 14 datos del primer inicio, el Expediente, el resumen de Inicio, los ejemplos, los textos de la pantalla y el manual. **Cada punto espera tu decisión** en su última columna.)

**Lo que ya es genérico y se queda:**
- La figura legal: A.C., I.A.P., I.B.P., S.C., A.B.P., asociación religiosa y otra.
- El folio de la Junta, el registro ante la Junta y el presupuesto ante la Junta: ya aparecen solo para I.A.P., I.B.P. y A.B.P.
- La donataria, la CLUNI, el régimen fiscal, el RFC, los estados y municipios, el patronato y las cuentas.

### A. Qué pide el primer inicio y «Mi institución»

| #   | Hoy                                                                                                                                                  | Por qué es ajeno                                                                                                                | Recomendación                                                                                                                                                                                | Tu decisión                                                                                                                                                            |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1  | **La capacidad total** («Capacidad total (personas)») es indispensable para todas                                                                    | Solo tiene sentido donde las personas viven o pasan el día. Un comedor comunitario o un programa de becas no tiene «lugares»    | Pedirla, y que sea indispensable, solo si atienden con «Viven en la institución» o «Estancia de día». Sin ella, Inicio no muestra «% ocupado» ni «lugares vacíos» (ya pasa así cuando falta) | Si                                                                                                                                                                     |
| A2  | «¿Cuántas personas atienden **hoy**?»                                                                                                                | En una residencia es una cifra fija; un programa por temporadas atiende a cientos al año                                        | Pregunta «¿A cuántas personas atienden?», con la ayuda «En una residencia, las que viven hoy. Si atienden por programa o temporada, cuántas en un año»                                       | Esta pregunta debemos tener cuidado por que podemos tener un cliente tipo A.C donde pueden atender a muchas personas como mencionas por temporada o simplemente varia. **2026-10-19: según cómo atienden.** Con residencia o estancia de día, «¿Cuántas personas atienden hoy?»; con las demás, «¿A cuántas personas atendieron en el último año?», aproximado. Un solo dato, sin migración |
| A3  | **«¿A quién atienden?»** solo tiene grupos de edad, y es indispensable                                                                               | Una organización de medio ambiente, de cultura o de derechos humanos, o una fundación que da donativos, no tiene a quién marcar | Agregar dos opciones: «Población en general (sin un grupo de edad)» y «Otras organizaciones». Con ellas no se preguntan el sexo ni las edades                                                | Si                                                                                                                                                                     |
| A4  | «¿Cómo los atienden?» tiene residencia, estancia de día, consulta, comunidad y domicilio                                                             | Faltan formas comunes                                                                                                           | Agregar «A distancia o en línea» y «Con apoyos: dinero, especie o becas»                                                                                                                     | Si                                                                                                                                                                     |
| A5  | «¿En qué los apoyan?» es de asistencia: cuidado, salud, discapacidad, educación, alimentación, violencia, adicciones, calle, migración, salud mental | No cubre los rubros que el SAT sí reconoce para una donataria                                                                   | Agregar «Medio ambiente», «Cultura y arte», «Derechos humanos», «Desarrollo comunitario» y «Empleo y capacitación»                                                                           | Si                                                                                                                                                                     |
| A6  | «¿Qué piden para que alguien **ingrese**?»                                                                                                           | «Ingresar» supone una residencia                                                                                                | «¿Qué piden para atender a alguien?»                                                                                                                                                         | Si                                                                                                                                                                     |

Agregar opciones no borra ni cambia nada de lo guardado; no hace falta migración.

### B. El Expediente

| #   | Hoy                                                                                                | Por qué es ajeno                                                                                                     | Recomendación                                                                                                                                                  | Tu decisión  |
| --- | -------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| B1  | El **Registro Nacional de Centros de Asistencia Social** se pide a quien atiende a menores de edad | Ese registro es de los centros donde niñas, niños o adolescentes viven. Una asesoría escolar de tarde no lo necesita | Pedirlo solo si atiende a menores **y** «Viven en la institución» | Si |
| B2  | La **licencia de uso de suelo** y el **programa de protección civil** se piden a todas | Suponen un inmueble donde se atiende a personas. Una organización en línea o en la comunidad no los tiene | Pedirlos solo si atiende con «Viven en la institución», «Estancia de día» o «Consulta». Para las demás siguen en la lista como «otro documento», si los tienen | Sí (2026-10-19) |
| B3  | El **reglamento interno** se pide a todas | Cualquier organización puede tener uno | Se queda igual; solo se revisa que su texto del manual no hable de residentes | Sí (2026-10-19) |

### C. Textos de la pantalla y del manual

| #   | Dónde                                            | Hoy                                                                  | Recomendación                                                                                                                                                   | Tu decisión |
| --- | ------------------------------------------------ | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| C1  | Ayuda del nombre legal (pantalla y manual)       | «Asilo San José, I.A.P.»                                             | «Fundación Ejemplo, A.C.»                                                                                                                                       | Si          |
| C2  | Ejemplo de «¿A qué se dedica?»                   | «Somos una casa hogar que acompaña a niñas y niños…»                 | Un ejemplo que no sea residencia, por ejemplo: «Acompañamos a familias de la colonia con talleres, despensas y asesoría para que sus hijos sigan en la escuela» | Si          |
| C3  | Ayuda de los servicios                           | «residencia, comedor, terapia física o talleres»                     | «comedor, talleres, terapia, becas o asesoría»                                                                                                                  | Si          |
| C4  | Manual, «A quién atienden»                       | «personas mayores (los abuelitos)»                                   | Quitar «(los abuelitos)» y explicar las dos opciones nuevas                                                                                                     | Si          |
| C5  | Manual, capacidad                                | «qué tan llena está **la casa**»                                     | «qué tan ocupados están los lugares»; aclarar que es solo para residencias y estancias de día                                                                   | Si          |
| C6  | Manual, rubro de donataria                       | «Para un asilo o una casa hogar suele ser Asistencial»               | «Para una organización de asistencia social suele ser Asistencial»                                                                                              | Si          |
| C7  | Primer inicio y bienvenida (anotado desde ERP-1) | Prometen «las fichas de cada persona», que son de módulos congelados | Quitar esas promesas                                                                                                                                            | Si          |

### D. Ejemplos y pruebas

| #   | Hoy                                                                                                               | Recomendación                                                                                                                                                                                                                                                                         | Tu decisión |
| --- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| D1  | Los dos ejemplos de desarrollo (asilo y casa hogar) y la institución de la prueba completa son residencias I.A.P. | Agregar un **tercer ejemplo, ficticio**: una A.C. que no es residencia (por ejemplo, «Red Comunitaria Ejemplo, A.C.», con talleres y despensas en su colonia, sin Junta ni capacidad). Es el caso con el que se prueba este bloque. Los dos de hoy se quedan como casos de residencia | Si          |

### E. Lo que no se toca aquí

- **El «tipo de institución»** (`elderly_home`, `children_home`, en `catalog::kind_of`): sale de «a quién atienden» y solo lo usan los módulos congelados y la guía de Proyectos. Se queda; la guía va con [[ERP-3 Proyectos sobre la base nueva|ERP-3]].
- **Los textos de los módulos congelados** (camas, «residente permanente», pensiones de adultos mayores): están congelados (ADR-043 §3).
- **Las reglas que leen «I.A.P.» o «Junta» en un documento:** reconocen documentos reales de esas figuras y no suponen que todas lo sean.
- **Para el CTO de la IA, sin hacerse aquí:** la ficha de la IA (`ai_sheet.rs`), los oficios del Asistente y del Capturista, el Lector y sus baterías tienen ejemplos de asilo y van a recibir las opciones nuevas. Cuando decidas este bloque, se le pide un bloque `ia` con la lista final de opciones.

### «Terminado cuando» propuesto

(Se ajusta con tus decisiones.)

- [ ] Con el ejemplo nuevo (una A.C. que no es residencia), el primer inicio y las cuatro ventanas de «Mi institución» se llenan sin una opción, pregunta ni texto ajeno a ella.
- [ ] Su Expediente pide solo los documentos que le tocan: sin Junta, sin el registro de centros de asistencia social, sin aviso sanitario, sin uso de suelo ni protección civil.
- [ ] Los dos ejemplos de residencia piden y muestran lo mismo que hoy: capacidad, ocupación, Junta y sus documentos.
- [ ] Ningún texto del núcleo (pantallas y manual) habla de asilo, casa hogar, residentes o abuelitos, salvo lo que escribió la institución.
- [ ] Lo que una institución ya guardó no cambia ni se pierde.
- [ ] Todas las pruebas pasan (`cargo test`, `pnpm test`), y el bloque `ia` con lo que le toca a la IA está en su Backlog.

## Plan

(CTO del ERP, 2026-10-19.) Pasos pequeños; cada uno termina con sus pruebas.

1. **Una condición puede mirar otra ventana.** La capacidad está en «Capacidad y cifras» y la decide «¿Cómo los atienden?», que está en «Su institución». Hoy una condición solo ve los datos de su ventana.
   - `profile::forms` evalúa las condiciones con los datos de todas las ventanas.
   - La ventana recibe también los datos de otras ventanas que deciden sus campos (`FormView.context`).
   - El primer inicio no pide ni cuenta lo que no aplica.
   - La regla del ADR-042 «un dato indispensable siempre se pregunta» cambia a: siempre se pregunta, o lo decide otro dato indispensable que se pregunta antes. La prueba lo revisa.
2. **El catálogo (A1–A6).**
   - Opciones nuevas en «a quién», «cómo» y «en qué».
   - La capacidad solo con residencia o estancia de día.
   - El sexo y las edades solo con un grupo de edad.
   - Las palabras de la cifra de personas según cómo atienden: el código dice cuáles y la pantalla las muestra.
   - «¿Qué piden para atender a alguien?».
   - Inicio muestra la ocupación solo cuando la capacidad aplica.
3. **El Expediente (B1, B2).**
   - El registro de centros de asistencia social, solo con menores que viven en la institución.
   - Uso de suelo y protección civil, solo con residencia, estancia de día o consulta.
   - Un tipo puede depender de dos datos (`Applies::fields`).
4. **Los textos (C1–C7)** de la pantalla (`es-MX.ts`) y del manual (`docs/manual/`).
5. **El tercer ejemplo (D1):** «Red Comunitaria Ejemplo, A.C.», ficticio, con su botón de desarrollo junto a los otros dos, y su prueba: el primer inicio, las ventanas y el Expediente sin nada ajeno.
6. **El bloque `ia`** en su Backlog: las opciones nuevas, el nuevo sentido de la cifra de personas (al año si no es residencia), y los ejemplos de asilo de la ficha, los oficios y las baterías.
7. `cargo test` y `pnpm test` completos, la prueba en la app con el ejemplo nuevo y con uno de residencia, los documentos (`05-modelo-datos.md`, roadmap, mapa) y el PR.

**Lo que se guardó:** agregar opciones no toca nada. La capacidad que ya guardó una institución sin residencia ni estancia de día se queda en la base, sin mostrarse ni contar. Se borra solo si la persona vuelve a guardar esa ventana, como cualquier dato que deja de aplicar.

## Cómo probarlo

(CTO del ERP, 2026-10-19.) Con `pnpm tauri dev`. Para empezar desde cero sin tocar tus datos: `$env:CIMIENTO_DATA_DIR="D:\Agente Proyectos\cimiento-vacio"; pnpm tauri dev`.

**Una A.C. que no es residencia, desde el primer inicio:**
- [x] En «A quién atienden» marca **Población en general** y, en «Cómo», **En la comunidad** y **Con apoyos: dinero, especie o becas**. Ves las opciones nuevas en los dos.
- [x] En el paso «Cuántas personas son» **no** se pide la capacidad, y la pregunta es «¿A cuántas personas atendieron en el último año?».
- [x] Con el último dato, el primer inicio se cierra solo. Inicio muestra la cifra de personas, sin «% ocupado» ni lugares.
- [x] En «Mi institución» › «Su institución» **no** aparecen el sexo ni las edades. La última sección dice «¿Qué piden para atender a alguien?».
- [x] En «Capacidad y cifras rápidas» no está la capacidad.
- [x] El Expediente no pide el registro ante la Junta, el de centros de asistencia social, el aviso sanitario, la licencia de uso de suelo ni el programa de protección civil.

**El mismo caso con el ejemplo:** en la puesta en marcha (o abajo en «Mi institución»), «Red comunitaria de ejemplo». Debe quedar al 100 % sin nada de residencia.

**Una residencia sigue igual:** «Asilo de ejemplo».
- [x] Inicio y «Mi institución» dicen «18 de 25 lugares».
- [x] «Capacidad y cifras rápidas» pide la capacidad y «¿Cuántas personas atienden hoy?».
- [x] El Expediente pide la Junta, el aviso sanitario, el uso de suelo y protección civil, como antes.
- [x] Si en «Su institución» cambias «Viven en la institución» por «En la comunidad», la capacidad deja de verse y de contar. Si lo regresas, vuelve con su cifra.

**Los textos:**
- [x] La ayuda del nombre legal dice «Fundación Ejemplo, A.C.». El ejemplo de «Sobre nosotros» ya no es una casa hogar.
- [x] La bienvenida dice «Cuidamos los datos de las personas», sin prometer fichas.
- [x] En el manual («?» de cada campo), «A quién atienden» explica las dos opciones nuevas, sin «abuelitos».

**Vuelta 2 (tus observaciones):**
- [ ] Asilo de ejemplo: en «Su institución» marca también **En la comunidad**. «Capacidad y cifras» pide la capacidad, «hoy» **y** «en el último año».
- [ ] Llena el del año y quita «Viven en la institución»: Inicio muestra la cifra del año con «En el último año», sin lugares. Regresa «Viven en la institución»: vuelve «18 de 25 lugares».
- [ ] Carga el asilo, abre «Su institución» y ciérrala, y carga la red comunitaria: al abrirla otra vez se ve la red, sin nada del asilo. Su Expediente no muestra la Junta.
- [ ] En el Expediente, un documento «Lo tenemos» sin archivo dice «Sin archivo».

Lo probé así en la app real, con una base aparte: la A.C. desde el primer inicio y el asilo de ejemplo.

PR: https://github.com/J41M3C4B/Cimiento-Agente-de-Ingenieria-de-Proyectos-/pull/8

## Resultado
Si cambio como los atienden de "Viven en las institucion a Comunidad funciona", pero puedo tener ambas activas y gana viven en la institucion. Ademas el conteo se sigue quedando, osea en capacidad desaparece cuantos pueden atender y se queda cuanto atendieron el ultimo año pero ademas hay problema con cada ejemplo, como que hay un bug que mantiene por un momento los datos de otro ejemplo y entonces los numeros y datos se revuelven abro el asilo y despues abro el AC y me aparecen los datos del acilo en algunos campos

(Y en el chat: «el expediente aparece como si hubiera archivos pero no tienen nada, pero sus estados marcan cosas».)

**Respuesta (CTO del ERP, 2026-10-19).** El bloque volvió a «En curso» y regresa a «Revisión» con esto:
- **Mixta y cifra que se queda:** la causa era un solo dato con dos sentidos. Con tu decisión ahora son **dos datos**:
  - «¿Cuántas personas atienden hoy?», junto a la capacidad, donde viven o pasan el día;
  - «¿A cuántas personas atendieron en el último año?» (nuevo, migración 0034), con las demás formas;
  - una mixta da las dos;
  - al cambiar cómo atiende, la cifra que deja de aplicar se guarda sin verse, y vuelve si regresas. El 18 de «hoy» ya no se lee como «18 en el último año».
  - Inicio y «Mi institución» dicen «En el último año» cuando la cifra es del año.
- **Datos de un ejemplo en otro**, dos errores:
  - al cargar un ejemplo se quedaba el Expediente del anterior: la A.C. mostraba la Junta del asilo. Ahora se reemplaza;
  - las ventanas de «Mi institución» guardaban lo que habían leído antes y al reabrirlas mostraban esos datos viejos. Si guardabas, se escribían. Ahora empiezan siempre de lo guardado, como ya lo hacían las de listas y documentos.
- **Expediente con estados y sin archivos:** así lo permite la regla (se tiene el papel sin subirlo), y los ejemplos la usan. Con tu decisión, la fila de un documento que tienen sin archivo ahora dice **«Sin archivo»**.
