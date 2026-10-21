---
tipo: bloque
id: ERP-10
carril: erp
estado: backlog
prioridad: alta
caja: Autenticación, permisos y almacén de auditoría; Datos operativos del ERP; Eventos y tareas automáticas
pedido_por: IA-6
depende_de:
rama:
adr:
creado: 2026-10-21
cerrado:
---

## Qué y por qué

**Pedido por el riel IA.** El líder aprobó [el plano de la IA](../../plano-de-la-ia.md) y sus decisiones A1 a A7 el 2026-10-21 ([ADR-003](../../adr/ADR-003-la-plataforma-de-ia-por-dentro.md)). Su §7 cuadra los dos planos punto por punto. Para que la plataforma se acople al núcleo sin que el ERP la conozca, necesita del núcleo siete cosas (plano de la IA, §8). Van juntas en este pedido; **el riel ERP decide en qué bloque suyo entra cada una**, o si alguna se vuelve un bloque aparte.

1. **El contrato de capacidad debajo del núcleo (A2). Es el más urgente, porque toca al ERP-8, que está en curso.**
   - Qué se necesita: que el contrato nombre el permiso por su id, y que el catálogo de permisos, los roles y la decisión sigan en K2.
   - Para qué: que el núcleo también pueda publicar capacidades (documentos, búsqueda, historial), y que el gestor de contexto las pida por el ejecutor.
2. **Una pregunta de autorización en K2.**
   - Qué se necesita: dada la cuenta, la organización y el permiso, si se puede o no; y qué clases de datos ve esa cuenta.
   - Para qué: el ejecutor revisa el permiso antes y después de cada invocación, y la política filtra por las clases que ve el rol.
3. **La plataforma, dueña de sus tablas (A3).**
   - Qué se necesita: que la plataforma use el mecanismo de migraciones por dueño del K4, con sus tablas `ai_`.
   - Para qué: guardar sus tareas, pasos, aprobaciones y el uso de los modelos.
4. **Saber en K5 si una huella se aplicó.**
   - Para qué: si un `apply` no contestó, la plataforma no lo repite a ciegas.
5. **Un evento de tarea en el almacén de auditoría (K9):** apertura, paso, denegación, aprobación y cierre, con el id de la tarea, la capacidad, el resultado y el motivo, sin datos personales.
6. **Un actor del sistema en K8 (A7),** con permisos mínimos y sin poder aprobar nada.
   - Para qué: que las reacciones a hechos y los vencimientos corran como tareas sin ser una persona.
7. **Que los tipos de salida de una capacidad nombren sus campos del diccionario (A5).**
   - Para qué: que la clase de un dato se declare una sola vez, en K3, y la política filtre campo por campo.

**Además, para cuadrar los planos:** que el plano del ERP enlace el plano de la IA. El riel IA no edita el plano del ERP.

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

## Resultado
