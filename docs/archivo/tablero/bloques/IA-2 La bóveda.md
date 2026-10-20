---
tipo: bloque
id: IA-2
carril: ia
estado: hecho
prioridad: alta
fase: MVP-2
patron: [determinista]
rama: ia/ia-2-boveda
adr: "[[ADR-044-la-boveda]]"
creado: 2026-10-18
cerrado: 2026-10-20
---

## Qué y por qué

Hoy el escáner detiene a la persona cada vez que cree ver un dato personal, y cada error pide una regla nueva. Con la bóveda, hacia la IA van marcas tipadas (`[CORREO_1]`, `[NOMBRE_1]`) y nunca el valor. La IA decide qué es cada marca por su contexto, puede preguntar qué se sabe de una marca sin recibir el valor, y el código la resuelve en el equipo según lo que acepta cada campo. Sin la ventana de «datos personales» en el chat.

**Decidido por el líder (2026-10-18):** lo de salud de una sola persona se sigue generalizando antes de salir, como hoy, aunque el nombre vaya como marca. Solo los grupos de 3 o más llevan atributos de salud. Se puede revisar más adelante.

**Patrón (ADR-045):** determinista. La bóveda no usa IA: marca, resuelve y escribe la leyenda con código. Es del carril `ia` porque decide lo que recibe la IA ([[roles-y-tablero|Roles y tablero]]); el escáner sigue siendo del carril `erp` y no cambia.

**La propuesta:** [[ADR-044-la-boveda|ADR-044]].

## Terminado cuando

(Aprobado por el líder el 2026-10-19, con la aclaración del ADR-044 §2.)

- [x] **Ningún valor marcado aparece en ninguna solicitud a la IA, en todas las baterías en seco:** Asistente, Capturista, los tres Lectores y la conversación de Proyectos. Una revisión en el proveedor simulado lo comprueba en cada solicitud. *(`MockProvider::personal_data_sent`: todos los casos de las baterías del Asistente y del Capturista (`no_personal_value`) y una prueba de cada Lector y de la conversación.)*
- [x] **El Asistente y el Capturista ya no se detienen por un dato personal:** el servicio ya no devuelve la ventana, y «No es dato personal» y `cleared` salieron del chat. *(El código de la pantalla y `ScannerConfig::cleared` quedan sin usar: ERP-4.)*
- [x] **Una marca se resuelve en cualquier campo, y el valor pasa por la validación del campo y por `proposals::propose`.** Pruebas: el correo y el teléfono en su campo; la representante y el RFC que cuenta la persona en «Datos legales»; «alcaldía Benito Juárez» en el municipio y «calle [NOMBRE_1] 12» en la calle, aunque el escáner los tome por persona; un nombre en la misión lo cubre `propose` como hoy; una sugerencia con una marca inventada se descarta. *(En seco, todas. En real, la calle 3 de 3; el municipio con la marca sola, 0 de 5: lo resuelve IA-8. Los nombres de lugar ya no se tapan en `propose`, como al guardar.)*
- [x] **La persona lee sus valores:** la respuesta llega a la pantalla con cada marca resuelta.
- [x] **La salud de una persona sale generalizada**, sin número, y nunca se resuelve.
- [x] **Las baterías con casos nuevos** (dos personas distintas, el correo y la representante de la institución, la alcaldía, una CURP en la pregunta, salud de una persona), primero en seco y después en una corrida real con el modelo más barato. Asistente y Capturista no empeoran contra el v11 y el v5 en aciertos, inventados ni costo. *(Asistente 46 de 50, las 20 de antes 39 de 40, $1.51 MXN; Capturista 39 de 44, campos 231 de 236, $1.17 MXN; nada inventado. Detalle en `07-ia-y-costos.md`.)*
- [x] El principio 5, `04-escaner-datos-sensibles.md`, `07-ia-y-costos.md` y la metodología de los patrones dicen cómo sale un dato personal: como marca.

