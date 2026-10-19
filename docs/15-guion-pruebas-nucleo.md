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

- [x] **Un respaldo de lo que tienes:** **Seguridad → Respaldo**, con una contraseña que recuerdes (sección 9).
- [x] **La llave:** en **Ayuda automática**, «Probar ahora» dice «Todo en orden».
- [x] **Los documentos para probar:**
  - con la llave gratuita, solo documentos ficticios: los de `fixtures/expediente/` (ábrelos en Word y guárdalos como PDF o Word) o documentos públicos;
  - **tus documentos reales, solo con llave de pago** (ADR-040): con la gratuita, Google puede usar lo que se le manda.
- [x] Ten a la mano **una foto o un escaneo** de un documento ficticio y un PDF con contraseña (`D:\Agente Proyectos\pdf_pruebas\protected.pdf`).

**Cuánto gasta:** cada pregunta al Asistente, unos $0.02 MXN; «Llenar con mis documentos», unos $0.04 por ventana. Toda esta prueba, menos de $5 MXN. Con la llave gratuita, el límite es de 500 consultas al día.

## 1. Primer inicio y cuentas (con una carpeta vacía)

Para empezar desde cero sin tocar tus datos, abre la app con una carpeta aparte:
`$env:CIMIENTO_DATA_DIR="D:\Agente Proyectos\cimiento-vacio"; pnpm tauri dev`

**La cuenta de administración**
- [x] Pide «Crear la cuenta de administración» con tu nombre, usuario y contraseña. Una contraseña de menos de 8 caracteres no pasa, y las dos deben coincidir.
- [x] Después muestra el **código de recuperación**, con «Copiar el código». Avisa que no se vuelve a mostrar. Sigue con «Ya lo guardé».

**La bienvenida**
- [x] Tres pantallas: «Le damos la bienvenida a SociAI», «Cuidamos a quienes atienden» y «Si tiene dudas, aquí estamos», con «Siguiente», «Atrás» y «Empezar».

**Puesta en marcha**
- [x] Muestra tres cosas: la ayuda automática («Falta poner la llave» en una carpeta nueva), las cuentas y los datos de la institución (cuántos pasos faltan).
- [x] «Lo hará la dirección» entra a la app sin llenar los datos. Inicio avisa «Faltan datos indispensables de su institución» y cuántos quedan, con «Llenarlos ahora».

**Los datos indispensables de la institución** (ADR-042: 14 datos en cinco pasos y la revisión: Su institución, A quién atienden, Dónde están, Cómo están constituidos, Cuántas personas son, Revisar)
- [x] No pide dinero, inmueble, contacto ni domicilio: eso se llena después.
- [x] Cada campo se ve y se revisa igual que en su ventana de **Mi institución** (las mismas palabras, opciones y «?»).
- [x] Cada paso guarda con «Guardar y seguir». Si sales a la mitad y vuelves, sigue donde te quedaste.
- [x] Lo que falta para seguir aparece bajo «Para seguir falta:», con el nombre de cada campo, y no deja avanzar. «0» voluntarias es una respuesta.
- [x] En «Revisar», lo que falta sale marcado. «Terminar y entrar» abre la app y dice «¡Listo! Los datos de su institución quedaron guardados.».
- [x] Lo que escribiste aquí aparece igual en **Mi institución**: nada se pide dos veces. Lo que sigue faltando ahí empieza por los indispensables.
- [x] Si llenas esos 14 datos desde las ventanas de **Mi institución**, el aviso de Inicio desaparece: es la misma cuenta.
- [x] En Inicio, «Siguientes pasos» pide registrar personal, personas que atienden, espacios e ingresos y egresos.

## 2. Entrar, bloquear y las cuentas

**Entrar y bloquear**
- [x] Toca tu nombre (arriba) → «Bloquear». Dice «SociAI está bloqueado» y pide tu contraseña; al entrar sigues donde estabas.
- [x] Deja la app 15 minutos sin tocarla: se bloquea sola.
- [x] Escribe mal la contraseña 5 veces: pide esperar unos segundos antes de intentar otra vez.
- [x] «Olvidé mi contraseña» con tu código de recuperación pone una contraseña nueva.

