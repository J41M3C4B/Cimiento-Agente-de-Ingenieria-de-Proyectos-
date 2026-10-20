---
tipo: bloque
id: IA-7
carril: ia
estado: backlog
prioridad: alta
fase: MVP-2
patron: [determinista]
rama:
adr: "[[ADR-046-nombres-con-un-modelo-local]]"
creado: 2026-10-19
cerrado:
---

## Qué y por qué

El escáner reconoce a una persona con listas de nombres y de palabras de lugar, y cada falla pide otra palabra. Con el examen de 708 frases ficticias cubre completas el 17 % de las personas, y ninguna escrita en minúsculas o en mayúsculas.

Un modelo local de reconocimiento de nombres (RoBERTa BNE capitel-plus, ONNX int8, 125 MB, Apache 2.0) cubre el 98.4 %, entiende el contexto y quita las listas. La medición y la decisión están en el ADR-046.

**Patrón:** determinista (1). El modelo no genera texto ni llama a un servicio: con la misma entrada da la misma salida, en el equipo. Necesita un modelo y su batería, y por eso se propone en el carril `ia`. **La frontera la decide el líder:** los formatos del escáner (CURP, RFC, CLABE…) siguen siendo del ERP.

## Terminado cuando

(Propuesta del CTO de la IA; falta la aprobación del líder.)

- [ ] El escáner reconoce a las personas con el modelo local, en Rust, sin red; las listas de nombres, apellidos, palabras de lugar, títulos y santos ya no existen
- [ ] El examen (generador y lo esperado, solo ficticios) está en `fixtures/` y su prueba en seco pasa: personas cubiertas completas ≥ 98 % y no vistas ≤ 1.5 %
- [ ] Los pesos son los oficiales, convertidos por nosotros a ONNX, con su huella guardada; la licencia del modelo y de su corpus está confirmada y anotada
- [ ] Un documento largo se lee por tramos en segundo plano; el tiempo y la memoria están medidos en un equipo modesto y anotados en `07-ia-y-costos.md`
- [ ] El instalador lleva el modelo y el motor, y su tamaño nuevo está anotado
- [ ] Las pruebas del escáner, del Expediente (`full_expediente`) y de las baterías en seco pasan; lo que cambia se explica en el PR
- [ ] `cargo test` completo pasa

## Plan

(Al pasar a «En curso».)

## Cómo probarlo

- [ ] 

## Resultado
