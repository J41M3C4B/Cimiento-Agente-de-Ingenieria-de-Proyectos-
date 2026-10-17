---
tipo: mapa
actualizado: 2026-10-16
---

# Mapa del proyecto

La página de inicio: dónde estamos, qué sigue, qué espera tu respuesta y dónde está cada cosa. **Claude la actualiza al cerrar cada bloque.**

> [!info] Hoy (2026-10-16, noche)
> **Probaste el guion completo (sección 15): todo salió bien.** Siguió la fase 2 del [ADR-042](adr/ADR-042-primer-inicio-solo-lo-indispensable.md) (tu problema 4).
>
> **Fase 2 hecha: un archivo solo se guarda en su lugar si el código confía en que es de ahí.**
> - El código cuenta qué tan seguro está: las palabras de ese documento en el archivo, contra las de cualquier otro, y el nombre del archivo.
> - Si sus palabras dicen que es otro documento, **no lo guarda**: pregunta «Este archivo parece «…»», con «Guardarlo en «…»», «Sí es «…»» y «No guardarlo». Si nada lo reconoce (una foto que no se lee), pregunta si es ese documento. Mientras pregunta no se escribe nada, ni sugerencias.
> - El umbral se midió en seco con los 24 archivos de la prueba completa y los 13 de la batería: de 563 lugares equivocados, **ninguno** se guardó; en su lugar, solo pregunta por la foto sin texto.
> - La medición encontró dos reglas que fallaban y se corrigieron: la licencia dice «uso **del** suelo», y la copia desteñida se lee «Junta**l** de asistencia privada».
> - «Sí es…» queda en la Bitácora.
>
> **Siguiente:** que pruebes la fase 2 ([guion, sección 5](15-guion-pruebas-nucleo.md), las dos casillas nuevas sin marcar); después, la fase 3 (sugerencias al subir).
>
> **Antes (2026-10-16): la prueba completa del núcleo** ([[Observaciones Pruebas]]): 33 de 39 datos quedaron bien; lo que falló fue el camino y el diseño. De ahí salió el [ADR-042](adr/ADR-042-primer-inicio-solo-lo-indispensable.md): un solo camino para llenar la institución, en cinco fases.
>
> **Fase 1 hecha: el primer inicio pide solo lo indispensable.**
> - 14 datos en cinco pasos cortos (antes unos 19 en seis). Ya no pide dinero, inmueble ni contacto.
> - Los decide el código: todo campo que cambia lo que el programa pide (a quién atienden, cómo, figura jurídica, donataria, CLUNI) es indispensable, y una prueba lo revisa.
> - Cada paso se guarda por su ventana de «Mi institución», con las mismas palabras, reglas, escáner e historial.
> - Se cierra solo con el último dato indispensable, venga del primer inicio o de una ventana. El aviso de Inicio y la tarjeta de «Mi institución» salen de la misma cuenta: ya no puede pasar lo del problema 1.
> - Dinero, personal, beneficiarios y espacios se piden en «Siguientes pasos» de Inicio (ahora con Finanzas).
> - Al probarlo en la app encontré otro problema: «Benito Juárez» como alcaldía hacía saltar el escáner, como si fuera el nombre de una persona. Ya no pasa: el municipio solo acepta un nombre de lugar.
>
> Lo probaste con el guion completo: todo bien.
>
> **Antes (2026-10-15, noche): la prueba completa del núcleo, con una institución de prueba** ([16 · La institución de prueba](16-institucion-de-prueba.md)): «Casa San Ejemplo», inventada, con **24 archivos** de su Expediente como los subiría una institución. Hay PDF con texto, escaneos (uno con la hoja de lado y otro desteñido), fotos de celular, una captura con letra chica, Word con y sin hoja escaneada, una página mixta, y archivos con fallas: con contraseña, dañado, Excel y .odt que no se usan, uno en el lugar equivocado y una foto sin texto. La guía dice qué es la institución, la ruta de la prueba, qué debe pasar con cada archivo, qué debe quedar en «Mi institución», 13 preguntas al Asistente con su respuesta y una tabla para calificar (98 puntos).
>
> Probada en seco, sin IA: los 24 archivos hacen lo que deben. Al armarla salieron dos problemas:
> - **corregido:** el número de una escritura escrito con palabras y partido en dos renglones se leía incompleto («47800» por «47820»);
> - **pendiente conocido:** una copia desteñida se lee con errores (el folio de la Junta pierde su «/») y no se marca «con dudas».
>
> **Siguiente:** que hagas la prueba completa siguiendo la guía; después, la fase 5.
>
> **Antes (2026-10-15, noche): C4, fase 4: el guion de pruebas del núcleo está listo** ([15 · Guion de pruebas del núcleo](15-guion-pruebas-nucleo.md)). Es la lista de lo que tienes que probar, pantalla por pantalla, con lo que debe pasar en cada paso y una casilla para marcarlo: primer inicio y cuentas, Inicio, Mi institución, el Expediente y Documentos, «Llenar con mis documentos», el Asistente, «Llenarlo por mí», y Ayuda automática, Seguridad y Ayuda. Junta en un solo lugar las pruebas que estaban sueltas en este mapa.
>
> Al revisar las pantallas para escribirlo encontré cuatro textos viejos y los corregí:
> - el manual de Seguridad hablaba de un PIN que ya no existe (desde las cuentas, se bloquea con «Bloquear» o solo a los 15 minutos);
> - la Ayuda decía «Olvidé mi PIN»; ahora es «Olvidé mi contraseña»;
> - el manual de «Llenarlo por mí» y la pantalla de Ayuda automática decían que el asistente nunca ve el teléfono ni el domicilio: desde el ADR-041 los copia si tú se los cuentas.
>
> **Antes (2026-10-15, noche): C4, fase 3 hecha: los tres agentes del núcleo están aprobados** ([ADR-036, aprobación](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)). Corrida final con el modelo más barato, dos rondas, nunca más de 14 llamadas por minuto:
>
> | Agente | Resultado | Inventado | Costo |
> |---|---|---|---|
> | Asistente | 32 de 32, citas 18 de 18 | 0 | $1.17 MXN |
> | Capturista («Llenarlo por mí») | 28 de 30 casos, 186 de 188 campos | 0 | $0.55 MXN |
> | «Llenar con mis documentos» | 35 de 36 campos, páginas 35 de 35 | 0 | $0.29 MXN |
>
> Lo que falló fue algo que no tomaron (el año dicho con palabras, la comida entre los apoyos, una edad), nunca un dato distinto. La primera corrida encontró que «vienen voluntarios de **una** parroquia» pasaba como 1 voluntario; ya se corrigió para los tres agentes.
>
> **Antes (2026-10-15, tarde): C4, fase 2 hecha: el Asistente y lo que encontraste al probar la fase 1** ([ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md), [ADR-040](adr/ADR-040-nombres-propios-de-la-institucion-en-su-expediente.md)):
> - «Tenemos 35 espacios» después de hablar del poder ya no se va al poder: la causa era una frase mía de C3, que se quitó. Ahora ofrece «Llenarlo por mí»;
> - «Tenemos 30 espacios… y viven 12 niñas» deja las dos cifras;
> - «con base en ese mismo» siempre le pregunta al Lector (lo hace el código), y «¿qué documentos tengo?» trae la lista con sus nombres, escrita por el programa;
> - el modelo a veces copia mal una letra («Reg1menes»): la cita ya no se pierde por eso, pero una cifra distinta nunca pasa;
> - los nombres de la institución en sus documentos sí llegan a la IA (tu decisión, ADR-040).
>
> Con el modelo más barato: Asistente 32 de 32, Capturista 28 de 28, «Llenar con mis documentos» 18 de 18; nada inventado.
>
> **Después de tu prueba de la fase 2:**
> - cerrar la ventana que «Llenarlo por mí» estaba llenando apaga el modo solo;
> - «Llenarlo por mí» ahora sugiere también el domicilio, el teléfono, el correo, el RFC y los folios que le cuentas, copiados letra por letra de tus palabras (tu decisión, [ADR-041](adr/ADR-041-el-capturista-copia-los-datos-propios-de-la-institucion.md)). Un dígito cambiado se tira. Si al escribir un teléfono o un RFC el programa pregunta si es dato personal, contesta «No es dato personal».
>
> Medido con la llave nueva: Capturista v4, 29 de 30 casos bien, nada inventado, $0.56 MXN las dos rondas. El domicilio completo, el teléfono y el RFC se sugieren copiados.
>
> Lo probaste y funciona.
>
> **Antes (2026-10-15, mañana): C4 empezó. Fase 1 hecha: el Capturista tiene su batería** ([ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)):
> - 12 casos de lo que una persona cuenta en el chat, con trampas: sumas, edades supuestas, cifras con palabras, datos protegidos;
> - el código ahora revisa que una cifra esté en tus palabras: «20 camas arriba y 15 abajo» ya no se vuelve «35»;
> - lo que dice el Capturista después de llenar ya lo escribe el programa: qué dejó, qué no tomó y qué falta contar. Antes la IA podía prometer algo que el código había tirado.
>
> Con el modelo más barato: 68 de 69 casos bien, nada inventado, unos $0.02 MXN por caso.
>
>
> **Antes (2026-10-14): arreglos tras tu prueba de C3** ([ADR-039](adr/ADR-039-mapa-de-lugares-desde-el-manual.md)):
> - el botón «Ir a…» ya no se escribe a mano: sale de un mapa armado con el manual y los formularios. Una página nueva solo necesita su entrada en el manual;
> - el respaldo desde «Mi institución» lleva a **Seguridad**, y nunca se ofrece un botón a la pantalla donde ya estás;
> - «con base en ese mismo» ahora le pregunta al Lector por el documento de la respuesta anterior, en vez de contestar de memoria (1 de 2 rondas; la otra ofrece «Llenar con mis documentos»).
>
> Batería de 14 preguntas con el modelo más barato, dos rondas: 25 de 28, nada inventado, $1.14 MXN.
>
> **Antes (2026-10-13): C3 está hecho: el Asistente le pregunta al Lector del Expediente** ([ADR-036, precisiones C3](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md)):
> - el Asistente ya no lee documentos: le pregunta al Lector, que contesta con la frase exacta, y el programa muestra el documento y la página;
> - cada pregunta lleva el mapa del Expediente: qué documentos hay y cómo están (nunca qué dicen);
> - la IA ve el tipo del documento, nunca el nombre del archivo (tu decisión), también en C2;
> - «Llenar con mis documentos» también aparece bajo una respuesta del Asistente, como botón.
>
> Con el modelo más barato: **26 de 26 preguntas bien en dos rondas**, nada inventado, unos $0.04 MXN por pregunta. Nuestro protocolo JSON le ganó a las funciones nativas de Gemini (26 contra 25 bien, 35 % más barato): se queda.
>
> **Siguiente:** que lo pruebes (pasos abajo); después, **C4**: la revisión completa del núcleo.