**Administración → Personal y cuentas**
- [x] «Dar acceso a alguien sin ficha» con «Dirección y contaduría» y una contraseña temporal.
- [x] Entra con esa cuenta: pide «Cambie su contraseña» antes de seguir.
- [x] Con esa cuenta **no aparecen** «Ayuda automática» ni «Administración», y en Seguridad no está «Restaurar un respaldo».
- [x] Con esa cuenta, borra una persona del patronato: dice «Se pidió borrarlo. La persona administradora decide.».
- [x] De vuelta como administración, en «Solicitudes» está lo que se pidió borrar, con «Borrar por completo» y «Devolverlo». «Devolverlo» lo regresa.
- [x] «Bitácora» muestra las entradas, el cambio de contraseña y la solicitud. Nunca muestra contraseñas.
- [x] «Desactivar» impide entrar con esa cuenta: «Esta cuenta está desactivada».

## 3. Inicio

- [x] El saludo cambia con la hora («Buen día», «Buenas tardes» o «Buenas noches»).
- [x] **Las cifras:** «Personas que atendemos» (de cuántos lugares, % ocupado) y «Balance del año». Sin datos de dinero dice «Todavía no se puede calcular el balance».
- [x] Las cifras de personas coinciden con **Mi institución → Capacidad y cifras rápidas**. Cambia una allá, guarda y vuelve: Inicio la muestra cambiada.
- [x] **La tarjeta de la institución:** su nombre, «N % de sus datos» y «Quienes tienen acceso». El porcentaje sube cuando llenas algo en Mi institución.
- [x] «Por hacer» dice «Todavía no hay pendientes…», y «En curso» y «Notificaciones» se pueden abrir.
- [x] Ningún botón lleva a una pantalla vacía o con error.

## 4. Mi institución

**Arriba**
- [x] Dice «Faltan N datos por llenar». «Ver qué falta» abre la lista, y cada dato lleva a la ventana donde se llena.
- [x] Al llenar uno y guardar, el número baja.

**Las cuatro ventanas** (Su institución, Contacto y ubicación, Datos legales y fiscales, Capacidad y cifras rápidas)
- [x] Cada ventana guarda aunque falten datos. Lo que falta pasa a «Faltan N datos».
- [x] **Lo que no cabe no se guarda**, y el mensaje dice cuál dato y por qué:
  - un código postal de 4 números;
  - un RFC mal escrito;
  - una edad mínima mayor que la máxima;
  - una fecha que no existe (30 de febrero).
- [x] El **«?»** de cada dato dice «Qué poner» y «Para qué sirve», sin internet. «Preguntar más» abre el Asistente con la pregunta escrita.
- [x] En «Datos legales», el folio de la Junta solo aparece si la figura es I.A.P., I.B.P. o A.B.P.; el rubro y el oficio de donataria, solo si es donataria autorizada («Sí»); la clave CLUNI, solo si tiene CLUNI.
- [x] En «Notas generales» escribe un nombre de persona inventado con una CURP inventada y guarda. Debe aparecer «Ojo: encontramos datos de personas», con «Tapar y seguir» y «Mejor no guardarlo». La CURP no se puede dejar sin tapar.
- [x] «Cancelar» o la X cierran la ventana sin guardar nada.

**Patronato y cuentas**
- [x] Agrega una persona inventada al patronato con su cargo. «Ya no forma parte» la pasa aparte, sin borrarla.
- [x] Agrega una cuenta con una CLABE ficticia de 18 números. Una CLABE con un número cambiado no pasa: el programa revisa su número de control.
- [x] La CLABE se ve tapada. «Mostrar» la enseña y avisa «Quedó anotado que alguien la vio»; en la Bitácora aparece «Consultó la CLABE…».

## 5. El Expediente y Documentos

**El Expediente** (en Mi institución)
- [x] Muestra los documentos que le tocan a tu institución, según su figura y lo que dijiste de ella, con «Tiene N de M», en trámite, por vencer y vencidos.
- [x] Marca uno como «Lo tenemos» **sin subir archivo**: cuenta como que lo tienes.
- [x] Pon una vigencia de aquí a 20 días: sale «Por vencer». Una fecha pasada: «Vencido».
- [x] «Agregar otro documento» con un nombre tuyo («Convenio con el DIF»).

**Subir un archivo**
- [x] **Poder del representante legal → Subir archivo**, con el poder ficticio en PDF:
  - mientras lee, dice que puedes cerrar la ventana; ciérrala y sigue trabajando;
  - la fila dice «Leyendo: N de M páginas» y al terminar sale un aviso de cómo fue.
