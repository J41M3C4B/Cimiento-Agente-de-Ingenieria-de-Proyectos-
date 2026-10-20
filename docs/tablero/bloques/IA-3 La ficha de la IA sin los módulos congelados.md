---
tipo: bloque
id: IA-3
carril: ia
estado: cerrado
prioridad: media
fase: F1
patron: [determinista]
rama:
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado: 2026-10-20
---

## Qué y por qué

Salió de ERP-1. La ficha de «Mi institución» que lee la IA (`core/ai_sheet.rs`) todavía trae lo que guardan Personal, Beneficiarios, Instalaciones y Finanzas: el dinero, el personal y los espacios. La usan el Asistente (herramienta `sheet_section`) y Proyectos al redactar. Con esos módulos ocultos, la IA podría hablar de datos que la persona no ve. Una institución nueva no tiene esos datos, pero las de prueba sí.

Hay que decidir qué lleva la ficha mientras los módulos están congelados (las cifras rápidas del núcleo en su lugar) y medir que la redacción de Proyectos no empeore.

## Terminado cuando

(Por definir.)

- [ ] La ficha y sus partes no leen un módulo oculto (`core::modules::shown`).
- [ ] El Asistente y Proyectos siguen pasando sus baterías en seco.

## Plan

## Cómo probarlo

## Resultado

**Se juntó con [[IA-9 El modelo de datos|IA-9]] (F1, [[ADR-048-el-flujo-de-datos|ADR-048]], decisión del líder del 2026-10-20):** la ficha de la IA lee el modelo de datos, sin lo de los módulos ocultos.