## Plan

**Ficha (patrón 1, determinista):**

| | |
|---|---|
| **Regla** | Entran un texto y lo que halló el escáner. Sale el texto con marcas `[TIPO_n]`. El mismo valor lleva la misma marca en todo el encargo; se compara sin mayúsculas, acentos ni espacios, y un teléfono por sus 10 dígitos. Lo de salud lleva su etiqueta de hoy y nunca se resuelve. `reveal` cambia cada marca por su valor, también las mal escritas (`[NOMBRE 1]`, `NOMBRE_1`), y nunca inventa uno |
| **Dónde vive** | `ai/vault.rs`, en la base. Una bóveda por encargo, solo en memoria |
| **Pruebas** | Mismo valor, misma marca; dos personas, dos marcas; la salud sin número; marcas mal escritas; una marca que no existe no se resuelve; ningún valor llega al proveedor simulado; nada va al disco ni a la bitácora |
| **Uso por la IA** | No es herramienta. El pipeline marca todo lo que sale; el agente resuelve los argumentos de las herramientas; el Asistente resuelve su respuesta y el Capturista sus sugerencias |

**Pasos:**

1. `ai/vault.rs`: marcar y resolver, con sus pruebas (mismo valor, misma marca; la salud sin número; marcas mal escritas; nada en disco).
2. `ai::pipeline` y `ai::agent` con una bóveda por encargo; herramientas con sus argumentos resueltos y su resultado marcado.
3. En `core/agents`: la sugerencia resuelta antes de `proposals::propose` (en cualquier campo, con su validación), la leyenda y la respuesta resuelta para la pantalla. El Asistente y el Capturista sin la ventana.
4. Oficios v12 (Asistente) y v6 (Capturista), con los casos nuevos de las baterías en seco.
5. La revisión «ningún valor en ninguna solicitud» en el proveedor simulado, para todas las baterías.
6. Corrida real: Asistente y Capturista, dos rondas, a 14 por minuto, salida a archivo.
7. Documentos. Una nota al carril `erp`: el código de la ventana del Asistente queda sin usar, y con la bóveda puede medir si quita reglas por caso del escáner.

## Cómo probarlo

En la app de la rama `ia/ia-2-boveda` (carpeta `D:\Agente Proyectos\cimiento-ia`, `pnpm tauri dev`), con la llave de Gemini guardada:

- [x] En Inicio, en el Asistente: «¿Dónde anoto la CURP LOPM800101MDFRZN09 de una señora que atendemos?». **No aparece la ventana de «datos personales»**; contesta que esa parte llegará después.
- [ ] En «Mi institución», en el Asistente: «Nuestro correo es contacto@casaficticia.org, ¿dónde lo pongo?». Ofrece «Llenarlo por mí» en «Contacto y ubicación»; al usarlo, la sugerencia del correo trae **el correo tal cual lo escribió**.
- [ ] «Llenarlo por mí» en «Datos legales y fiscales»: «Nuestra representante legal es Rosa Méndez Ruiz y nuestro RFC es CSE870512AB3». Las sugerencias traen **el nombre y el RFC tal cual**, sin preguntar nada antes.
- [x] «Llenarlo por mí» en «Contacto y ubicación»: «Estamos en la Calle Ana María Rodríguez 8». La calle llega como «Ana María Rodríguez» (antes se tapaba como nombre).
- [x] Saber que no salió nada: en las métricas de la IA o en la bitácora, cada pregunta con datos deja `scanner.leak_prevented` con **cuántos** datos de cada tipo, nunca cuáles.

- [ ] **Segunda vuelta (2026-10-20), en una sola conversación y sin reiniciarla, desde «Mi institución»:** el correo, luego la representante y el RFC, luego «Estamos en la Calle Ana María Rodríguez 8». La calle se ofrece en **«Contacto y ubicación»**, no en «Datos legales y fiscales».

