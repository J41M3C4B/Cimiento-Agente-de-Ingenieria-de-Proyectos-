---
tipo: bloque
id: ERP-5
carril: erp
estado: backlog
prioridad: alta
caja: Mi institución
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones. Para cumplirla, el IA-1 retira la IA vieja que vive dentro de Mi institución y saca los vectores de `core`:
- el Asistente, el Capturista y sus herramientas (`core::agents::assistant`, `capturist` y `tools`);
- la IA del Lector (`core::reader::ask`, `core::reader::fill` y la lectura con IA al adjuntar, `core::reader::upload::after_upload`);
- los vectores (`core::archive::vectors`), que pasan a la plataforma.

Estas partes de Mi institución usan algo de eso:
- `core/profile/forms.rs`: la vista de un formulario usa `capturist::Fill` y `fill_of_form`;
- `core/overview.rs` y la prueba de `core/places.rs`: recorren `capturist::FILLABLE`;
- `commands/records.rs`: al adjuntar un documento, `suggest_from` llama a la lectura con IA y emite `record-suggested`;
- `core/onboarding/service.rs`: lee la configuración de la IA (`ai::settings`) para saber si está lista;
- `core/archive/search.rs`: la búsqueda por significado usa los vectores.

Se necesita que **Mi institución deje de depender de esas piezas y de la IA**. Lo que hacían se vuelve a construir sobre la plataforma; no se conserva.

Las propuestas por campo (`core::agents::proposals`) y la lectura con reglas del Lector (`core::reader`, sin IA) no se retiran en el IA-1.

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
