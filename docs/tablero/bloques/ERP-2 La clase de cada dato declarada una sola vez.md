---
tipo: bloque
id: ERP-2
carril: erp
estado: en-curso
prioridad: media
caja: Clasificación de datos (lo declara el ERP); Datos operativos del ERP
pedido_por: IA-3
depende_de: ERP-8
rama: erp/erp-2-diccionario-de-datos
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

La política de datos (IA-3) necesita saber de qué clase es cada dato, y solo el módulo conoce sus datos. Hoy cada campo trae una sensibilidad, pero hay datos que no la tienen o que se tratan distinto en otro lado. Este bloque hace que **cada dato del ERP declare su clase una sola vez**, con las clases que fije el ADR del IA-3.

**Cambio con el plano del ERP (ERP-7, 2026-10-20).** El lugar único es el **diccionario de datos del núcleo** (K3, [plano del ERP](../../plano-del-erp.md), §5 y paso 4 de la §11). Cada módulo declara ahí sus entidades y sus campos, y de ahí salen la validación, «qué falta» y la clase que usa la política. `AiUse` sale del dato: qué ve la IA es política del IA-3, no una propiedad del campo. Al terminar se pide al riel IA que «falta información de la persona» pueda nombrar campos del diccionario.

**Del [[ERP-10 Lo que pide el plano de la IA al núcleo|ERP-10]] (punto 7, ADR-003 A5):** que los tipos de salida de una capacidad nombren sus campos del diccionario, para que la clase se declare una sola vez y la política filtre campo por campo.

## Terminado cuando

Aprobado por el líder el 2026-10-23, **sin esperar al IA-3:** el diccionario usa las clases provisionales del núcleo (`kernel::CLASSES`, las mismas que `DataClass` del contrato), y el ADR del IA-3 las sustituye sin cambiar la forma del diccionario.

- [ ] El núcleo tiene el diccionario: cada módulo declara sus entidades y campos, con su tipo, su regla, cuándo aplica, si es obligatorio y su clase.
- [ ] El núcleo valida una entrada y calcula «qué falta» de cualquier entidad con el diccionario, sin código del módulo.
- [ ] La prueba de conformidad falla si un campo no tiene clase.
- [ ] El pedido al riel IA está en el tablero.

## Plan

Todo va en el núcleo (`crates/kernel`), y el módulo de muestra lo usa de punta a punta. El código viejo (`common/forms.rs`) se queda para Mi institución hasta el ERP-1, sin duplicar sus reglas.

1. **El diccionario (K3), en `kernel::dictionary`.**
   - Una `Entity` tiene un id (`note`) y sus campos.
   - Cada `Field` lleva:
     - su id;
     - su tipo: texto, texto largo, número, dinero en centavos (D3), año, fecha, una opción, varias opciones, sí o no, correo o teléfono;
     - sus opciones;
     - su presencia: opcional, esperado (cuenta en «qué falta», pero nunca impide guardar) u obligatorio (sin él no se guarda);
     - cuándo aplica: siempre, si otro campo de la misma entidad está lleno, o si tiene alguno de ciertos valores;
     - sus límites (rango de un número o largo de un texto);
     - su regla (código postal, RFC de una organización, CLABE, nombre de un lugar);
     - **su clase**, por id de `kernel::CLASSES`.
   - Las palabras no van ahí: la etiqueta de un campo sigue en `es-MX.ts`, bajo su id.
   - `AiUse` no pasa al diccionario: qué ve la IA es política (IA-3).
2. **El manifiesto lleva el catálogo del módulo** (`entities`). El registro (K12) lo revisa con lo demás del manifiesto, y un módulo que falla no entra. Las faltas que detecta:
   - un campo sin clase o con una clase que no existe;
   - ids mal formados o repetidos;
   - una opción o varias opciones sin opciones;
   - un rango al revés;
   - una condición que mira un campo que la entidad no tiene.
3. **Lo que el núcleo calcula con el diccionario,** sin código del módulo:
   - `validate`: lo que no pasa, con códigos fijos (`required`, `too_long`, `code_unknown`, `not_a_phone`…). Un valor de un campo que no aplica no se revisa y no se guarda.
   - `missing`: los campos esperados u obligatorios que aplican y están vacíos.
   - Dónde está cada campo, por su id completo (`muestra.note.phone`), y su clase. Con esto la política (IA-3) y la salida de una capacidad (A5) toman la clase del diccionario.
4. **Preparar un cambio usa el diccionario** (ERP-9). `kernel::propose` recibe la entidad del diccionario. De ahí salen la validación y qué campos son protegidos en el historial, y el módulo ya no los repite. **Protegido es una regla del núcleo:** todo campo cuya clase no sea `public` ni `internal`. El módulo puede agregar sus reglas propias (que un nombre no se repita, por ejemplo). La forma de la `Proposal` que aceptó el riel IA no cambia.
5. **Las reglas en un solo lugar.** Las comprobaciones de fecha, código postal, RFC de una organización, CLABE y nombre de un lugar pasan al núcleo. `common/forms.rs` y `common/ids.rs` las llaman de ahí, sin cambiar a quien ya las usa.
6. **El módulo de muestra** declara su nota en el diccionario:
   - el título, obligatorio, de hasta 120 caracteres, de clase `internal`;
   - el texto, de clase `internal`;
   - el teléfono, de clase `personal`.

   Su validación propia y su lista de protegidos salen del servicio.
7. **El pedido al riel IA**, como bloque en su Backlog: que «falta información de la persona» (`NeedsPerson`) nombre campos del diccionario, y que el tipo de salida de una capacidad nombre los suyos (A5). Así la política filtra campo por campo.
8. **La suite completa y el PR.** El plano del ERP lleva su fila en «Ajustes» si algo de esto lo precisa.

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