## Qué es SociAI, en una línea

Un programa de escritorio, local y con IA, para que asilos y casas hogar ordenen sus datos y armen proyectos para pedir donativos. Lo usan directivas y religiosas sin perfil técnico. Ver [Visión](00-vision.md).

## Qué sigue

|     | Bloque                | Qué hace                                                                                                                                                                                                                                                                       | Estado                |
| --- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------- |
| 1   | **C1**                | Cada documento va a un solo cajón; Documentos es la bandeja que pregunta a qué cajón va (por ahora solo el Expediente); la búsqueda siempre dentro de un cajón                                                                                                                 | **Hecho, por probar** |
| —   | **Lectura y llenado** | ADR-035, 037 y 038: una sola política de llenado, datos propios solos, Pdfium para texto y tablas, OCR (letra chica, hojas giradas, páginas mixtas, imágenes en Word y Excel, prioridad baja), lectura en segundo plano, el Expediente tapa y no rechaza, avisos de contraseña | **Hecho, por probar** |
| 2   | **C2**                | El Lector del Expediente: llena todo «Mi institución» buscando en el Expediente campo por campo, con la cita de cada dato; botón «Llenar con mis documentos»                                                                                                                   | **Hecho, por probar** |
| 3   | **C3**                | El Asistente pregunta al Lector del Expediente y sabe qué documentos existen («según mi registro, ¿qué nombre legal pongo?»)                                                                                                                                                   | **Hecho y probado**   |
| 4   | **C4**                | Revisión completa del núcleo: cada agente aprobado con el modelo más barato y cada pantalla probada por ti. Fases 1 a 4 hechas y tu prueba completa hecha; de ella salió el ADR-042 (un solo camino para llenar la institución, fases 1 y 2 hechas); después, la metodología de agentes | **En curso**          |
| 5   | **N5**                | El expediente hacia Proyectos, la IA e Inicio                                                                                                                                                                                                                                  | Pendiente             |
| —   | Módulos               | Finanzas (facturas), Personal y Beneficiarios (Lector solo con reglas), Instalaciones, Donantes: cada uno con su cajón, su Lector y su batería, siguiendo la [metodología de los Lectores](agents/metodologia-lectores.md)                                                     | Esperan al núcleo     |

