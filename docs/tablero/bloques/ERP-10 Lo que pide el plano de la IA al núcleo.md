---
tipo: bloque
id: ERP-10
carril: erp
estado: backlog
prioridad: alta
caja: Autenticación, permisos y almacén de auditoría; Datos operativos del ERP; Eventos y tareas automáticas
pedido_por: IA-6
depende_de: ERP-8
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

**Dónde entra cada punto (decisión del riel ERP, 2026-10-21):**

| Punto | Bloque | Estado |
|---|---|---|
| 1. El permiso por su id (A2) | [[ERP-8 La forma de un módulo y el registro de módulos\|ERP-8]] | Hecho en su PR (#24). El `enum Permission` no se mudó. El núcleo guarda el catálogo por id (`Manifest::permissions`, `Modules::permission(id)`). Lo que falta es del IA-5: que el contrato nombre el permiso por id. |
| 3. La plataforma, dueña de sus tablas (A3) | ERP-8 | Hecho en su PR (#24): la tabla `kernel_migrations (owner, version)`, `kernel::migrate_owner(conn, "ai", …)` y `kernel::conform` para cualquier dueño. El id `ai` queda reservado. |
| El plano del ERP enlaza el de la IA | ERP-8 | Hecho en su PR (#24), con su fila en la §13. |
| 2. La pregunta de autorización y las clases que ve cada cuenta (K2) | **Este bloque** | Por hacer |
| 5. El evento de tarea en el almacén de auditoría (K9) | **Este bloque** | Por hacer |
| 4. Saber si una huella se aplicó (K5) | [[ERP-9 Preparar y aplicar, con historial y origen\|ERP-9]] | Anotado en su tarjeta |
| 6. El actor del sistema (K8, A7) | [[ERP-3 El calendario de vencimientos\|ERP-3]] | Anotado en su tarjeta |
| 7. Los tipos de salida nombran sus campos del diccionario (A5) | [[ERP-2 La clase de cada dato declarada una sola vez\|ERP-2]] | Anotado en su tarjeta |

Este bloque se queda con lo que el ejecutor del IA-2 necesita del núcleo para cada invocación: autorizar y dejar registro.

## Terminado cuando

(Propuesta del riel ERP; la aprueba el líder.)

- [ ] **Los permisos generales del núcleo,** los que hoy son el `enum Permission` (usar, borrar, ajustes, administrar), tienen id en el catálogo de K2, igual que los de los módulos.
- [ ] **Los roles agrupan permisos por id,** y los dos roles de hoy (administración y dirección) siguen pudiendo lo mismo.
- [ ] **K2 responde una pregunta:** dada la cuenta, la organización y el id del permiso, sí o no. Una cuenta bloqueada, un permiso que no existe o una organización ajena dicen que no.
- [ ] **K2 dice qué clases de datos ve una cuenta,** con las clases del IA-3. Si el IA-3 no ha cerrado, queda el lugar y una prueba con las clases de hoy.
- [ ] **El almacén de auditoría (K9) guarda el evento de tarea:** apertura, paso, denegación, aprobación y cierre, con el id de la tarea, la capacidad, el resultado y el motivo. Una prueba demuestra que no guarda datos personales.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto.

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
