# 02 · Flujo funcional

## Etapas de un proyecto

```
PROFILE → CALL_SELECTION → DIAGNOSIS → PRIORITIZATION → DRAFTING → REVIEW → READY
```

El perfil de la institución es previo y compartido por todos los proyectos; cada proyecto valida que esté completo antes de empezar.

**Desde ADR-016 el proyecto nace de su convocatoria:** al empezarlo la persona sube la convocatoria (y sus anexos, marcando qué es cada archivo) y dice cómo se llama, quién la convoca y de qué año. El proyecto ya existe con su convocatoria leyéndose en segundo plano; `CALL_SELECTION` es ahora «confirmar la convocatoria» y va antes del diagnóstico (ADR-017).

| Etapa | Qué pasa | Quién hace qué | Condición para avanzar (código) |
|---|---|---|---|
| `PROFILE` | Verificar perfil de la institución | Código revisa campos obligatorios | Perfil completo y confirmado en los últimos 12 meses |
| `CALL_SELECTION` | Confirmar la convocatoria de la que nació el proyecto | Código lee el PDF; directivo revisa «esto es lo que entendimos» y la confirma | Lectura `ready` o `partial` y convocatoria confirmada por la persona |
| `DIAGNOSIS` | Conversación guiada: apertura, hasta 5 porqués y causa de fondo (ADR-017) | IA conduce y redacta; código decide el paso y garantiza el final; directivo responde y confirma la causa de fondo | Causa de fondo confirmada y resumen confirmado |
| `PRIORITIZATION` | Elegir el objetivo (ADR-018) | IA propone objetivos que atacan la causa de fondo y encajan con la convocatoria; código calcula puntaje; directivo elige | Un objetivo marcado como «el proyecto» |
| `DRAFTING` | Textos por secciones + presupuesto + cronograma | IA redacta cada texto; directivo lo corrige y confirma; código calcula y valida presupuesto y cronograma | Textos requeridos, presupuesto y cronograma confirmados |
| `REVIEW` | Revisión final | Código corre el checklist (sin IA) | Checklist sin errores (los avisos no bloquean) |
| `READY` | Guía en Word | Código arma el .docx con las conclusiones; la persona lo usa para llenar los formatos del donante | — |

Reglas:

- Se puede **regresar** a cualquier etapa anterior. Al regresar, lo que dependa de ella se marca como "por revisar" (no se borra).
- No se puede **saltar** etapas.
- Las transiciones viven en `domain/stage.rs` con pruebas para cada camino.

## Diagnóstico: conversación guiada (ADR-017)

Es la rutina del proyecto, no un chat libre. Va **siempre** en este orden y **solo con IA** (si falla, lo escrito queda guardado y se reintenta):

1. **Contexto primero:** el perfil de la institución (lo que ya se sabe no se pregunta) y la convocatoria confirmada (qué financia, a quién, montos, indicadores y resultados esperados).
2. **Apertura:** una sola pregunta, siempre con la estructura **idea de propuesta – obstrucción – beneficio futuro**: qué proyecto tienen en mente, por qué todavía no se ha podido resolver y cómo cambiaría la vida de las personas o la operación de la institución, medido con lo que la convocatoria pide. Si la idea no encaja con lo que la convocatoria financia, la IA lo señala (`fit`), sin bloquear.
3. **Cinco porqués:** la IA pregunta «¿por qué?» sobre lo que la persona acaba de decir, hasta 5 veces. La causa raíz suele aparecer en el 3.º o 4.º y es el **centro del proyecto**. La IA puede proponerla desde el 3.º si la causa que la persona dio consta literalmente en su texto; en el 5.º el código la propone con lo mejor que haya.
4. **Confirmación explícita:** «La causa de fondo parece ser X. ¿Es así?» Solo un «sí» la cierra (el silencio no confirma). Un «no» con explicación pide una propuesta nueva; un segundo «no» toma las palabras de la persona tal cual.
5. **Resumen:** con la conversación y la causa de fondo, la IA arma el resumen (misma salida de abajo); lo que no se preguntó (alternativas, sostenibilidad, cotizaciones) queda en `open_questions`.

El código lleva la cuenta de la fase, el tope de porqués y el estancamiento (dos respuestas vagas seguidas → opciones cerradas; tres → se propone lo que hay). Detalle y decisiones en `adr/ADR-017-conversacion-guiada-con-ia.md`.

