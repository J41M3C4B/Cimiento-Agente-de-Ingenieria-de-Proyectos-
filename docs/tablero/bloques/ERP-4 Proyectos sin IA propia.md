---
tipo: bloque
id: ERP-4
carril: erp
estado: backlog
prioridad: alta
caja: Proyectos
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones, y la nueva arquitectura manda sobre el código viejo.

Hoy Proyectos tiene sus propias tareas de IA (conversación, diagnóstico, prioridades, redacción y lectura de convocatorias), y por eso usa `crate::ai` en su código y en sus comandos. Se necesita que **Proyectos no use la IA de ninguna forma**, para que la prueba del IA-1 pase. Lo que hacían esas tareas se vuelve a construir después como capacidades de la plataforma, no se conserva.

## Terminado cuando

(Lo escribe el riel ERP con el líder.)

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

**Decisión del líder (2026-10-20): camino B.** Proyectos no avanza hasta que el núcleo esté listo. Lo que dependa de la IA **se quita, no se sustituye a mano**: las etapas cuya condición de avance la produce la IA (`root_cause_confirmed` y `diagnosis_summary_confirmed` de la conversación y el resumen, y `call_confirmed` de la lectura de la convocatoria) se retiran hasta que existan las capacidades nuevas. El «Terminado cuando» se escribe cuando el ERP-5 termine (a lo más un bloque en curso por riel), con el líder.

**Comandos que este bloque retira** (primera lista, para el DIS-4; se confirma al planear):
- conversación: `conversation_start`, `conversation_send`, `conversation_retry`;
- diagnóstico: `diagnosis_summary_generate`, `needs_propose`;
- redacción: `drafting_prepare`, `section_draft`, `sections_draft_all`;
- convocatorias: `project_create_from_call`, `call_brief_make`, y por consecuencia `call_reading_get`, `call_reading_confirm` y `call_reading_retry`.

**Lo que se queda de los comandos de Proyectos** (sin IA): `project_list`, `project_create`, `project_delete`, `project_set_color`, `project_set_donor_kind`, y el presupuesto, el calendario, la revisión y la guía (`budget_*`, `schedule_*`, `review_get`, `guide_export`). Qué etapas quedan y cómo avanzan se fija al escribir el «Terminado cuando».

**Para el IA-1 y el riel IA:** `ai/dry_run_tests.rs` y `test_support.rs` llaman a funciones de Proyectos que este bloque retira (`start_conversation`, `send_message`, `generate_summary`, `draft_section`…). Hay que acordar quién ajusta esas pruebas.

## Resultado
