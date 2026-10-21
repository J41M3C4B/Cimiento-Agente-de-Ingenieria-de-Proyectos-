---
tipo: bloque
id: ERP-2
carril: erp
estado: backlog
prioridad: media
caja: Clasificación de datos (lo declara el ERP); Datos operativos del ERP
pedido_por: IA-3
depende_de: IA-3, ERP-8
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

La política de datos (IA-3) necesita saber de qué clase es cada dato, y solo el módulo conoce sus datos. Hoy cada campo trae una sensibilidad, pero hay datos que no la tienen o que se tratan distinto en otro lado. Este bloque hace que **cada dato del ERP declare su clase una sola vez**, con las clases que fije el ADR del IA-3.

**Cambio con el plano del ERP (ERP-7, 2026-10-20).** El lugar único es el **diccionario de datos del núcleo** (K3, [plano del ERP](../../plano-del-erp.md), §5 y paso 4 de la §11). Cada módulo declara ahí sus entidades y sus campos, y de ahí salen la validación, «qué falta» y la clase que usa la política. `AiUse` sale del dato: qué ve la IA es política del IA-3, no una propiedad del campo. Al terminar se pide al riel IA que «falta información de la persona» pueda nombrar campos del diccionario.

**Del [[ERP-10 Lo que pide el plano de la IA al núcleo|ERP-10]] (punto 7, ADR-003 A5):** que los tipos de salida de una capacidad nombren sus campos del diccionario, para que la clase se declare una sola vez y la política filtre campo por campo.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] El núcleo tiene el diccionario: cada módulo declara sus entidades y campos, con su tipo, su regla, cuándo aplica, si es obligatorio y su clase.
- [ ] El núcleo valida una entrada y calcula «qué falta» de cualquier entidad con el diccionario, sin código del módulo.
- [ ] La prueba de conformidad falla si un campo no tiene clase.
- [ ] El pedido al riel IA está en el tablero.

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
