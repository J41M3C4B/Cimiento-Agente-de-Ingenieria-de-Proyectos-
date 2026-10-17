---
tipo: guia
fecha: 2026-10-15
sobre: "[[15-guion-pruebas-nucleo]]"
---

# 16 · La institución de prueba: Casa San Ejemplo

La prueba completa del núcleo con una institución inventada, de principio a fin: se crea desde una carpeta vacía, se suben sus 24 archivos, se llena «Mi institución» y se le pregunta al Asistente. Aquí está **todo lo que es la institución** y **lo que debe quedar al final**, para compararlo con lo que hizo el programa y calificar el núcleo.

Todo es ficticio: los nombres, el RFC, la CLUNI, la CLABE y los folios son inventados. Con estos archivos puedes usar la llave gratuita.

**Dónde están los archivos:** `fixtures/expediente/prueba-completa/archivos/` (24 archivos). Se rehacen con `python generar.py` en esa carpeta. La prueba en seco de esos mismos archivos, sin IA, es `cargo test full_expediente -- --ignored --nocapture` (`esperado.json`).

## 1. La institución

**Casa de Reposo Ficticia San Ejemplo, I.A.P.**, «Casa San Ejemplo». Casa de reposo en la Ciudad de México, fundada en 2015, para personas mayores de 60 años en abandono o de escasos recursos. Tiene residencia permanente y, desde 2019, estancia de día.

### Su institución

| Dato                                      | Lo que debe quedar                                                                                                                                         | De dónde sale                                                 | Resultado                                                                                                                                                                                              |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Nombre de la institución                  | Casa San Ejemplo                                                                                                                                           | Lo dices tú (paso B)                                          | Si                                                                                                                                                                                                     |
| ¿A qué se dedica?                         | Lo que digas en el paso B («una casa de reposo para personas mayores»). Si lo dejas vacío, «Llenar con mis documentos» sugiere: dar un hogar, cuidado y compañía a personas mayores de 60 años en situación de abandono o de escasos recursos, con residencia permanente y estancia de día | Lo dices tú (paso B); si no, «Llenar con mis documentos» (acta, informe). Un campo lleno no recibe sugerencias (ADR-038) | No, dio Casa de reposo para personas mayores                                                                                                                                                           |
| ¿A quién atienden?                        | Personas mayores (60 años o más)                                                                                                                           | Lo dices tú (paso B)                                          | Si                                                                                                                                                                                                     |
| ¿Atienden a mujeres, a hombres o a ambos? | Mujeres y hombres                                                                                                                                          | «Llenar con mis documentos» (reglamento: «de cualquier sexo») | Si                                                                                                                                                                                                     |
| ¿Cómo los atienden?                       | Viven en la institución; Estancia de día                                                                                                                   | Lo dices tú (paso B)                                          | Si                                                                                                                                                                                                     |
| ¿En qué los apoyan?                       | Cuidado, Salud, Alimentación                                                                                                                               | «Llenar con mis documentos» (acta, informe)                   | Si                                                                                                                                                                                                     |
| ¿Qué servicios o programas ofrecen?       | Residencia permanente, estancia de día, terapia física y ocupacional, talleres, atención médica y pláticas para las familias                               | «Llenar con mis documentos» (informe)                         | brindar alojamiento, alimentación y cuidados                                                                                                                                                           |
| Edad mínima                               | 60                                                                                                                                                         | «Llenar con mis documentos» (reglamento, acta)                | Si                                                                                                                                                                                                     |
| Edad máxima                               | (vacía: no hay)                                                                                                                                            | Ningún documento la dice: **no debe sugerirse nada**          | Si                                                                                                                                                                                                     |
| ¿Qué piden para que alguien ingrese?      | Solicitud firmada, valoración médica y geriátrica, estudio socioeconómico y carta compromiso del familiar                                                  | «Llenar con mis documentos» (reglamento, art. 2)              | solicitud firmada por la persona o por su familiar responsable; valoración médica y geriátrica realizada por el equipo de la Casa; estudio socioeconómico; y carta compromiso del familiar responsable |

### Contacto y ubicación

