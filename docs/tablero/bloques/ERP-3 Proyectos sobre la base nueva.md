---
tipo: bloque
id: ERP-3
carril: erp
estado: backlog
prioridad: media
fase: MVP-4
patron: [determinista]
rama: erp/erp-3-proyectos
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

El problema que hace despegar el negocio: llevar de una convocatoria a un proyecto que cabe en ella y que la institución entiende. Parte de lo que dejaron las fases 2 a 4 (convocatoria, diagnóstico, redacción y guía) y del expediente hacia Proyectos (antes N5). Seguramente se parte en bloques de los dos carriles al definirlo.

## Terminado cuando

(Propuesta del CTO del ERP, 2026-10-20, para este primer bloque. **Espera tu aprobación** junto con las decisiones de la revisión.)

- [ ] Con la convocatoria ficticia nueva y cualquiera de los tres ejemplos, un proyecto se crea y llega a la guía en seco. Basta haber terminado el primer inicio: la etapa «Datos de la institución» ya no detiene a nadie.
- [ ] La revisión dice, de cada documento que pide la convocatoria, si está en el Expediente y vigente a la fecha de cierre, si vence antes, si falta, o si no es del Expediente y lo prepara el proyecto.
- [ ] La guía en Word describe a la institución como es: su figura legal, a qué se dedica, a quién atiende y la cifra de personas que le toca. No dice «Institución de asistencia» a cualquiera, no pide capacidad donde no aplica y no habla de módulos congelados. Trae la lista de entrega con el estado de cada documento.
- [ ] Ningún texto de Proyectos (pantallas, manual y guía) habla de asilo, casa hogar o residentes, salvo lo que escribió la institución.
- [ ] Lo que una institución ya guardó en un proyecto no cambia ni se pierde.
- [ ] Todas las pruebas pasan (`cargo test`, `pnpm test`), y el bloque `ia` de la elegibilidad (C1) está en su Backlog.

**El criterio de la fase,** «las dos instituciones preparan un proyecto con una convocatoria real y lo entienden», pasa al último bloque de la partición (abajo): se puede validar hasta que exista «¿Pueden participar?».

## Revisión: Proyectos sobre la base nueva

(CTO del ERP, 2026-10-20. Revisados:
- las siete etapas y sus condiciones (`domain/stage.rs`);
- qué le pide Proyectos al núcleo (`core/api.rs`);
- la revisión automática (`domain/checklist.rs`) y los requisitos de la convocatoria (`domain/requirements.rs`);
- la guía en Word (`guide.rs`) y las pantallas de `src/modules/projects/`.

**Cada punto espera tu decisión** en su última columna.)

**Lo que ya funciona y se queda:**
- El proyecto nace de su convocatoria. Sus archivos se leen por el mismo camino que el Expediente, con OCR, escáner y sin guardar el original.
- La lectura canónica, con citas que revisa el código, y la ficha de la convocatoria.
- La conversación del diagnóstico: el código decide la etapa y cuándo termina, y la IA redacta la pregunta.
- El objetivo, la redacción por secciones y la confirmación de cada una.
- El presupuesto y el cronograma, calculados en Rust.
- La revisión automática: monto máximo y mínimo, contrapartida, gastos administrativos, duración, fecha de cierre y secciones sin confirmar.
- La guía en Word, armada por el código y revisada por el escáner.

### A. Lo que quedó de la base vieja

| # | Hoy | Por qué estorba | Recomendación | Tu decisión |
|---|---|---|---|---|
| A1 | **La etapa «Datos de la institución»** pide que la institución haya confirmado sus datos hace menos de 365 días | Desde el ADR-042, terminar el primer inicio ya confirma los datos, y cada guardado después también. La regla de 365 días es de cuando se confirmaba todo de una vez: si nadie cambia nada en un año, el proyecto se atora sin decir por qué | La condición es «terminó el primer inicio», la misma que usa todo el núcleo. Sin regla de 365 días | |
| A2 | **«¿A cuántas personas atienden?»** sale de los grupos de población, en el diagnóstico (la sugerencia de «personas beneficiadas») y en la guía | Los grupos de población son el detalle que llenaba Beneficiarios, que está congelado. Una institución nueva no los tiene. Las cifras del ERP-2 («hoy» o «en el último año») no se leen | Usar las cifras del núcleo: «hoy» donde las personas viven o pasan el día, y «en el último año» en las demás. Si la institución tiene las dos, mostrar las dos | |
| A3 | **La guía, «8. Datos de la institución»** escribe «Tipo: Institución de asistencia» a toda institución que no es asilo ni casa hogar, y pone la capacidad siempre que la haya | Es del núcleo de antes del ERP-2: el tipo viejo (asilo, casa hogar u otra) | Escribir la figura legal (A.C., I.A.P.…), a qué se dedica, a quién, cómo y en qué atiende, la cifra de A2, el RFC, la donataria y la CLUNI si las tiene. La capacidad solo donde aplica (ERP-2, A1) | |
| A4 | **La guía, «Instalaciones»** sale del módulo de Instalaciones (`core::api::facility_lines`) | El módulo está congelado y oculto (ERP-1). La guía es lo único que todavía lo lee | Quitarla de la guía mientras el módulo esté apagado. Que lo decida el interruptor de `core/modules.rs`, para que vuelva sola al encenderlo | |
| A5 | **Textos de Proyectos:** las pantallas, el manual y los comentarios del código no se revisaron en el ERP-2 (el código todavía dice «su desarrollo está en pausa») | Mismo criterio que el ERP-2 | Revisarlos con la misma vara: sin asilo, casa hogar ni residentes, y sin prometer módulos congelados | |

