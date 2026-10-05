# Caso dorado: "remodelación de baño"

Caso de referencia para evaluar la conversación del diagnóstico y su resumen. Si un cambio en prompts, modelos o lógica empeora este resultado, no se acepta.

Institución: `institucion-asilo.json`.

## Conversación simulada (ADR-017)

El caso se juega como la conversación guiada: la IA conduce y la persona responde con estas respuestas (las mismas están en `src-tauri/src/ai/golden.rs`). La convocatoria es una ficticia confirmada de antemano.

| Paso | Qué dice la persona |
|---|---|
| Apertura (idea, obstrucción, beneficio) | Queremos remodelar el baño de la planta baja con regadera a ras de piso y barras de apoyo, porque hoy el piso resbala y no hemos tenido dinero para hacerlo; así los 14 abuelitos con movilidad reducida podrían bañarse con seguridad y habría menos caídas. |
| Porqué 1 | Porque el edificio es de los años 70 y no se pensó para personas mayores; no hay barras ni regadera a ras de piso. Este año hubo 3 caídas, una con fractura de cadera, y 6 de ellos van en silla de ruedas. |
| Porqué 2 | No se ha hecho nada más por falta de dinero; solo se pusieron tapetes antiderrapantes, pero se mueven. |
| Porqué 3 | Porque no tenemos un plan ni un fondo para adecuar la casa; solo se arregla cuando algo falla. Bañar a una persona en silla de ruedas requiere a 2 personas y tarda casi 40 minutos, y en la noche solo hay 1 enfermera. |
| (Si hace falta) porqués 4 y 5 | El mantenimiento lo cubre la institución y no hay un responsable que revise la casa. / No habíamos pedido ayuda antes porque no sabíamos que existían convocatorias para esto. |

La conversación termina cuando la IA propone la causa de fondo y la persona la confirma («Sí, es esa»). Entonces se arma el resumen.

## Resultado esperado (criterios, no texto exacto)

El resumen del diagnóstico debe:

- [ ] Replantear la necesidad como **seguridad y accesibilidad en la higiene de adultos mayores con movilidad reducida**, no como "remodelación de baño".
- [ ] Usar las cifras: 14 personas afectadas, 6 en silla de ruedas, 3 caídas en el año (1 con fractura).
- [ ] Identificar la **carga del personal** como consecuencia: 2 personas y ~40 minutos por baño asistido; 1 enfermera de noche.
- [ ] Proponer al menos estas alternativas o componentes: regadera a ras de piso, barras de apoyo, piso antiderrapante, silla de baño/regadera, puerta amplia; y considerar capacitación del personal en movilización segura.
- [ ] Sugerir **indicadores medibles**: número de caídas en baño, tiempo promedio por baño asistido, número de residentes que se bañan con menor asistencia.
- [ ] Marcar como `open_questions` los datos que faltan (por ejemplo: cotizaciones, medidas del baño, costo de mantenimiento anual).
- [ ] No inventar cifras que no se dieron.
- [ ] No incluir nombres ni datos personales.

## Ejemplo de planteamiento aceptable

> Adecuar el área de higiene de planta baja para que 14 personas adultas mayores con movilidad reducida, 6 de ellas usuarias de silla de ruedas, puedan bañarse de forma segura y con mayor autonomía, reduciendo el riesgo de caídas (3 en el último año, una con fractura) y la carga del personal de cuidado.