| Dato                 | Lo que debe quedar             | De dónde sale                                                | Resultado                      |
| -------------------- | ------------------------------ | ------------------------------------------------------------ | ------------------------------ |
| Teléfono             | 55 5123 4567                   | «Llenarlo por mí» (paso F)                                   | 55 5123 4567                   |
| Correo               | contacto@casasanejemplo.org.mx | «Llenarlo por mí» (paso F)                                   | contacto@casasanejemplo.org.mx |
| Calle                | de los Ejemplos                | «Llenarlo por mí»                                            | Calle de los Ejemplos          |
| Número exterior      | 128                            | «Llenarlo por mí»                                            | 128                            |
| Número interior      | (vacío)                        | —                                                            |                                |
| Colonia              | Del Valle Centro               | «Llenarlo por mí»                                            | Del Valle Centro               |
| Código postal        | 03100                          | «Llenarlo por mí»                                            | 03100                          |
| Municipio o alcaldía | Benito Juárez                  | «Llenarlo por mí» o «Llenar con mis documentos» (constancia) | BENITO JUAREZ                  |
| Estado               | Ciudad de México               | «Llenarlo por mí» o «Llenar con mis documentos»              | Si                             |

> [!warning] Trampa a propósito
> El registro ante la Junta (2015) trae un domicilio viejo: «Calle de los Robles número 45, colonia Centro». El domicilio de hoy es el de la constancia del SAT y el aviso de privacidad.

### Datos legales y fiscales

| Dato                         | Lo que debe quedar                                                                              | De dónde sale                                                                                                         | Resultado                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ---------------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Nombre legal                 | Casa de Reposo Ficticia San Ejemplo (la constancia lo sugiere en mayúsculas; puedes corregirlo) | Sugerencia al subir la constancia                                                                                     | CASA DE REPOSO FICTICIA SAN EJEMPLO                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Objeto social                | El artículo cuarto del acta (I a IV)                                                            | Sugerencia al subir el acta                                                                                           | I. Brindar alojamiento, alimentación, vestido y atención médica a personas adultas mayores de sesenta años en situación de abandono o de escasos recursos. II. Proporcionar terapia física, ocupacional y actividades recreativas que mejoren la calidad de vida de las personas residentes. III. Ofrecer orientación a las familias sobre el cuidado de la persona mayor. IV. Recibir donativos en dinero o en especie para el cumplimiento de su objeto. |
| Figura jurídica              | I.A.P.                                                                                          | Lo dices tú (paso B)                                                                                                  | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Año de fundación             | 2015                                                                                            | Sugerencia al subir el acta                                                                                           | 2015                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Folio ante la Junta          | JAP-1234/2015                                                                                   | Sugerencia al subir el registro. **Pendiente conocido:** de la copia desteñida sale «JAP-12342015». Corrígelo a mano. | JAP-12342015                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| RFC                          | CRF150310KX4                                                                                    | Sugerencia al subir la constancia                                                                                     | CRF150310KX4                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Régimen fiscal               | Personas morales con fines no lucrativos                                                        | Sugerencia al subir la constancia                                                                                     | si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Código postal fiscal         | 03100                                                                                           | Sugerencia al subir la constancia                                                                                     | 03100                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ¿Es donataria autorizada?    | Sí                                                                                              | Lo dices tú (paso B)                                                                                                  | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ¿Tiene CLUNI?                | Sí                                                                                              | Lo dices tú (paso B)                                                                                                  | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Rubro autorizado             | Asistencial                                                                                     | Sugerencia al subir el oficio                                                                                         | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Número del oficio            | 600-04-02-2017-11234                                                                            | Sugerencia al subir el oficio                                                                                         | 600-04-02-2017-11234                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Fecha del oficio             | 14 de marzo de 2017                                                                             | Sugerencia al subir el oficio                                                                                         | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Clave CLUNI                  | CRF15031009015                                                                                  | Sugerencia al subir la captura de la CLUNI                                                                            | CRF15031009015                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Representante legal          | Rosa Ficticia del Ejemplo                                                                       | «Llenarlo por mí» (paso F); el poder no dice su nombre, solo «la Directora General»                                   | Tuvimos el problema 8, descrito en notas/Observaciones Pruebas                                                                                                                                                                                                                                                                                                                                                                                             |
| ¿Hasta cuándo vale su poder? | 18 de enero de 2027                                                                             | Sugerencia al subir el poder                                                                                          | Si                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

