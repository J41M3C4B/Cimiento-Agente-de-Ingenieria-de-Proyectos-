---
tipo: guia
fecha: 2026-10-15
sobre: "[[ADR-036-cajones-de-documentos-y-un-agente-por-cajon]]"
---

# 15 · Guion de pruebas del núcleo

La prueba completa del núcleo por la institución: C4, fase 4 ([ADR-036](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)). Los tres agentes ya se aprobaron con sus baterías (fase 3). Lo que falta es que tú pruebes cada pantalla con la app abierta, como la usaría una directora.

El núcleo es: el primer inicio y las cuentas, **Inicio**, **Mi institución** con su Expediente, **Documentos**, el **Asistente** y las pantallas de servicio (Ayuda automática, Seguridad, Administración y Ayuda). Personal, Beneficiarios, Instalaciones, Finanzas y Proyectos **no entran** (nota `C1_NOTAS`): si aparecen, no los pruebes.

## Cómo usar este guion

- **Marca cada casilla** cuando el paso salga como dice «Debe pasar». En Obsidian basta con hacer clic en ella.
- **Si algo sale distinto, anótalo** en una nota de `notas/` con la plantilla «observacion»: en qué pantalla y en qué paso estabas, lo que hiciste, lo que esperabas y lo que pasó. Con una captura, mejor. Una nota por problema.
- **No hace falta hacerlo todo de una vez.** Cada sección se prueba sola, en el orden que quieras, salvo la 1, que va con una carpeta vacía.
- **Nunca escribas datos reales en las notas** (nombres, RFC, cuentas): la carpeta está en el repositorio.
- Al terminar, avísame. Leo tus notas, corrijo de raíz y te digo qué volver a probar.

## 0. Antes de empezar

- [ ] **Un respaldo de lo que tienes:** **Seguridad → Respaldo**, con una contraseña que recuerdes (sección 9).
- [ ] **La llave:** en **Ayuda automática**, «Probar ahora» dice «Todo en orden».
- [ ] **Los documentos para probar:**
  - con la llave gratuita, solo documentos ficticios: los de `fixtures/expediente/` (ábrelos en Word y guárdalos como PDF o Word) o documentos públicos;
  - **tus documentos reales, solo con llave de pago** (ADR-040): con la gratuita, Google puede usar lo que se le manda.
- [ ] Ten a la mano **una foto o un escaneo** de un documento ficticio y un PDF con contraseña (`D:\Agente Proyectos\pdf_pruebas\protected.pdf`).

**Cuánto gasta:** cada pregunta al Asistente, unos $0.02 MXN; «Llenar con mis documentos», unos $0.04 por ventana. Toda esta prueba, menos de $5 MXN. Con la llave gratuita, el límite es de 500 consultas al día.

## 1. Primer inicio y cuentas (con una carpeta vacía)

Para empezar desde cero sin tocar tus datos, abre la app con una carpeta aparte:
`$env:CIMIENTO_DATA_DIR="D:\Agente Proyectos\cimiento-vacio"; pnpm tauri dev`

**La cuenta de administración**
- [ ] Pide «Crear la cuenta de administración» con tu nombre, usuario y contraseña. Una contraseña de menos de 8 caracteres no pasa, y las dos deben coincidir.
- [ ] Después muestra el **código de recuperación**, con «Copiar el código». Avisa que no se vuelve a mostrar. Sigue con «Ya lo guardé».

**La bienvenida**
- [ ] Tres pantallas: «Le damos la bienvenida a SociAI», «Cuidamos a quienes atienden» y «Si tiene dudas, aquí estamos», con «Siguiente», «Atrás» y «Empezar».

**Puesta en marcha**
- [ ] Muestra tres cosas: la ayuda automática («Falta poner la llave» en una carpeta nueva), las cuentas y los datos de la institución (cuántos pasos faltan).
- [ ] «Lo hará la dirección» entra a la app sin llenar los datos. Inicio avisa «Faltan los datos de su institución», con «Llenarlos ahora».

**Los datos de la institución** (siete pasos: Su institución, Ubicación y contacto, Datos legales, Personas y equipo, Dinero, Su casa, Revisar)
- [ ] Cada paso guarda con «Guardar y seguir». Si sales a la mitad y vuelves, sigue donde te quedaste.
- [ ] Lo que falta para seguir aparece bajo «Para seguir falta:» y no deja avanzar.
- [ ] En «Revisar», lo que falta sale marcado. «Terminar y entrar» abre la app y dice «¡Listo! Los datos de su institución quedaron guardados.».
- [ ] Lo que escribiste aquí aparece igual en **Mi institución**: nada se pide dos veces.

## 2. Entrar, bloquear y las cuentas

