---
tipo: bloque
id: IA-7
carril: ia
estado: backlog
prioridad: alta
caja: Contratos de capacidades; Registro común de capacidades; Ejecutor; Aprobación
pedido_por: ERP-9
depende_de: IA-5, ERP-9
rama:
adr:
creado: 2026-10-23
cerrado:
---

## Qué y por qué

**Pedido por el riel ERP.** El [[ERP-9 Preparar y aplicar, con historial y origen|ERP-9]] puso en el núcleo la propuesta, la huella y la unidad de trabajo: todo cambio de datos se prepara con `kernel::propose` y se aplica con `kernel::apply`, en una sola transacción, con su historial, su origen y su hecho. El riel IA aceptó el formato, y su «Resultado» deja una lista de lo que pone el riel IA. Hoy la plataforma todavía no lo usa:

- **La aprobación se ata a la huella del ejecutor**, la de la capacidad y su entrada, y no a la `proposal.fingerprint` del núcleo, que también incluye la versión del dato.
- **La `Call` no lleva la huella aprobada**, y el contrato no tiene un error para «el dato cambió desde que se aprobó» (`prepare_again`).
- **Ninguna capacidad puede aplicar en el núcleo.** `kernel::apply` abre su transacción con la conexión en exclusiva (`&mut Connection`), pero la `Call` y todo el ejecutor trabajan con la conexión compartida. Por eso, en el `rebase` del [[IA-5 Las capacidades de un módulo fuera de la app|IA-5]], la capacidad de la muestra quedó de lectura (aprobado por el líder el 2026-10-23).
- **Un `apply` que no contestó** termina hoy en `uncertain_change` sin preguntar al núcleo. `kernel::applied` ya puede decir si esa huella se aplicó.

Para qué: el ERP-1 publica las capacidades de cambio de Mi institución, y cada una debe pasar por el núcleo con la aprobación atada a lo que la persona vio.

**Lo que dejó escrito el ERP-9 para el riel IA:**
1. `Call.fingerprint` y `CapabilityError::Stale` (en la pantalla, con el motivo `data_changed`).
2. Que el ejecutor ate la aprobación a `proposal.fingerprint`.
3. Que el registro exija que el `prepare` de toda capacidad de cambio devuelva una `kernel::Proposal`.
4. Cómo traduce el contrato los demás resultados de `apply`: `invalid` como `Invalid`; `bad_origin`, `bad_fact`, `version_not_raised` y `database` como `Failed`.
5. Que una propuesta sin cambios no pida aprobación.
6. Que un `apply` incierto se consulte con `kernel::applied` y nunca se aplique por su cuenta.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] **Una capacidad de cambio aplica en el núcleo.** La muestra publica `muestra.guardar_nota`, con su permiso `muestra.notas.escribir`:
  - su `prepare` devuelve la `kernel::Proposal`;
  - su `apply` llama a `kernel::apply`, que guarda la nota con su historial, su hecho y el origen de la tarea (el id de la tarea y la cuenta que aprobó).

  Una prueba de la app la recorre de punta a punta: a la Dirección se le niega, y la Administración prepara, aprueba y guarda. Nada escribe saltándose el núcleo.
- [ ] **La aprobación se ata a la huella del núcleo.** La persona aprueba `proposal.fingerprint`, y el `apply` la recibe en `Call.fingerprint`. Si el dato cambió después de aprobar, el contrato responde `CapabilityError::Stale`, y el ejecutor prepara de nuevo y pide otra aprobación, con el motivo `data_changed`. Una prueba lo demuestra.
- [ ] **El registro lo hace cumplir.** Rechaza una capacidad de cambio cuyo `prepare` no devuelve una `kernel::Proposal`.
- [ ] **Cada resultado de `apply` tiene su traducción fija:**
  - `invalid` se vuelve `Invalid`, con su campo y su código;
  - `already_applied` termina bien, con el mismo cambio;
  - `bad_origin`, `bad_fact`, `version_not_raised` y `database` se vuelven `Failed`.
- [ ] **Una propuesta sin cambios no pide aprobación:** la tarea termina y dice que no había nada que cambiar.
- [ ] **Un `apply` incierto se consulta, no se repite.** Si la app se cerró a media aplicación, al volver el ejecutor pregunta a `kernel::applied` por la huella aprobada:
  - si se aplicó, la tarea termina bien;
  - si no, la tarea lo dice y no aplica sola.

  Una prueba lo demuestra.
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