### Capacidad y cifras rápidas

| Dato                              | Lo que debe quedar | De dónde sale                                    | Resultados |
| --------------------------------- | ------------------ | ------------------------------------------------ | ---------- |
| Capacidad total                   | 75                 | «Llenarlo por mí» (paso F)                       | Si         |
| ¿Cuántas atienden hoy?            | 70                 | «Llenarlo por mí» (paso F)                       | Si         |
| ¿Cuántas trabajan con sueldo?     | 31                 | «Llenar con mis documentos» (informe 2025, p. 1) | Si         |
| ¿Cuántas ayudan como voluntarias? | 18                 | «Llenar con mis documentos» (informe 2025, p. 1) | Si         |
| Notas generales                   | (libre)            | —                                                | --         |

### Patronato y cuentas (a mano, paso G)

| Persona                  | Cargo       | Desde                   |
| ------------------------ | ----------- | ----------------------- |
| Ernesto Ficticio Ramírez | Presidencia | 12 de diciembre de 2023 |
| Mónica Ficticia López    | Secretaría  | 5 de julio de 2019      |
| Jorge Ejemplo Sánchez    | Tesorería   | 12 de diciembre de 2023 |
| Laura Ficticia Méndez    | Vocal       | 12 de diciembre de 2023 |
| Pedro Ejemplo Ruiz       | Vocal       | 12 de diciembre de 2023 |

**Cuenta bancaria:** CLABE `012180000123456782` (ficticia, con su número de control bien), para donativos, «Sí» se pone en los documentos. El banco lo toma el programa de la CLABE.

### Dinero e inmueble (en sus módulos, después del núcleo)

El primer inicio ya no los pide (ADR-042): son de Finanzas e Instalaciones. Para cuando se prueben esos módulos:

- **Dinero** (del estado de actividades 2025): gasto del año 4,755,000. Ingresos al año: donativos de personas 1,240,000 (Donativo ocasional), de empresas y fundaciones 2,310,000 (Donante fijo), en especie 415,000 (Otro ingreso), cuotas de recuperación 980,000 (Cuota aproximada), eventos 260,000 (Otro ingreso).
- **Su casa:** 2 pisos, en comodato hasta 2036 (contrato desde el 1 de abril de 2016, por 20 años), con papeles.

## 2. La ruta de la prueba

Hazla en una carpeta vacía, para no mezclarla con tus datos:
`$env:CIMIENTO_DATA_DIR="D:\Agente Proyectos\cimiento-prueba"; pnpm tauri dev`

**A · Empezar**
1. Crea la cuenta de administración, guarda el código y pasa la bienvenida.
2. En «Puesta en marcha», pon la llave y elige **«Lo hará la dirección»**: entras con «Mi institución» vacía.

**B · Lo primero que dice la persona** («Llenarlo por mí»)
3. En «Su institución», escríbele: *«Somos Casa San Ejemplo, una casa de reposo para personas mayores. Viven aquí y también tenemos estancia de día.»* Usa las sugerencias y guarda. Deben quedar el nombre, a quién atienden y cómo.
4. En «Datos legales y fiscales»: *«Somos una I.A.P., donataria autorizada y tenemos CLUNI.»* Usa y guarda.
5. El Expediente debe mostrar **18 documentos** (sección 3). Antes del paso 4 no aparecen los de donataria, CLUNI y Junta.

**C · Subir los archivos** en el orden de la sección 3, cada uno en su documento del Expediente («Subir archivo»). Revisa lo que dice cada aviso.

**D · Las sugerencias de los documentos.** Abre «Datos legales y fiscales»: deben esperar las 12 sugerencias de la sección 4, cada una con su documento y página. Corrige el folio de la Junta, usa las demás y guarda.

**E · «Llenar todo con mis documentos»** (en el Expediente). Revisa cada ventana contra las tablas de la sección 1: lo marcado «Llenar con mis documentos» en «De dónde sale» debe llegar como sugerencia con su documento y página, y la edad máxima y la capacidad no deben sugerirse. Si sugiere 48 en «¿Cuántas atienden hoy?», está bien: son las residentes del informe; en el paso F lo cambias a 70.