El resumen es un borrador: el directivo lo confirma o lo corrige (un resumen corregido vuelve a pedir confirmación). Todo número del resumen debe aparecer en lo que el directivo dijo; si no, el código pide una corrección y, si persiste, lo señala en pantalla. Sin la IA no hay resumen: la persona intenta otra vez cuando vuelva.

Salida del diagnóstico (JSON validado):

```json
{
  "problem_statement": "Planteamiento del problema en una oración",
  "affected": { "group": "adultos mayores", "count": 18, "description": "..." },
  "current_consequences": ["..."],
  "root_causes": ["..."],
  "reframed_need": "Espacio de higiene seguro y accesible para adultos mayores con movilidad reducida",
  "alternatives": [{ "title": "...", "pros": ["..."], "cons": ["..."] }],
  "suggested_indicators": ["Número de caídas en baño", "Tiempo promedio por baño asistido"],
  "open_questions": ["Dato que falta confirmar"]
}
```

## Priorización

La IA propone necesidades (puede ser más de una a partir del diagnóstico). El **puntaje lo calcula el código** con criterios que el directivo califica del 1 al 5:

| Criterio | Peso inicial |
|---|---|
| Personas beneficiadas (proporción de la población) | 25 % |
| Gravedad del riesgo si no se atiende | 25 % |
| Relación con la misión | 20 % |
| Viabilidad (costo, tiempo, capacidad de ejecutar) | 15 % |
| Sostenibilidad | 15 % |

Los pesos son configurables. El encaje con convocatorias se evalúa en la etapa siguiente para no mezclar "qué necesitamos" con "qué nos financian".

## Selección de convocatoria y checklist

Cada convocatoria se convierte (una sola vez) en un `call_template` con requisitos estructurados:

```json
{ "id": "REQ-03", "text": "Monto máximo solicitado", "kind": "max_amount", "value": 500000, "blocking": true, "source": { "document_id": "...", "page": 4 } }
```

Tipos de requisito evaluables por código: `max_amount`, `min_amount`, `allowed_category`, `required_document`, `deadline`, `max_duration_months`, `min_beneficiaries`, `cofunding_percent`, `section_required`, `max_length_chars`. Lo que no encaje en un tipo queda como `manual` y el directivo lo marca a mano.

La IA solo interviene para: extraer requisitos del PDF la primera vez (el humano los confirma) y explicar en lenguaje sencillo por qué algo no cumple.

## Redacción (ADR-018)

- **Secciones.** Siempre: los datos de la convocatoria y lo que hay que entregar (los arma el código). Después, la propuesta del proyecto: si la convocatoria pide una propuesta en documento aparte y dice qué debe incluir, hay una sección por cada exigencia (las secciones base que no cubre quedan opcionales); si no, una estructura base: qué, por qué, para quién, dónde, cuándo, cómo, cuánto, resultados e indicadores, cómo se mantendrá. El código propone cuál aplica y la persona lo confirma.
- Cada texto se redacta con ayuda automática desde lo ya confirmado (perfil, convocatoria, resumen, causa de fondo, objetivo, presupuesto, cronograma), se corrige y se confirma por separado. Es borrador hasta que la persona lo confirma.
- El presupuesto es una tabla de `budget_item` capturada por la persona; IVA, totales, contrapartida y tope administrativo los calcula el código y los compara con lo que pide la convocatoria.
- El cronograma son actividades y meses; el código valida los meses y la duración contra el máximo de la convocatoria.
- Si cambia el presupuesto o el cronograma, pierden su confirmación y los textos que hablan de ellos quedan «por revisar».

## Revisión

Checklist automático, sin IA, que se recalcula con un botón. **Errores** (bloquean): monto fuera de [mínimo, máximo], contrapartida no alcanzada, tope administrativo excedido, duración mayor al máximo, totales que no cuadran, presupuesto o cronograma sin confirmar, secciones requeridas sin confirmar, y el escáner sobre todo lo que irá a la guía con algún hallazgo. **Avisos** (no bloquean): fecha de cierre ya pasada, la idea no encaja bien con lo que apoya la convocatoria, montos en otra moneda, documentos por reunir.

## Listo: la guía en Word

El formato del donante **no** se llena aquí. Se genera un Word con las conclusiones del proyecto para que la persona las pase a los formularios y documentos que le toquen: datos de la convocatoria (con dónde lo dice cada uno), qué hay que entregar, resumen, propuesta del proyecto, presupuesto, cronograma, indicadores, datos de la institución y pendientes. Detalle en `adr/ADR-018-redaccion-revision-y-guia-en-word.md`.