- [x] Al volver a abrirlo están llenas la fecha de emisión, la vigencia y el número de escritura, cada una con «Leído del archivo, página N».
- [x] Corrige a mano la vigencia, guarda y sube otra vez el archivo: tu vigencia no cambia; la fecha de emisión y el número sí se leen del archivo nuevo.
- [x] En una opinión de cumplimiento (32-D), la vigencia dice «Calculado con la fecha de emisión del archivo».
- [x] La constancia del SAT deja **sugerencias** en «Datos legales y fiscales» (RFC, régimen, código postal fiscal), con «Lo leímos en «…», página N». Nada se guarda sin «Usar» y «Guardar».
- [x] **Sugerencias solo en lo vacío:** escribe a mano el nombre legal y guarda; su sugerencia desaparece, y otro documento con el nombre ya no lo sugiere.
- [x] «Descartar» una sugerencia y volver a subir el mismo archivo: no vuelve.
- [x] **Una foto o un escaneo** se lee; si una página sale borrosa, avisa «con dudas en la página N».
- [x] Un acta escaneada larga no congela la computadora mientras se lee. Una hoja escaneada de lado y un Word con una hoja escaneada pegada también se leen.
- [x] **El lugar equivocado** (ADR-042, fase 2): sube la constancia en el lugar del acta. **No se guarda:** pregunta «Este archivo parece «Constancia de situación fiscal»», con «Guardarlo en «Constancia de situación fiscal»», «Sí es «Acta constitutiva»» y «No guardarlo».
  - «No guardarlo» (o la X): el acta sigue sin archivo y no aparece ninguna sugerencia.
  - «Guardarlo en «Constancia…»»: queda en la constancia y deja sus sugerencias. Si la constancia ya tenía archivo, la pregunta avisa que lo reemplaza.
  - «Sí es «Acta constitutiva»»: queda en el acta, y la Bitácora dice «Dijo que un archivo es el documento donde lo subió, aunque parecía otro».
  - Cierra la ventana mientras se lee: la pregunta sale igual, donde estés.
- [x] **Una foto que no se lee**, en cualquier documento: pregunta «No pudimos confirmar que sea «…»». Con «Sí es…» se guarda. Si el nombre del archivo dice el documento («licencia uso de suelo.jpg»), no pregunta.
- [x] **Sugerencias al subir** (ADR-042, fase 3; necesita la ayuda automática con su llave): con «Capacidad y cifras rápidas» sin el personal, sube el informe anual. Unos segundos después de «Lo tenemos», un aviso dice «encontramos N datos más para «Mi institución»» y en qué ventanas esperan. Ahí están el personal con sueldo (31) y el voluntario (18), con «Lo leímos en «…», página 1», sin pulsar «Llenar con mis documentos».
  - Un archivo que no habla de ningún campo vacío (la CLUNI, con la CLUNI ya escrita) no dice nada más.
  - «Notas generales» nunca se llena sola.
- [x] **Cómo se escriben** (ADR-042, fase 5): con el municipio vacío, sube la constancia del SAT y pulsa «Llenar con mis documentos» en «Contacto y ubicación». La sugerencia del municipio dice «Benito Juárez», con acento, aunque la constancia lo escriba «BENITO JUAREZ». En «Llenarlo por mí», «Calle de los Ejemplos 128» deja la calle «de los Ejemplos».
- [x] **Con contraseña:** el PDF protegido avisa que tiene contraseña. No dice «dañado».
- [x] «Quitar archivo»: se quita, y lo que ya se llenó con él se queda.

**Documentos** (en el menú)
- [x] «Subir documento» pregunta «¿A qué cajón va?» (solo el Expediente) y «¿Qué documento es?». Al seguir, abre la misma ventana del Expediente.
- [x] El documento se ve igual si lo subes aquí o desde Mi institución, agrupado por tipo o por año. El lápiz vuelve a abrir su ventana.
- [x] Cambia el archivo de un documento: en Documentos queda solo el nuevo.
- [x] «Buscar en el expediente» encuentra un documento por su nombre y, debajo, lo que dicen por dentro, con documento y página.
- [x] Borra un documento del que ya aceptaste una sugerencia: se borra sin error y el dato aceptado se queda.

## 6. «Llenar con mis documentos»

Con al menos el acta, la constancia, el registro ante la Junta, el informe anual y el reglamento subidos.