**F · «Llenarlo por mí» para lo que ningún documento dice**
- Contacto: *«Estamos en Calle de los Ejemplos 128, colonia Del Valle Centro, alcaldía Benito Juárez, Ciudad de México, C.P. 03100. Teléfono 55 5123 4567 y correo contacto@casasanejemplo.org.mx.»* Cuando pregunte por los datos personales, contesta «Esto no son datos personales de nadie».
- Datos legales: *«Nos representa la directora, Rosa Ficticia del Ejemplo.»*
- Capacidad: *«En total podemos atender a 75 personas a la vez y hoy atendemos a 70: 48 viven aquí y 22 vienen de día.»* Debe dejar **75 y 70**. Si escribes solo «48 viven aquí y 22 vienen de día», no debe sumar.

**G · A mano:** el patronato y la cuenta de la sección 1. La vigencia de la licencia de uso de suelo: 15 de mayo de 2024 (vale un año desde su expedición), y debe salir **«Vencido»**.

**H · Terminar el primer inicio:** en Inicio, el aviso ya debe haber desaparecido: los 14 datos indispensables (ADR-042) los llenaste en las ventanas. Si sigue, «Llenarlos ahora» abre solo lo que falta de ellos; «Terminar y entrar».

**I · El Asistente:** las preguntas de la sección 5.

## 3. Los 24 archivos: dónde van y qué debe pasar

En este orden. «Llena solo» son los datos del propio documento (emisión, vigencia, número), con «Leído del archivo, página N».

| # | Archivo | Dónde se sube | Qué prueba | Debe pasar |
|---|---|---|---|---|
| 1 | `19-acta-constitutiva-version-final.pdf` | Acta constitutiva | En el lugar equivocado: es la constancia del SAT con nombre de acta | Se guarda y avisa «Este archivo parece «Constancia de situación fiscal», no este documento». No llena nada |
| 2 | `01-acta-constitutiva.pdf` | Acta constitutiva (cambiar archivo) | PDF con texto; reemplaza al anterior | Llena solo: emisión 10/03/2015, número 42110. Deja sugerencias (año, objeto social) |
| 3 | `02-reformas-estatutos-2019-escaneo.pdf` | Reformas a los estatutos | Escaneo de 2 páginas; trae la CURP de una persona | Leído de imagen. Avisa «Tapamos 1 dato personal». Llena solo: emisión 22/08/2019, número **47820** (el número se parte en dos renglones) |
| 4 | `03-poder-notarial-hoja-girada.pdf` | Poder del representante legal | Escaneo con la hoja 2 de lado | Leído de imagen. Llena solo: emisión 18/01/2024, vigencia 18/01/2027, número 51300. Sugiere la vigencia del poder |
| 5 | `04-constancia-situacion-fiscal.pdf` | Constancia de situación fiscal | PDF con texto | Llena solo: emisión 06/02/2026. Deja 4 sugerencias (RFC, nombre legal, régimen, código postal fiscal) |
| 6 | `05-opinion-cumplimiento-foto.jpg` | Opinión de cumplimiento (32-D) | Foto de celular, chueca y sobre una mesa | Leído de imagen. Llena solo: emisión 20/09/2026, folio 26NB1234567 y vigencia **20/10/2026**, «Calculado con la fecha de emisión del archivo». Sale **«Por vencer»** (o «Vencido» después del 20 de octubre) |
| 7 | `06-oficio-donataria-escaneo.pdf` | Autorización como donataria | Escaneo limpio | Llena solo: emisión 14/03/2017, número 600-04-02-2017-11234. Deja 3 sugerencias (número, fecha, rubro) |
| 8 | `07-cluni-captura.png` | Constancia de la CLUNI | Captura de pantalla con letra chica | Leído de imagen. Llena solo: emisión 11/05/2016, clave. Sugiere la clave CLUNI |
| 9 | `08-registro-junta-destenido.pdf` | Registro ante la Junta | Copia desteñida y sucia | Leído de imagen. Llena solo: emisión 02/06/2015 y el folio. **Pendiente conocido:** el folio sale «JAP-12342015» (sin la «/») y no avisa «con dudas» |
| 10 | `09-aviso-funcionamiento.docx` | Aviso de funcionamiento sanitario | Word | Llena solo: emisión 03/09/2025 |
| 11 | `10-proteccion-civil-con-contrasena.pdf` | Programa de protección civil | PDF con contraseña | No se guarda; avisa que tiene contraseña (nunca «dañado») |
| 12 | `10b-proteccion-civil.pdf` | Programa de protección civil | El mismo, sin contraseña | Se guarda. No llena nada (este documento no trae datos propios que el programa lea) |
| 13 | `11-aviso-privacidad.docx` | Aviso de privacidad | Word con el teléfono y el correo de la institución | Se guarda. Avisa «Tapamos 2 datos personales»: el teléfono y el correo, porque todavía no están guardados como de la institución |
| 14 | `12-reglamento-interno-con-hoja-escaneada.docx` | Reglamento interno | Word con la página 3 pegada como imagen | Leído de imagen (esa página) |
| 15 | `13-organigrama.png` | Organigrama | Dibujo con cuadros, sin nombres | Leído de imagen |
| 16 | `14b-estados-financieros-2025-danado.pdf` | Estados financieros | PDF cortado a la mitad | No se guarda; avisa que está dañado |
| 17 | `14-estados-financieros-2025.pdf` | Estados financieros | PDF con una tabla con bordes | Se guarda |
| 18 | `15b-informe-anual-2025.odt` | Informe anual | Tipo de archivo que no se usa | No se guarda; pide un PDF, un Word o una foto |
| 19 | `15-informe-anual-2025.pdf` | Informe anual | PDF con texto | Se guarda |
| 20 | `16-presupuesto-junta-2026.xlsx` | Presupuesto ante la Junta | Excel: el Expediente no lo toma | No se guarda; pide un PDF, un Word o una foto |
| 21 | `16b-presupuesto-junta-2026.pdf` | Presupuesto ante la Junta | PDF con tabla | Se guarda |
| 22 | `20-foto-sin-texto.jpg` | Licencia de uso de suelo | Foto tan borrosa que no se lee | Se guarda «pero no pudimos leer su texto… Cuenta como que lo tienen» |
| 23 | `17-licencia-uso-suelo-foto.jpg` | Licencia de uso de suelo (cambiar archivo) | Foto legible | Leído de imagen. Llena solo: emisión 15/05/2023 |
| 24 | `18-contrato-comodato-pagina-mixta.pdf` | «Agregar otro documento»: Contrato de comodato | Página 1 con encabezado impreso y el cuerpo escaneado | Leído de imagen (página 1) |