**Entrar y bloquear**
- [ ] Toca tu nombre (arriba) → «Bloquear». Dice «SociAI está bloqueado» y pide tu contraseña; al entrar sigues donde estabas.
- [ ] Deja la app 15 minutos sin tocarla: se bloquea sola.
- [ ] Escribe mal la contraseña 5 veces: pide esperar unos segundos antes de intentar otra vez.
- [ ] «Olvidé mi contraseña» con tu código de recuperación pone una contraseña nueva.

**Administración → Personal y cuentas**
- [ ] «Dar acceso a alguien sin ficha» con «Dirección y contaduría» y una contraseña temporal.
- [ ] Entra con esa cuenta: pide «Cambie su contraseña» antes de seguir.
- [ ] Con esa cuenta **no aparecen** «Ayuda automática» ni «Administración», y en Seguridad no está «Restaurar un respaldo».
- [ ] Con esa cuenta, borra una persona del patronato: dice «Se pidió borrarlo. La persona administradora decide.».
- [ ] De vuelta como administración, en «Solicitudes» está lo que se pidió borrar, con «Borrar por completo» y «Devolverlo». «Devolverlo» lo regresa.
- [ ] «Bitácora» muestra las entradas, el cambio de contraseña y la solicitud. Nunca muestra contraseñas.
- [ ] «Desactivar» impide entrar con esa cuenta: «Esta cuenta está desactivada».

## 3. Inicio

- [ ] El saludo cambia con la hora («Buen día», «Buenas tardes» o «Buenas noches»).
- [ ] **Las cifras:** «Personas que atendemos» (de cuántos lugares, % ocupado) y «Balance del año». Sin datos de dinero dice «Todavía no se puede calcular el balance».
- [ ] Las cifras de personas coinciden con **Mi institución → Capacidad y cifras rápidas**. Cambia una allá, guarda y vuelve: Inicio la muestra cambiada.
- [ ] **La tarjeta de la institución:** su nombre, «N % de sus datos» y «Quienes tienen acceso». El porcentaje sube cuando llenas algo en Mi institución.
- [ ] «Por hacer» dice «Todavía no hay pendientes…», y «En curso» y «Notificaciones» se pueden abrir.
- [ ] Ningún botón lleva a una pantalla vacía o con error.

## 4. Mi institución

**Arriba**
- [ ] Dice «Faltan N datos por llenar». «Ver qué falta» abre la lista, y cada dato lleva a la ventana donde se llena.
- [ ] Al llenar uno y guardar, el número baja.

**Las cuatro ventanas** (Su institución, Contacto y ubicación, Datos legales y fiscales, Capacidad y cifras rápidas)
- [ ] Cada ventana guarda aunque falten datos. Lo que falta pasa a «Faltan N datos».
- [ ] **Lo que no cabe no se guarda**, y el mensaje dice cuál dato y por qué:
  - un código postal de 4 números;
  - un RFC mal escrito;
  - una edad mínima mayor que la máxima;
  - una fecha que no existe (30 de febrero).
- [ ] El **«?»** de cada dato dice «Qué poner» y «Para qué sirve», sin internet. «Preguntar más» abre el Asistente con la pregunta escrita.
- [ ] En «Datos legales», el folio de la Junta solo aparece si la figura es I.A.P., I.B.P. o A.B.P.; el rubro y el oficio de donataria, solo si es donataria autorizada («Sí»); la clave CLUNI, solo si tiene CLUNI.
- [ ] En «Notas generales» escribe un nombre de persona inventado con una CURP inventada y guarda. Debe aparecer «Ojo: encontramos datos de personas», con «Tapar y seguir» y «Mejor no guardarlo». La CURP no se puede dejar sin tapar.
- [ ] «Cancelar» o la X cierran la ventana sin guardar nada.

**Patronato y cuentas**
- [ ] Agrega una persona inventada al patronato con su cargo. «Ya no forma parte» la pasa aparte, sin borrarla.
- [ ] Agrega una cuenta con una CLABE ficticia de 18 números. Una CLABE con un número cambiado no pasa: el programa revisa su número de control.
- [ ] La CLABE se ve tapada. «Mostrar» la enseña y avisa «Quedó anotado que alguien la vio»; en la Bitácora aparece «Consultó la CLABE…».

## 5. El Expediente y Documentos

**El Expediente** (en Mi institución)
- [ ] Muestra los documentos que le tocan a tu institución, según su figura y lo que dijiste de ella, con «Tiene N de M», en trámite, por vencer y vencidos.
- [ ] Marca uno como «Lo tenemos» **sin subir archivo**: cuenta como que lo tienes.
- [ ] Pon una vigencia de aquí a 20 días: sale «Por vencer». Una fecha pasada: «Vencido».
- [ ] «Agregar otro documento» con un nombre tuyo («Convenio con el DIF»).