- [x] **Expediente → «Llenar todo con mis documentos»:** dice «Buscando en sus documentos…» y al terminar cuántas sugerencias dejó y en cuántas ventanas.
- [x] Cada sugerencia dice de qué documento y página salió. Abre esa página y compruébalo: **una cifra sugerida está escrita tal cual en el documento**.
- [x] Ningún campo que ya tenías escrito recibe sugerencia.
- [x] Lo que ningún documento dice queda vacío: no inventa.
- [x] En una ventana ya llena, el botón «Llenar con mis documentos» avisa «Todos los campos que puedo llenar en esta ventana ya están escritos» y no gasta nada.
- [x] «Descartar» una sugerencia y volver a pulsar el botón: no vuelve a salir.

## 7. El Asistente

El botón con la estrella, arriba a la derecha, en cualquier pantalla.

**Sin documentos: el manual**
- [x] «¿Cómo hago un respaldo?»: responde con el manual, dice de dónde lo tomó y ofrece «Ir a Seguridad».
- [x] Apaga el internet y pregunta algo: dice que no hay conexión, responde con el manual y ofrece «Intentar de nuevo». Vuelve a prender el internet.
- [x] Desde la pantalla donde ya estás, el botón «Ir a…» nunca lleva a esa misma pantalla.

**Con documentos**
- [x] «Según mi registro, ¿qué nombre legal pongo?»: da el nombre y abajo «Lo tomé de sus documentos: ‹archivo›, página N». Ofrece ir a «Datos legales y fiscales» o «Llenar con mis documentos».
- [x] Luego «Con base en ese mismo, ¿a quién atendemos?»: vuelve a citar el **mismo** documento.
- [x] «¿Qué documentos tengo?»: la lista con sus nombres (los que tiene, los que faltan, en trámite, por vencer).
- [x] «¿Qué dice ‹nombre de tu archivo›.pdf?», también con el nombre de la foto: contesta con lo que dice. Nunca dice que no puede ver imágenes.
- [x] «¿Cuántos kilos de comida nos donó el banco de alimentos en 2024?» o algo que **ningún documento dice**: dice que no lo encontró, sin inventar.
- [x] «Llena los datos legales con lo que dicen mis documentos» (ADR-042, fase 4): **ya no** ofrece un botón en el chat. Dice que al subir cada documento ya llegan sus sugerencias y que el botón «Llenar con mis documentos» está en la ventana, y ofrece ir a «Datos legales y fiscales».
- [x] La conversación se borra al cerrar la app.

**No debe pasar:** que cite un documento que no habla del tema, que dé una cifra que no está en la página que cita, o que te pida nombres de personas.

## 8. «Llenarlo por mí»

En una ventana de Mi institución, «Llenarlo por mí» abre el Asistente en modo de llenado («Llenando «…»»).