Conocido: «Estamos en Benito Juárez, Ciudad de México» en «Llenarlo por mí» no llena el municipio (el modelo más barato no pone la marca ahí). Lo resuelve IA-8.

## Resultado
La ventana de datos personales ya no frena, lo que si noto es que hay mas latencia o tiempo de espera para una respuesta y es demasiado, no se precisamente si sea por que el proveedor tiene mucha demanda, tenemos segun los logs de ayuda automatica 24s de media por respuesta, sin embargo con el involucramiento de un nuevo modelo para interceptar los datos y todo eso no se a cuanto sume el tiempo o si el contador de la IA tambien cuenta todo el proceso en general. 

En la prueba 3 volvimos a tener el mismo problema de 2 llamadas, cuando en una sola se pudo haber omitido la sugerencia de llenarlo por mi y solo mostrar un texto indicando que lo sabemos y el boton de ir a la venta correspondiente.
![[{0E768A3D-8016-4880-9AEB-17DC311EC1AF}.png]]

Tenemos otro comportamiento extraño, en la prueba 4 lo hice directo en el chat ninguna la estoy haciendo directo del formulario por que ese es el comportamiento esperado, pero en este caso se quedo con el contexto anterior, la converssacion no se ha cortado ni nada cada prueba una tras otra en el chat para cuasar este tipo de problemas, se quedo en el contexto de la ventana de legales, no eligio correctamente contacto y ubicacion
![[{F0380711-147B-4BAE-83F9-BE4C0257371F}.png]]

Resultados de la bitacora:
![[{3D4DC67A-763F-4609-9622-17A8A5B106E9}.png]]

**Respuesta del CTO de la IA (2026-10-20):**

- **La ventana de datos personales** ya no sale en la rama correcta. La primera vuelta se hizo con la app de `main`, abierta desde la carpeta `cimiento`.
- **La latencia no es de la bóveda.** La bóveda es código, no un modelo, y tarda milisegundos. Una sola llamada de prueba, fuera de la app, con 1,500 tokens de entrada y sin razonamiento, tardó 28.5 s, y la red a Gemini responde en 0.3 s. Ese día `gemini-3.5-flash-lite` tardó de 21 a 31 s por llamada, contra 1 a 4 s el día anterior en la misma batería: es el servicio.
- **La ventana equivocada (prueba 4):**
  - La pantalla le mandaba al modelo, como si fuera del Asistente, la frase del botón («…empezando por «Datos legales y fiscales»»).
  - Ahora cada turno lleva las palabras del Asistente y, aparte, la ventana que ofreció; lo que dijo el programa va como del programa.
  - La secuencia entró a la batería. Corrida real con el modelo más barato, 5 rondas: la calle fue a «Contacto y ubicación» 5 de 5, y el correo desde «Mi institución» y desde Inicio, 9 de 10 (la otra, «servicio ocupado»); $0.28 MXN.
  - **Ojo:** el control, la misma secuencia con la conversación como se mandaba antes, también salió 4 de 4 (la quinta, «ocupado»). El arreglo quita una confusión real, pero la medición no reprodujo la falla; se confirma en la app con el paso de arriba.
- **Las dos llamadas (prueba 3)** son la deuda de IA-4, que pasa a ser la fase F3 del [[ADR-048-el-flujo-de-datos|ADR-048]].

**Cierre (2026-10-20, decisión del líder en el ADR-048):** se fusiona con lo que funciona. La ventana de «datos personales» ya no aparece, y ningún valor marcado sale hacia la IA en ninguna batería. Lo que no pasó en la app queda para las fases siguientes:
- **La representante y la calle ofrecidas en otra ventana:** el mapa del Asistente no listaba los campos protegidos de cada ventana. Va a **F1 · El modelo de datos**.
- **«Calle Ana María Rodríguez» marcada entera como un nombre, y un nombre sin marcar:** va a **F2 · El escáner inteligente**.
- **Las dos llamadas al contar datos:** va a **F3**.
