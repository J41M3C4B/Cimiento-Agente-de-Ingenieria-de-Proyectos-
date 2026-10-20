# 08 · Estilo de redacción

Aplica a **todo texto visible** en la app y a los prompts de la IA cuando hablan con el usuario. Todos los textos de UI viven en `src/i18n/es-MX.ts`.

## Voz

Como una persona de confianza que sabe del tema y quiere ayudar: cálida, clara, paciente. Nunca un sistema, nunca un consultor presumiendo.

- **Trato:** de *usted* cordial (usuarias: directivas y religiosas). Configurable a *tú* en ajustes. *(Decisión pendiente de validar con la institución.)*
- **Español de México**, natural, sin regionalismos fuertes.
- **Oraciones cortas.** Una idea por oración.
- **Primero lo que hay que hacer**, luego el porqué (si hace falta).
- **Sin culpa:** los errores son de la app, no de la persona.

## Lo que ve la persona no es lo que lee la IA

La estructura que consume la IA (esquema de la convocatoria, ficha de «Mi institución» para el prompt, JSON de las respuestas) **no se le muestra a la persona como vista principal**. Ella ve una *ficha*: corta, concisa, en sus palabras, para entender rápido y decidir. El detalle estructurado puede ofrecerse aparte, como «Consultar el detalle». Ver ADR-024.

## Prohibido en la UI

Jerga técnica: *PII, input, output, prompt, token, JSON, query, upload, dashboard, error 500, null, sincronizar, parámetro, campo obligatorio, validación fallida.*

## Glosario técnico → sencillo

| En vez de | Decir |
|---|---|
| Subir archivo | Agregar documento |
| Campo obligatorio | Este dato nos falta |
| Datos sensibles / PII | Datos personales de alguien |
| Validación fallida | Algo no cuadra |
| Procesando… | Estamos leyendo su documento… |
| Error de conexión | No pudimos conectarnos. Revise el internet e intente otra vez. |
| Exportar | Guardar el documento listo |
| Plantilla | Formato |
| Requisito bloqueante | Esto es indispensable para la convocatoria |
| Beneficiarios | Personas que se van a beneficiar |
| Sin confirmar | Falta que usted lo revise |

## Microtextos de referencia

**Bienvenida**
> ¡Hola! Vamos a armar su proyecto paso a paso. Usted solo responda; nosotros nos encargamos del formato y las cuentas.

**Cuarentena por datos personales**
> Ojo: este documento parece traer nombres y CURP de personas. Para cuidarlas, mejor los tapamos antes de guardarlo.
> [Tapar y seguir] [Mejor no guardarlo]

**Archivo tipo expediente**
> Este archivo parece una lista de personas. Para cuidarlas, no lo guardamos. Si necesita esos datos para el proyecto, mejor escriba solo los totales (por ejemplo: "12 de 18 usan silla de ruedas").

**Dato sugerido por la IA**
> Esto lo sugerimos nosotros con lo que nos contó. ¿Es correcto? [Sí, está bien] [Corregir]

**Algo no cuadra**
> El total del presupuesto ($184,500) no coincide con lo que se pide en el cuestionario ($180,000). ¿Cuál es el correcto?

**Requisito no cumplido**
> Esta convocatoria apoya hasta $150,000 y el proyecto pide $184,500. Podemos ajustar el presupuesto o buscar quién ponga la diferencia.

**Tope de gasto de IA**
> Este mes ya se usó el presupuesto de ayuda automática. Puede seguir trabajando; las sugerencias regresan el próximo mes o si se amplía el tope.

**Error inesperado**
> Algo falló de nuestro lado. Su trabajo está guardado. Intente otra vez y, si sigue pasando, avísele a [responsable].

## Preguntas del diagnóstico

- Una pregunta a la vez.
- Abiertas y concretas: "¿Qué pasa hoy por no tener esto?" mejor que "Describa la problemática".
- Pedir cifras, nunca nombres: "¿A cuántas personas…?"
- Reconocer la respuesta antes de seguir: "Entiendo, entonces son 18 abuelitos y 6 usan andadera."
- Nunca juzgar ni corregir con tono de examen.

## Documentos para el donante

El texto **del proyecto** (lo que lee la fundación) sí usa un registro formal y técnico-social adecuado: objetivos, metas, indicadores, beneficiarios. La sencillez aplica a la app, no al documento final.

## Accesibilidad

- Tamaño de letra base grande (18 px) y opción para agrandar.
- Alto contraste; nunca transmitir información solo con color.
- Botones con texto, no solo íconos.
- Máximo una acción principal por pantalla.
