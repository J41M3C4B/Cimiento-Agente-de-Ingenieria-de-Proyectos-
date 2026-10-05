# ADR-024 · Dos capas: lo que consume la IA no es lo que ve la persona (la «Ficha» de la convocatoria)

**Estado:** Aceptada (2026-10-04), a petición de la persona dueña del proyecto tras probar el paso 1. Primera ronda: la convocatoria.

## Contexto
La app tiene estructuras que existen **para la IA**: el esquema canónico de la convocatoria (ADR-015), la ficha de «Mi institución» que va en los prompts (ADR-023), los JSON de las respuestas. En el paso 1 se le mostraba a la persona esa estructura tal cual: unos 25 grupos de líneas con archivo y página, repetidos en el paso 1, en el panel del proyecto y en «Ver todo el detalle». Las usuarias (directivas y religiosas) no necesitan leer el esquema para decidir; necesitan entender rápido de qué trata la convocatoria y si les conviene. No existía un resumen para ellas: lo más parecido estaba escondido en la ventana de detalle.

La auditoría mostró que pasa en varios lugares: el resumen del diagnóstico copia el JSON de la IA campo por campo (y su editor, uno por uno), la pantalla «Ayuda automática» muestra modelos y unidades del servicio, una sección de redacción puede mostrar la frase cruda de la convocatoria, y Documentos habla de «fragmentos».

## Decisión
1. **Tres capas.** (a) **IA**: esquema, fichas para el prompt, JSON; nunca es la vista principal. (b) **Persona**: una *ficha* corta y en sus palabras. (c) **Consulta**: la estructura completa con citas, detrás de «Consultar el detalle», para comprobar algo puntual.
2. **La ficha la compone Rust** (`documents/canonical/card.rs`, `CallCard`), no la pantalla: la lógica de qué es importante vive en el código, como el resto de las reglas. La pantalla solo la dibuja. Los textos son claves; las palabras viven en `es-MX.ts`.
3. **Ficha de la convocatoria:** quién es; «en pocas palabras» (3 o 4 frases); cifras clave con su página; «¿nos conviene?» en bloques de a lo más 3 puntos con «y N más» (quién puede participar, qué apoya, qué se puede pagar y qué no); alertas solo si existen (documentos que se contradicen, datos clave que faltan, dudas, lectura parcial). La telemetría de la lectura sale de la vista principal.
4. **«En pocas palabras» híbrido.** Cifras, fechas y listas las arma el código sin IA. Las frases las escribe la IA una sola vez (`call.brief`, nivel ligero), a partir del mismo texto corto que ya recibe la conversación (`CallSummary::context_text`), con la revisión de cifras de siempre y un reintento. Si falla o inventa cifras, no se guarda y la ficha sale igual con el objetivo del documento. Se guarda en `call_reading.brief_text` y se borra cuando la convocatoria se lee otra vez. Se genera la primera vez que se abre una lectura que no lo tiene (también las ya leídas), protegido por el registro de procesos (ADR-023) para no gastar dos llamadas.
5. **En el paso 1** la pregunta pasa de «¿es la correcta?» a «¿es la convocatoria que quiere usar?»: una sola acción principal. **En el panel del proyecto** va la ficha compacta en lugar de la lista de temas.
6. **Convención de nombres:** los tipos que se muestran a la persona terminan en `Card` (o `View` cuando es una vista de datos de la persona); los de la IA son `…Context`, canónico o `…Summary` de lectura, y no se importan en una pantalla. Regla 8 de `docs/agents/principios-de-ingenieria.md` y nota en `docs/08-estilo-redaccion.md`.

## Lo que enseñaron las convocatorias reales
Se imprimió la Ficha de tres lecturas reales (NMP 2025 y 2026, Alsea) sin llamadas (`cargo test print_card_of_a_real_call -- --ignored`). Midió ~2,000 a ~3,100 caracteres frente a la lista de ~30 temas y decenas de líneas. Y mostró defectos que los datos de prueba no tenían, ya corregidos y con prueba:
- Algunas lecturas archivan las mismas líneas en «qué apoya» y en «lo que se puede pagar»: el segundo bloque se omite si repite al primero. Dentro de un bloque, una línea repetida se dice una vez y no cuenta en «y N más».
- Algunas convocatorias no dan fecha de cierre sino de registro o postulación: la ficha usa «Cuándo se postula» (nunca una sesión informativa). Un número suelto donde debía haber una fecha («15») no se muestra.
- Una convocatoria que da el dinero por modalidad no tiene monto único: la ficha nombra las modalidades.

## Consecuencias
- Un tope de tamaño de la ficha, probado, para que no vuelva a ser una listota.
- Una llamada ligera por convocatoria (la primera vez que se abre), sin releer el documento.
- Pendientes de la auditoría, para las siguientes rondas: resumen del diagnóstico narrado (los campos solo al corregir); «Ayuda automática» en sencillo con lo técnico en «Avanzado»; la frase cruda de la convocatoria en redacción cuando la IA no la explicó; «fragmentos» en Documentos; y retirar el código de la lista de temas (`CallTopics`) que ya nadie usa.