**Al final:** el Expediente dice **«Tiene 18 de 18»**, con la opinión de cumplimiento «Por vencer» y, cuando escribas su vigencia, la licencia de uso de suelo «Vencido».

## 4. Las sugerencias que dejan los documentos (paso D)

Deben esperar **12**, cada una con «Lo leímos en «archivo», página N»:

| Ventana | Dato | Valor | Documento |
|---|---|---|---|
| Datos legales | Año de fundación | 2015 | Acta, p. 1 |
| Datos legales | Objeto social | «I. Brindar alojamiento, alimentación, vestido y atención médica… IV. Recibir donativos…» | Acta, p. 2 |
| Datos legales | ¿Hasta cuándo vale su poder? | 18/01/2027 | Poder, p. 2 |
| Datos legales | RFC | CRF150310KX4 | Constancia, p. 1 |
| Datos legales | Nombre legal | CASA DE REPOSO FICTICIA SAN EJEMPLO | Constancia, p. 1 |
| Datos legales | Régimen fiscal | Personas morales con fines no lucrativos | Constancia, p. 2 |
| Datos legales | Código postal fiscal | 03100 | Constancia, p. 1 |
| Datos legales | Número del oficio | 600-04-02-2017-11234 | Oficio, p. 1 |
| Datos legales | Fecha del oficio | 14/03/2017 | Oficio, p. 1 |
| Datos legales | Rubro autorizado | Asistencial | Oficio, p. 1 |
| Datos legales | Clave CLUNI | CRF15031009015 | CLUNI, p. 1 |
| Datos legales | Folio ante la Junta | JAP-1234/2015 (hoy sale JAP-12342015: pendiente conocido) | Registro, p. 1 |

Ninguna para el nombre, a quién atienden ni la figura: ya los habías dicho tú. Y ninguna del domicilio: el domicilio no lo sugiere ningún documento todavía (sección 7).

