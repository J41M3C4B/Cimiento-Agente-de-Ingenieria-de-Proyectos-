---
tipo: bloque
id: ERP-2
carril: erp
estado: backlog
prioridad: media
caja: Clasificación de datos (lo declara el ERP); Datos operativos del ERP
pedido_por: IA-3
depende_de: IA-3
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

La política de datos (IA-3) necesita saber de qué clase es cada dato, y solo el módulo conoce sus datos. Hoy cada campo trae una sensibilidad, pero hay datos que no la tienen o que se tratan distinto en otro lado. Este bloque hace que **cada dato del ERP declare su clase una sola vez**, con las clases que fije el ADR del IA-3.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] Cada campo del catálogo y cada dato de los módulos declara su clase en un solo lugar.
- [ ] Una prueba recorre todos los datos y falla si alguno no tiene clase.

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