Detalle de cada bloque en el [Roadmap](09-roadmap.md), «Fase 10».

## Lo que espera tu respuesta

- **Probar el archivo en su lugar** (ADR-042, fase 2): [guion, sección 5](15-guion-pruebas-nucleo.md), las dos casillas sin marcar.
- **Dos decisiones que salieron de la prueba completa** (sección 7 de la guía):
  - cómo detectar una copia desteñida que el lector de imágenes da por buena;
  - si la constancia del SAT debe sugerir también el domicilio de «Contacto y ubicación» (hoy solo el código postal fiscal).
- **Llave de pago antes de documentos reales** (ADR-040): con la llave gratuita, Google puede usar lo que se le manda.
- Borrar el archivo suelto `E:\Git\search_live3.txt` (lo dejó un error mío; el sistema no me deja borrarlo).
- Confirmar el precio de `gemini-embedding-2` en la página oficial de Google (anotado 0.20 USD por millón de tokens, tomado de sitios que comparan precios).

## Cómo probar

Todo lo que hay que probar del núcleo está en el [15 · Guion de pruebas del núcleo](15-guion-pruebas-nucleo.md): una sección por pantalla, con una casilla por paso. Las pruebas que antes estaban aquí (C1, la lectura y el llenado, C2 y C3) quedaron dentro.