## 5. Preguntas al Asistente (paso I)

| Pregunta | Respuesta esperada | Debe citar |
|---|---|---|
| ¿Qué documentos tengo? | La lista: los 18, con la opinión de cumplimiento por vencer y la licencia vencida | (la escribe el programa) |
| ¿Hasta cuándo vale el poder de la directora? | Hasta el 18 de enero de 2027 | Poder, p. 2 |
| ¿Cuántos extintores tenemos? | Doce, revisados cada año en agosto | Protección civil, p. 1 |
| ¿Cuál es el plazo del comodato? | 20 años desde el 1 de abril de 2016 | Contrato de comodato, p. 1 (la página escaneada) |
| ¿Cuántas caídas hubo el año pasado? | Cuatro, en los baños de la planta baja | Informe anual, p. 3 |
| ¿Cuánto vamos a gastar en 2026? | 5,480,000 pesos | Presupuesto ante la Junta, p. 1 |
| ¿De quién depende enfermería? | De la Dirección General | Organigrama |
| ¿Qué número de escritura tienen las reformas? | 47,820 | Reformas, p. 1 |
| ¿Cuál es nuestro domicilio? | Calle de los Ejemplos 128, Del Valle Centro. Si menciona el de la Junta (Robles 45), debe decir que es el registrado en 2015 | Constancia o aviso de privacidad |
| ¿Cuál es la CURP de la secretaria del patronato? | Que no la tiene: la CURP se tapó al subir las reformas | (nada) |
| ¿Cuántos residentes tuvimos en 2023? | Que ningún documento lo dice | (nada) |
| Según mi acta, ¿a qué nos dedicamos? Y luego: «Con base en ese mismo, ¿desde cuándo existimos?» | El objeto del acta; luego 2015, del mismo documento | Acta, p. 2 y p. 1 |

**No debe pasar:** una cifra que no esté en la página que cita, un documento que no habla del tema, la CURP de alguien o un dato inventado.

## 6. Cómo calificar

Marca cada renglón «Bien», «Mal» (dio otra cosa) o «Inventó» (dio algo que no está en ninguna parte). **El núcleo pasa con nada inventado y al menos 90 % bien.**

| Parte                                                                                                                                  | Cuántos | Bien | Mal | Inventó |
| -------------------------------------------------------------------------------------------------------------------------------------- | ------- | ---- | --- | ------- |
| Avisos de los 24 archivos (sección 3)                                                                                                  | 24      |      |     |         |
| Datos propios de los documentos (emisión, vigencia, número)                                                                            | 19      |      |     |         |
| Sugerencias de los documentos (sección 4)                                                                                              | 12      |      |     |         |
| «Llenar todo con mis documentos»: los 10 datos marcados así en la sección 1, más la edad máxima y la capacidad, que no deben sugerirse | 12      |      |     |         |
| «Llenarlo por mí»: 6 datos del paso B, 11 del paso F y que no sume                                                                     | 18      |      |     |         |
| Preguntas al Asistente (sección 5)                                                                                                     | 13      |      |     |         |
| **Total**                                                                                                                              | **98**  |      |     |         |

Anota cada «Mal» o «Inventó» en una nota de `notas/` con el paso, el archivo o la pregunta, y lo que salió.

## 7. Lo que ya se sabe

- **Pendiente conocido:** la copia desteñida del registro ante la Junta se lee con errores: el folio pierde su «/» y la página no se marca «con dudas». El aviso de «con dudas» se basa en qué tan seguro dice estar el lector de imágenes, y en esta copia dice estar seguro. Por decidir: cómo detectar una copia así.
- **Para decidir:** la constancia del SAT trae el domicilio completo (calle, colonia, código postal, municipio y estado), pero hoy solo sugiere el código postal fiscal. Podría sugerir también el domicilio de «Contacto y ubicación».
- **Corregido al armar esta prueba:** el número de una escritura escrito con palabras y partido en dos renglones se leía incompleto («47800» por «47820»).
- **En seco, sin IA,** los 24 archivos ya hacen lo de la sección 3 y dejan las 12 sugerencias de la sección 4 (`cargo test full_expediente -- --ignored --nocapture`). Lo que falta medir es lo que usa IA (pasos B, E, F e I) y lo que ve una persona en la pantalla.
