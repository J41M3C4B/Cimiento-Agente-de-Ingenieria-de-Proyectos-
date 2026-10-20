---
tipo: bloque
id: DIS-4
carril: diseno
estado: backlog
prioridad: alta
caja: Interfaces de los módulos
pedido_por: IA-1
depende_de: ERP-4
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Parte del [[DIS-3 Las pantallas sin la IA vieja|DIS-3]], que cubre Mi institución y el Asistente. El IA-1 exige que **ningún módulo llame a la IA por su cuenta**, y el ERP-4 retira de Proyectos sus tareas de IA. Las pantallas de Proyectos están hechas alrededor de ellas, así que hay que decidir cómo se ven **sin** esas llamadas antes de que los comandos se retiren.

Hoy las pantallas de Proyectos llaman, entre otros, a estos comandos (la lista final la fija el ERP-4):
- la conversación: `conversation_start`, `conversation_send`, `conversation_retry`;
- el diagnóstico: `diagnosis_summary_generate`, `needs_propose`;
- la redacción: `drafting_prepare`, `section_draft`, `sections_draft_all`;
- la lectura de convocatorias: `project_create_from_call`, `call_brief_make`.

Se necesita que **las pantallas de Proyectos dejen de usarlos**, y que lo que queda se vea completo y usable a mano (escribir, editar y confirmar). Lo que hacían se vuelve a construir como capacidades de la plataforma, con el chat y las confirmaciones del DIS-1 y el DIS-2; no se conserva.

## Terminado cuando

(Lo escribe el riel de diseño con el líder, cuando el ERP-4 publique qué comandos retira.)

- [ ] 

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

- El texto «asistente» de la bienvenida y del arranque (`onboarding`, `help`) habla del asistente de Proyectos. Se resuelve aquí o en el DIS-1.

## Resultado
