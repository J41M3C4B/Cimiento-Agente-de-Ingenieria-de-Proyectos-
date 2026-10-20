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