**Subir un archivo**
- [ ] **Poder del representante legal → Subir archivo**, con el poder ficticio en PDF:
  - mientras lee, dice que puedes cerrar la ventana; ciérrala y sigue trabajando;
  - la fila dice «Leyendo: N de M páginas» y al terminar sale un aviso de cómo fue.
- [ ] Al volver a abrirlo están llenas la fecha de emisión, la vigencia y el número de escritura, cada una con «Leído del archivo, página N».
- [ ] Corrige a mano la vigencia, guarda y sube otra vez el archivo: tu vigencia no cambia; la fecha de emisión y el número sí se leen del archivo nuevo.
- [ ] En una opinión de cumplimiento (32-D), la vigencia dice «Calculado con la fecha de emisión del archivo».
- [ ] La constancia del SAT deja **sugerencias** en «Datos legales y fiscales» (RFC, régimen, código postal fiscal), con «Lo leímos en «…», página N». Nada se guarda sin «Usar» y «Guardar».
- [ ] **Sugerencias solo en lo vacío:** escribe a mano el nombre legal y guarda; su sugerencia desaparece, y otro documento con el nombre ya no lo sugiere.
- [ ] «Descartar» una sugerencia y volver a subir el mismo archivo: no vuelve.
- [ ] **Una foto o un escaneo** se lee; si una página sale borrosa, avisa «con dudas en la página N».
- [ ] Un acta escaneada larga no congela la computadora mientras se lee. Una hoja escaneada de lado y un Word con una hoja escaneada pegada también se leen.
- [ ] **El lugar equivocado:** sube la constancia en el lugar del acta. Se guarda, avisa «Este archivo parece «Constancia de situación fiscal», no este documento» y no llena nada.
- [ ] **Con contraseña:** el PDF protegido avisa que tiene contraseña. No dice «dañado».
- [ ] «Quitar archivo»: se quita, y lo que ya se llenó con él se queda.

**Documentos** (en el menú)
- [ ] «Subir documento» pregunta «¿A qué cajón va?» (solo el Expediente) y «¿Qué documento es?». Al seguir, abre la misma ventana del Expediente.
- [ ] El documento se ve igual si lo subes aquí o desde Mi institución, agrupado por tipo o por año. El lápiz vuelve a abrir su ventana.
- [ ] Cambia el archivo de un documento: en Documentos queda solo el nuevo.
- [ ] «Buscar en el expediente» encuentra un documento por su nombre y, debajo, lo que dicen por dentro, con documento y página.
- [ ] Borra un documento del que ya aceptaste una sugerencia: se borra sin error y el dato aceptado se queda.

## 6. «Llenar con mis documentos»

Con al menos el acta, la constancia, el registro ante la Junta, el informe anual y el reglamento subidos.

- [ ] **Expediente → «Llenar todo con mis documentos»:** dice «Buscando en sus documentos…» y al terminar cuántas sugerencias dejó y en cuántas ventanas.
- [ ] Cada sugerencia dice de qué documento y página salió. Abre esa página y compruébalo: **una cifra sugerida está escrita tal cual en el documento**.
- [ ] Ningún campo que ya tenías escrito recibe sugerencia.
- [ ] Lo que ningún documento dice queda vacío: no inventa.
- [ ] En una ventana ya llena, el botón «Llenar con mis documentos» avisa «Todos los campos que puedo llenar en esta ventana ya están escritos» y no gasta nada.
- [ ] «Descartar» una sugerencia y volver a pulsar el botón: no vuelve a salir.

## 7. El Asistente

El botón con la estrella, arriba a la derecha, en cualquier pantalla.

**Sin documentos: el manual**
- [ ] «¿Cómo hago un respaldo?»: responde con el manual, dice de dónde lo tomó y ofrece «Ir a Seguridad».
- [ ] Apaga el internet y pregunta algo: dice que no hay conexión, responde con el manual y ofrece «Intentar de nuevo». Vuelve a prender el internet.
- [ ] Desde la pantalla donde ya estás, el botón «Ir a…» nunca lleva a esa misma pantalla.

**Con documentos**
- [ ] «Según mi registro, ¿qué nombre legal pongo?»: da el nombre y abajo «Lo tomé de sus documentos: ‹archivo›, página N». Ofrece ir a «Datos legales y fiscales» o «Llenar con mis documentos».
- [ ] Luego «Con base en ese mismo, ¿a quién atendemos?»: vuelve a citar el **mismo** documento.
- [ ] «¿Qué documentos tengo?»: la lista con sus nombres (los que tiene, los que faltan, en trámite, por vencer).
- [ ] «¿Qué dice ‹nombre de tu archivo›.pdf?», también con el nombre de la foto: contesta con lo que dice. Nunca dice que no puede ver imágenes.
- [ ] «¿Cuántos extintores tenemos?» o algo que **ningún documento dice**: dice que no lo encontró, sin inventar.
- [ ] «Llena los datos legales con lo que dicen mis documentos»: ofrece el botón «Llenar con mis documentos»; al pulsarlo dice cuántas sugerencias dejó, con «Revisarlas».
- [ ] La conversación se borra al cerrar la app.

