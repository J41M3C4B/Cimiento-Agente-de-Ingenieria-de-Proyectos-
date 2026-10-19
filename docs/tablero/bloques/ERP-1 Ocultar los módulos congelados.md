---
tipo: bloque
id: ERP-1
carril: erp
estado: backlog
prioridad: alta
fase: MVP-1
rama:
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

Personal, Beneficiarios, Instalaciones, Finanzas y Donantes están congelados hasta validar el MVP (ADR-043 §3), pero la app los sigue mostrando. Hay que dejar la app enfocada en el núcleo y Proyectos sin borrar su código.

## Terminado cuando

(Propuesta del CTO, por aprobar.)

- [ ] El riel, «Mi institución», el primer inicio (incluida la bienvenida), Inicio y el mapa de lugares del Asistente solo muestran el núcleo y Proyectos.
- [ ] Nada del núcleo lee un módulo oculto para mostrarlo: lo que tomaba de ellos (ocupación, cifras rápidas, balance) se lo queda el núcleo o deja de mostrarse.
- [ ] El código de los módulos compila y todas las pruebas pasan, también `cargo test architecture`.
- [ ] Encenderlos de nuevo es un solo cambio, documentado.

## Plan

## Cómo probarlo

## Resultado