### B. El Expediente hacia Proyectos (antes N5)

| # | Hoy | Por qué importa | Recomendación | Tu decisión |
|---|---|---|---|---|
| B1 | **La revisión** solo cuenta los documentos obligatorios: «La convocatoria pide 7 documentos obligatorios. Reúnalos antes de entregar» | El Expediente ya sabe qué documentos tiene la institución, si tienen archivo y hasta cuándo valen, y Proyectos no lo usa. Es justo lo que una directora revisa a mano antes de entregar | De cada documento que pide la convocatoria, el código dice uno de cuatro estados: «Lo tienen» (vigente), «Vence antes del cierre», «Falta en su Expediente» o «Lo prepara el proyecto» (cartas, formatos del donante). Para reconocer que «Copia del acta constitutiva» es el acta, usa las frases y las palabras con que se nombra cada tipo de documento («constitutiva», «cluni», «donataria», «32d»), las mismas que ya reconocen un archivo (`reader::kinds`): una lista por tipo, no por caso. Lo que no reconoce queda como «Revíselo usted». Sin IA | |
| B2 | **La fecha** contra la que se mide la vigencia | Un documento puede valer hoy y vencer antes de entregar | Contra la fecha de cierre de la convocatoria. Si no la dice, contra hoy | |
| B3 | **¿Bloquea?** | Un documento vencido puede renovarse antes de entregar | Que sea un aviso, no un error: el proyecto sigue y el estado queda en la guía | |
| B4 | **La guía** no dice qué entregar | Es la lista que la persona usa el día de la entrega | Una sección «Qué entregar», con cada documento, su estado y su archivo en el Expediente (el nombre del archivo solo en la guía; nunca va a la IA) | |

### C. ¿Pueden participar?

| # | Hoy | Por qué importa | Recomendación | Tu decisión |
|---|---|---|---|---|
| C1 | **Nadie revisa si la institución puede participar.** La lectura ya trae «quiénes pueden participar», «quiénes no» y «requisitos de la organización», pero como frases. La IA solo juzga si la idea encaja con lo que apoya la convocatoria (en la primera respuesta de la conversación) | Es lo primero que hay que saber, antes de invertir una hora en la conversación. Una A.C. sin donataria, o con un año de constituida, no debería llegar a la guía para descubrirlo | Un paso «¿Pueden participar?» justo después de confirmar la convocatoria. El código cruza con «Mi institución» lo que se puede revisar sin adivinar: figura legal, donataria, CLUNI, años desde la constitución, estado o región. Lo que no se puede revisar se muestra como «Revíselo usted», con su cita. **Pide un bloque `ia`:** que la lectura entregue esos requisitos con un vocabulario cerrado y su cita. El cruce, la pantalla y los datos del núcleo son del ERP | |
| C2 | **¿Bloquea?** | Un «no cumple» puede ser un error de lectura | No bloquea: dice «No cumple: …» con su cita y la persona decide si sigue | |

### D. El recorrido y cómo se prueba

| # | Hoy | Por qué importa | Recomendación | Tu decisión |
|---|---|---|---|---|
| D1 | **No hay una convocatoria ficticia en PDF.** Hay un texto por páginas de la Fase 3 (`fixtures/convocatoria-ficticia-paginas.txt`, «Fondo Ficticio de Asistencia») que ya no usa ninguna prueba. Las pruebas de Proyectos usan textos sueltos | Sin ella, B y C no se pueden probar de punta a punta, y las convocatorias reales no pueden entrar al repositorio | Partir de ese texto y hacer un PDF (con su `generar.py`, como el Expediente) que pida acta, CLUNI, donataria vigente, dos años de constituida, un monto máximo y una carta del donante (algo que el Expediente no tiene). Y una prueba completa en seco, como la del Expediente: crear, leer con el modelo simulado, revisar los documentos y llegar a la guía | |
| D2 | **Proyectos sin convocatoria** (`internal`): la puerta existe en el modelo y está apagada en la pantalla | El MVP es de convocatoria a proyecto | Se queda apagada | |
| D3 | **Cómo se ve:** las etapas, la conversación y las ventanas | Es del carril `diseno` | Lo que salga de tus pruebas va a un bloque `DIS` | |

### E. Lo que le toca a otros carriles

- **IA:** la elegibilidad con vocabulario cerrado en la lectura de la convocatoria (para C1). Los oficios de Proyectos (conversación, objetivo, redacción) ante la ficha de la IA genérica ya están en [[IA-7 La IA ante el núcleo genérico|IA-7]]. Falta medir el costo de IA por proyecto, que es un criterio de la Fase 11.
- **Diseño:** lo que salga de tu prueba con las instituciones.

### Partición que propongo

1. **ERP-3 (este):** A, B y D1. Todo es del ERP, no espera a nadie y se prueba en seco.
2. **Un bloque `ia`:** la elegibilidad con vocabulario cerrado (C1). Lo escribo en su Backlog cuando lo apruebes.
3. **ERP-5, «¿Pueden participar?»** (C), cuando el bloque `ia` esté hecho. Su «Terminado cuando» lleva el criterio de la fase: las dos instituciones preparan un proyecto con una convocatoria real y lo entienden.

## Plan

## Cómo probarlo

## Resultado