**No debe pasar:** que cite un documento que no habla del tema, que dé una cifra que no está en la página que cita, o que te pida nombres de personas.

## 8. «Llenarlo por mí»

En una ventana de Mi institución, «Llenarlo por mí» abre el Asistente en modo de llenado («Llenando «…»»).

- [ ] **Capacidad:** «Tenemos 30 espacios disponibles para atender y viven 12 niñas» deja las dos cifras como sugerencias, cada una con «Usted dijo: «…»».
- [ ] **No suma ni supone:** «Tenemos 20 camas arriba y 15 abajo» **no** deja 35. «Vienen voluntarios de una parroquia» no deja ninguna cifra de voluntarios. Te dice qué no tomó.
- [ ] **Su institución:** «Atendemos a niñas de 6 a 17 años que no pueden vivir con su familia; viven en la casa y les damos comida y apoyo psicológico» deja a quién atienden, sexo, cómo, en qué los apoyan y las edades.
- [ ] **Contacto:** un domicilio ficticio completo y un teléfono. Cuando pregunte por los datos personales, contesta «Esto no son datos personales de nadie»: deben quedar calle, número, colonia, código postal, municipio, estado y teléfono, copiados tal cual. Si contestas «Tapar y seguir», no los deja.
- [ ] **Datos legales:** «Somos una I.A.P. fundada en 1987, donataria autorizada, y nuestro RFC es ‹uno ficticio›»: deja figura, año, donataria y RFC.
- [ ] Después de dejar sugerencias, el Asistente dice cuáles dejó y qué más puedes contarle.
- [ ] **Al cerrar la ventana** (Guardar, Cancelar o la X), el modo se apaga: tu siguiente pregunta la contesta el Asistente, no vuelve a llenar.
- [ ] **Desde la plática:** pregunta algo del poder y luego escribe «Ahora tenemos 35 espacios»: debe ofrecer «Llenarlo por mí» en Capacidad, sin volver al poder.
- [ ] Las sugerencias que no revisaste siguen ahí al volver: Mi institución avisa «Hay N sugerencias en «…»», con «Revisarlas».

## 9. Ayuda automática, Seguridad y Ayuda

**Ayuda automática** (solo administración)
- [ ] «Probar ahora» dice «Todo en orden» y no gasta respuestas del día.
- [ ] «Cómo se está usando» cuenta las respuestas de hoy y las del último minuto, y cuánto tarda.
- [ ] El tope del mes: pon uno menor a lo que dice «Este mes van $…». La ayuda automática se pausa (el Asistente responde con el manual) y lo demás sigue funcionando. Regrésalo a su valor.

**Seguridad**
- [ ] «Crear el respaldo» con contraseña: queda en Descargas con su nombre.
- [ ] «Restaurar un respaldo» (solo administración): primero pregunta «¿Restaurar este respaldo?» y avisa que lo actual se guarda aparte. Después del respaldo, todo vuelve como estaba. Pruébalo en la carpeta vacía de la sección 1.
- [ ] «Revisar ahora» dice cuántos textos revisó y, si encuentra algo, en qué lugar, sin mostrar los datos.

**Ayuda**
- [ ] Están los pasos de un proyecto, «Si algo no funciona» y «Sus datos».
- [ ] «Buscar en el manual» con tus palabras («¿qué pongo en a qué se dedica?») encuentra la respuesta sin internet.

## 10. En toda la app

- [ ] Ningún texto tiene palabras técnicas (JSON, API, token, modelo, error 429…).
- [ ] Ninguna pantalla se queda en «Cargando…» ni pierde lo que escribiste al cambiar de pantalla.
- [ ] El modo oscuro (la luna, arriba a la derecha) se lee bien en todas las pantallas del núcleo.
- [ ] Ventana pequeña (la mitad de la pantalla): nada se corta ni se encima.

## Lo que ya se probó con las baterías

Lo que dicen las secciones 6, 7 y 8 ya pasó por las baterías con el modelo más barato (fase 3): Asistente 32 de 32, Capturista 28 de 30, «Llenar con mis documentos» 35 de 36, nada inventado. Tu prueba ve lo que una batería no puede ver: si se entiende, si los botones llevan a donde deben, cómo se comporta con tus propias palabras y si algo estorba.
