---
tipo: bloque
id: IA-3
carril: ia
estado: backlog
prioridad: media
caja: Clasificación de datos; Políticas de datos y PII
pedido_por:
depende_de: IA-2
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Hoy qué ve la IA se decide en varios lugares: la hoja de la IA, cada herramienta, el escáner y el oficio de cada agente. Este bloque define **las clases de datos** y **la política** que les toca, en el plano de control y una sola vez:
- la política previa al enrutador;
- el contexto mínimo;
- los controles de salida.

Incluye la regla del proveedor: los términos del plan gratuito de Gemini prohíben mandarle datos personales o confidenciales ([investigación externa](../../investigacion/investigacion-externa.md), §4.2), y hoy nada lo impide.

**SubQ como proveedor candidato (nota del líder, 2026-10-21).** El líder tiene una suscripción a SubQ (Subquadratic): un contexto de hasta 12 millones de tokens, con atención dispersa en vez de partir los documentos y recuperarlos por *embeddings*. Ofrece dos modelos, uno de decisión y uno más general, y el líder tiene 100 millones de tokens de regalo. Un documento de una institución no llega ni al millón de tokens, así que alcanzan para probar si sirve para el conocimiento documental (P6, [plano de la IA](../../plano-de-la-ia.md)). Un caso que encaja: las convocatorias, que son públicas y largas.

- **Se hará una investigación especializada** antes de decidir.
- **Lo que este bloque tiene que resolver para SubQ:**
  - sus condiciones (el contrato de API y el acuerdo de tratamiento de datos, que no son públicos): si entrena con lo que recibe, cuánto lo guarda, si lo borra y su confidencialidad (LFPDPPP, art. 52);
  - si los tokens de regalo cuentan como servicio pagado o como gratuito;
  - si vencen.

  Mientras no se sepa, la política solo le deja mandar lo público.
- **La comparación de calidad** (frente a la recuperación por fragmentos) va con la batería, cuando exista el IA-4, con una corrida real corta.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] Un ADR fija las clases de datos y qué permite cada una según el proveedor, y el líder lo aprueba.
- [ ] La política se aplica en el runtime, antes del enrutador, al armar el contexto y a la salida, sin reglas en ningún *prompt*.
- [ ] Con un proveedor gratuito, un dato personal o confidencial no sale, y una prueba lo demuestra.

## Plan

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

## Resultado