- [x] **Capacidad:** «Tenemos 30 espacios disponibles para atender y viven 12 niñas» deja las dos cifras como sugerencias, cada una con «Usted dijo: «…»».
- [x] **No suma ni supone:** «Tenemos 20 camas arriba y 15 abajo» **no** deja 35. «Vienen voluntarios de una parroquia» no deja ninguna cifra de voluntarios. Te dice qué no tomó.
- [x] **Su institución:** «Atendemos a niñas de 6 a 17 años que no pueden vivir con su familia; viven en la casa y les damos comida y apoyo psicológico» deja a quién atienden, sexo, cómo, en qué los apoyan y las edades.
- [x] **Contacto:** un domicilio ficticio completo y un teléfono. Cuando pregunte por los datos personales, contesta «Esto no son datos personales de nadie»: deben quedar calle, número, colonia, código postal, municipio, estado y teléfono, copiados tal cual. Si contestas «Tapar y seguir», no los deja.
- [x] **Datos legales:** «Somos una I.A.P. fundada en 1987, donataria autorizada, y nuestro RFC es ‹uno ficticio›»: deja figura, año, donataria y RFC.
- [x] Después de dejar sugerencias, el Asistente dice cuáles dejó y qué más puedes contarle.
- [x] **Al cerrar la ventana** (Guardar, Cancelar o la X), el modo se apaga: tu siguiente pregunta la contesta el Asistente, no vuelve a llenar.
- [x] **Desde la plática:** pregunta algo del poder y luego escribe «Ahora tenemos 35 espacios»: debe ofrecer «Llenarlo por mí» en Capacidad, sin volver al poder.
- [x] Las sugerencias que no revisaste siguen ahí al volver: Mi institución avisa «Hay N sugerencias en «…»», con «Revisarlas».
- [x] **Cada dato a su ventana** (ADR-042, fase 4): en «Contacto y ubicación», «Llenarlo por mí» y escribe «Nos representa la directora, ‹un nombre inventado›» (contesta «Esto no son datos personales de nadie»). La sugerencia queda en «Datos legales y fiscales», y el Asistente dice «Dejé 1 sugerencia en «Datos legales y fiscales»», con un botón para ir ahí. Ya no dice «no encontré datos para «Contacto y ubicación»».
- [x] **Lo igual no se repite:** cuéntale un dato que ya guardaste, igual (por ejemplo, la capacidad). No deja sugerencia y dice «Ya estaba guardado igual: …».
- [x] **En la plática normal**, sin «Llenarlo por mí»: escribe «Nos representa la directora, ‹un nombre inventado›». El Asistente nunca dice «dejé sugerencias»: la frase con el botón «Llenarlo por mí» la escribe el programa.
- [ ] **Desde la plática, con un dato ya guardado:** escribe «Estamos ubicados en la alcaldía ‹la que ya guardaste›» y pulsa «Llenarlo por mí». Dice «Ya estaba guardado igual: …» y **no** se queda en «Llenando «…»»: tu siguiente pregunta la contesta el Asistente. El modo de llenado empieza desde la plática solo cuando deja sugerencias en esa ventana.
- [ ] **Tu frase una sola vez:** si al pulsar «Llenarlo por mí» pregunta por datos personales (un nombre como «Benito Juárez» lo parece) y contestas, tu frase no vuelve a aparecer en el chat.
- [ ] **Una alcaldía no es una persona:** «Estamos ubicados en la alcaldía Benito Juárez» ya no pregunta por datos personales; con «Llenarlo por mí» deja la alcaldía o dice «Ya estaba guardado igual».

## 9. Ayuda automática, Seguridad y Ayuda

**Ayuda automática** (solo administración)
- [x] «Probar ahora» dice «Todo en orden» y no gasta respuestas del día.
- [x] «Cómo se está usando» cuenta las respuestas de hoy y las del último minuto, y cuánto tarda.
- [x] El tope del mes: pon uno menor a lo que dice «Este mes van $…». La ayuda automática se pausa (el Asistente responde con el manual) y lo demás sigue funcionando. Regrésalo a su valor.

**Seguridad**
- [x] «Crear el respaldo» con contraseña: queda en Descargas con su nombre.
- [x] «Restaurar un respaldo» (solo administración): primero pregunta «¿Restaurar este respaldo?» y avisa que lo actual se guarda aparte. Después del respaldo, todo vuelve como estaba. Pruébalo en la carpeta vacía de la sección 1.
- [x] «Revisar ahora» dice cuántos textos revisó y, si encuentra algo, en qué lugar, sin mostrar los datos.

**Ayuda**
- [x] Están los pasos de un proyecto, «Si algo no funciona» y «Sus datos».
- [x] «Buscar en el manual» con tus palabras («¿qué pongo en a qué se dedica?») encuentra la respuesta sin internet.

## 10. En toda la app

- [x] Ningún texto tiene palabras técnicas (JSON, API, token, modelo, error 429…).
- [x] Ninguna pantalla se queda en «Cargando…» ni pierde lo que escribiste al cambiar de pantalla.
- [x] El modo oscuro (la luna, arriba a la derecha) se lee bien en todas las pantallas del núcleo.
- [x] Ventana pequeña (la mitad de la pantalla): nada se corta ni se encima.

## La prueba completa con la institución de prueba

Para probar todo el camino de una vez, con 24 archivos ficticios (escaneos, fotos, Word, archivos con fallas) y lo que debe quedar al final, sigue [16 · La institución de prueba](16-institucion-de-prueba.md). Ahí está también la tabla para calificar el núcleo.

## Lo que ya se probó con las baterías

Lo que dicen las secciones 6, 7 y 8 ya pasó por las baterías con el modelo más barato (fase 3): Asistente 32 de 32, Capturista 28 de 30, «Llenar con mis documentos» 35 de 36, nada inventado. Tu prueba ve lo que una batería no puede ver: si se entiende, si los botones llevan a donde deben, cómo se comporta con tus propias palabras y si algo estorba.