## Lo último que se hizo

| Fecha | Qué | Dónde leerlo |
|---|---|---|
| 2026-10-16 | **ADR-042, fase 2 · un archivo solo se guarda en su lugar si el código confía en que es de ahí:** la confianza es una cuenta del código (palabras del documento contra las de otro, más el nombre del archivo); si no confía, pregunta antes de guardar nada. Umbral medido en seco: 563 lugares equivocados, ninguno guardado. Corregidas las reglas de la licencia de uso de suelo y del registro desteñido | [ADR-042](adr/ADR-042-primer-inicio-solo-lo-indispensable.md), [[Observaciones Pruebas]] |
| 2026-10-16 | **ADR-042, fase 1 · el primer inicio pide solo lo indispensable:** 14 datos del catálogo en cinco pasos, cada paso por su ventana, se cierra solo con el último dato, una sola cuenta del avance (sin fichas de módulos); el municipio con regla de nombre de lugar, sin escáner. Salió de tu prueba completa (33 de 39 datos bien) | [ADR-042](adr/ADR-042-primer-inicio-solo-lo-indispensable.md), [[Observaciones Pruebas]] |
| 2026-10-15 | **La prueba completa del núcleo:** la institución de prueba «Casa San Ejemplo», 24 archivos del Expediente (escaneos, fotos, Word y fallas a propósito), lo que debe quedar y cómo calificar; la prueba en seco `full_expediente`. Corregido: el número de escritura partido en dos renglones | [16 · La institución de prueba](16-institucion-de-prueba.md) |
| 2026-10-15 | **C4, fase 4 · guion de pruebas del núcleo:** una sección por pantalla con lo que debe pasar en cada paso; junta las pruebas sueltas del mapa. Se corrigieron cuatro textos viejos (el PIN de Seguridad y de la Ayuda, y lo que el asistente ve según el ADR-041) | [15 · Guion](15-guion-pruebas-nucleo.md) |
| 2026-10-15 | **C4, fase 3 · corrida final:** Asistente 32 de 32, Capturista 28 de 30, «Llenar con mis documentos» 35 de 36, nada inventado; los tres aprobados. «Una parroquia» ya no pasa como la cifra 1. Las baterías nunca pasan de 14 llamadas por minuto | [ADR-036, aprobación](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-15 | **Tras probar la fase 2:** cerrar la ventana apaga «Llenarlo por mí»; el Capturista sugiere los datos propios de la institución copiados letra por letra (ADR-041); lo que la persona aclara ante el escáner ya no se vuelve a tapar en ese encargo. Capturista v4: 29 de 30, nada inventado | [ADR-041](adr/ADR-041-el-capturista-copia-los-datos-propios-de-la-institucion.md) |
| 2026-10-15 | **C4, fase 2 · el Asistente y la prueba de la fase 1:** una sola revisión de citas que perdona un desliz de copia y nunca una cifra; «ese mismo» va al Lector por el código; la lista de documentos la escribe el programa; el bucle corre la herramienta antes que una respuesta prematura; Asistente v9 y Capturista v3. 32 de 32, 28 de 28 y 18 de 18, nada inventado. ADR-040: los nombres de la institución en su Expediente sí llegan a la IA | [ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md), [ADR-040](adr/ADR-040-nombres-propios-de-la-institucion-en-su-expediente.md) |
| 2026-10-15 | **C4, fase 1 · batería del Capturista:** 12 casos con trampas en las cuatro ventanas; el código revisa que cada cifra esté en las palabras de la persona; la IA ya no escribe la nota, el programa dice qué dejó, qué no tomó y qué falta contar. 68 de 69 casos bien, nada inventado | [ADR-036, precisiones C4](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-14 | **El mapa de lugares y «ese mismo»:** los botones «Ir a…» salen de un mapa armado con el manual y los formularios (sin listas a mano); el código elige el botón y nunca ofrece la pantalla donde ya estás; el respaldo lleva a Seguridad; la pregunta que sigue a un documento va al Lector con la clave de ese documento; el mapa del Expediente trae sus cifras para que la IA no cuente. Batería de 14: 25 de 28 en dos rondas, nada inventado | [ADR-039](adr/ADR-039-mapa-de-lugares-desde-el-manual.md) |
| 2026-10-13 | **C3 · el Asistente como orquestador:** le pregunta al Lector del Expediente (`record_reader`), que contesta con su cita revisada por el código; el mapa del Expediente en cada pregunta; la IA ve el tipo del documento, nunca el nombre del archivo; «Llenar con mis documentos» bajo una respuesta. Batería de 13 preguntas: 26 de 26 en dos rondas con el modelo más barato, $0.04 MXN por pregunta. Se queda el protocolo JSON (las funciones nativas de Gemini: 25 de 26 y 52 % más caras) | [ADR-036, precisiones C3](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **C2 · «Llenar con mis documentos»:** el Lector busca en el Expediente los pasajes de cada campo vacío y hace una llamada por ventana con el modelo más barato; el código revisa cita, página, cifra y valor. 18 de 18 campos bien, nada inventado, $0.14 MXN las cuatro ventanas. Las opciones van a la IA con su nombre en la pantalla | [ADR-036, precisiones C2](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **Lectura y llenado sin huecos (ADR-038):** una sola política de llenado (datos propios solos, sugerencias solo en campos vacíos, nunca lo rechazado, autoridad por campo, lugar equivocado); Pdfium lee también las tablas (salen `pdf-extract` y `lopdf`); OCR con prioridad baja, hojas giradas, páginas mixtas e imágenes en Word y Excel; el Expediente lee en segundo plano; la metodología para los módulos | [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md), [metodología](agents/metodologia-lectores.md) |
| 2026-10-13 | **Pdfium lee el texto de todo PDF:** palabras enteras (antes «Thi s report»), párrafos por la distancia entre renglones, guion de corte; de 5 a 20 veces más rápido. Un PDF, Word o Excel con contraseña lo dice en el Expediente y en las convocatorias | [ADR-037, bloque 2](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **Pdfium contra `pdf-extract` (medición):** en 24 PDF de muestra sacan las mismas letras, pero `pdf-extract` parte palabras («Thi s report outli nes») y es de 5 a 20 veces más lento; se decide Pdfium como único motor | [ADR-037](adr/ADR-037-pdfium-unico-motor-de-pdf.md) |
| 2026-10-13 | **El Expediente tapa, no rechaza:** ningún archivo del Expediente se rechaza por «lista de personas» ni pide «Tapar y seguir»; el escáner tapa todo, la pantalla dice cuántos datos y la bitácora guarda los conteos. Las convocatorias conservan el rechazo | [ADR-036, precisión del 2026-10-13](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-13 | **OCR · letra chica:** una imagen con letra de menos de 20 píxeles se agranda hasta que mida 30 (a lo más ×4) y se lee otra vez; una imagen chica sin ninguna palabra, ×3. Una captura ficticia pasa de 4 errores y sin título a leerse completa con su correo; una foto chica, de cero palabras a completa | [ADR-035, precisión 1](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-13 | **OCR con documentos reales (solo medición):** agrandar la letra chica lleva una captura de 13 palabras mal leídas a 2; dos actas reales se rechazan como «lista de personas» por dos reglas toscas del escáner; 37 páginas tardan 171 s en un equipo de 4 núcleos; no se entrena Tesseract ni se usa la nube | [ADR-035, precisiones 2026-10-13](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-12 | **C1:** cada documento lleva su cajón; Documentos es la bandeja (pregunta qué documento del Expediente es y abre su ventana); la búsqueda siempre en un cajón; quitar o reemplazar un archivo saca su texto del cajón; «Borrar esto y todo…» ya no falla con un dato aceptado | [ADR-036, precisiones C1](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **ADR-036:** cajones con dueño, un agente por cajón, el Asistente como orquestador, cada agente validado con el modelo más barato; primero el núcleo | [ADR-036](adr/ADR-036-cajones-de-documentos-y-un-agente-por-cajon.md) |
| 2026-10-12 | **IA6a con documentos reales:** el Lector entiende un acta real de 14 páginas y la ficha de la Junta (nombre, figura, año, objeto, número de institución); el objeto nunca incluye nombres del patronato | [Roadmap, IA6a](09-roadmap.md) |
| 2026-10-12 | **IA6a:** la clase y el tipo de cada documento; sus datos llegan como sugerencias con documento y página; un dato protegido solo por reglas | [ADR-034, precisiones IA6a](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-12 | **IA5:** búsqueda dentro de los documentos por palabras y por significado (vectores); medido antes de activarla: de 50 % a 100 % de aciertos con preguntas dichas con otras palabras | [ADR-034, precisiones IA5](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |
| 2026-10-10 | **OCR:** leer páginas escaneadas y fotos | [ADR-035](adr/ADR-035-leer-paginas-escaneadas-y-fotos-ocr.md) |
| 2026-10-10 | **N4:** el expediente de la institución con vigencia | [ADR-033 §5](adr/ADR-033-nucleo-expediente-y-formularios-universales.md) |
| 2026-10-10 | **IA3–IA4:** el Asistente en toda la app y el Capturista («Llenarlo por mí») | [ADR-034](adr/ADR-034-ia-en-todo-el-erp-agentes-herramientas-y-conocimiento.md) |

## Cómo está armado, en un dibujo

```
            Documentos  (la bandeja: aquí se sube y se ve todo)      ← ADR-036
                 │  «¿A qué cajón va?»
   ┌─────────────┼──────────────┬──────────────┬──────────────┐
Expediente     Personal       Finanzas       Proyectos     Beneficiarios,
(núcleo)       (módulo)       (módulo)       (convocatorias) Instalaciones…
 └ su Lector    └ solo reglas  └ su Lector    └ lectura canónica
 └ llena «Mi institución»

Asistente: la puerta de entrada en toda la app; pregunta al agente del cajón que corresponde.
Capas (ADR-032): base → módulos → núcleo → proyectos → pantallas; cada capa solo mira hacia abajo.
```

## Dónde está cada cosa

| Para saber… | Lee |
|---|---|
| Qué es y para quién | [Visión](00-vision.md) |
| Cómo está construido (capas, núcleo y módulos) | [Arquitectura](01-arquitectura.md) · [ADR-032](adr/ADR-032-monolito-modular-nucleo-y-modulos.md) |
| Qué datos nunca salen y cómo se cuidan | [Gobernanza de datos](03-gobernanza-datos.md) · [Escáner](04-escaner-datos-sensibles.md) |
| Qué tablas hay y de quién es cada una | [Modelo de datos](05-modelo-datos.md) |
| Cómo usamos la IA y cuánto cuesta | [IA y costos](07-ia-y-costos.md) |
| Cómo se escriben los textos de la pantalla | [Estilo de redacción](08-estilo-redaccion.md) |
| Qué se ha hecho y qué falta, bloque por bloque | [Roadmap](09-roadmap.md) |
| Cómo se ve el programa | [Sistema visual](13-sistema-visual.md) |
| Qué dice la ayuda del programa | [Manual](manual/programa.md) · [Mi institución](manual/mi-institucion.md) |
| Qué probar del núcleo, pantalla por pantalla | [Guion de pruebas del núcleo](15-guion-pruebas-nucleo.md) |
| La prueba completa: la institución de prueba, sus 24 archivos y lo que debe quedar | [16 · La institución de prueba](16-institucion-de-prueba.md) |
| Las reglas para trabajar en el código | [Principios](agents/principios-de-ingenieria.md) · [Flujo de trabajo](agents/flujo-de-trabajo.md) · [Comandos y pruebas](agents/comandos-y-pruebas.md) |
| Cómo un cajón lee sus archivos y llena sus formularios (para cada módulo) | [Metodología de los Lectores](agents/metodologia-lectores.md) · [ADR-038](adr/ADR-038-politica-unica-de-llenado-desde-documentos.md) |
| Por qué se decidió cada cosa | Los ADR, abajo · [Índice de ADR](adr/README.md) |

## Tus observaciones

Cómo dejarlas: [Notas y observaciones](notas/README.md). Una nota por observación, con la plantilla `plantillas/observacion`.

Abiertas (necesita el complemento Dataview):

```dataview
TABLE WITHOUT ID file.link AS "Nota", sobre AS "Sobre", fecha AS "Fecha"
FROM "notas"
WHERE estado = "abierta"
SORT fecha DESC
```

Observaciones dentro de los ADR: búsquelas con la búsqueda de Obsidian (Ctrl + Shift + F) escribiendo `"[!nota]" path:adr`.

## Las decisiones (ADR)

Las vigentes, de la más nueva a la más vieja:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha", tema AS "Tema", reemplaza_en_parte AS "Reemplaza en parte"
FROM "adr"
WHERE adr AND estado != "reemplazada"
SORT adr DESC
```

Las que ya no rigen, o que otra cambió en parte:

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", reemplazado_por AS "La reemplaza", reemplazado_en_parte_por AS "La cambia en parte"
FROM "adr"
WHERE adr AND (estado = "reemplazada" OR reemplazado_en_parte_por)
SORT adr DESC
```

Por tema (para ver solo lo de IA, del núcleo o de documentos):

```dataview
TABLE WITHOUT ID file.link AS "ADR", estado AS "Estado", fecha AS "Fecha"
FROM "adr"
WHERE contains(tema, "ia")
SORT adr DESC
```

Cambie `"ia"` por `"nucleo"`, `"documentos"`, `"proyectos"`, `"modulos"`, `"seguridad"`, `"interfaz"`, `"arquitectura"`, `"infraestructura"` o `"datos"`.
