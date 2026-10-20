---
tipo: bloque
id: ERP-4
carril: erp
estado: backlog
prioridad: media
fase: MVP-2
patron: [determinista]
rama:
adr: "[[ADR-044-la-boveda]]"
creado: 2026-10-19
cerrado:
---

## Qué y por qué

Anotado desde IA-2 (la bóveda, ADR-044).

Desde IA-2, el Asistente y el Capturista ya no detienen a la persona por un dato personal. Lo que escribe va a la IA con marcas (`[NOMBRE_1]`), y el código pone el valor en el equipo. Por eso:
- el servicio ya no devuelve `status: "quarantine"` en `assistant_ask` ni en `capture_ask`;
- los dos comandos ya no reciben `decision`.

Lo que queda sin usar, y es del carril `erp`:
- **La pantalla del Asistente** (`src/core/assistant/Assistant.tsx` y su prueba): la ventana de «datos personales» del chat, `QuarantineDialog` en ese lugar, el `decision` que manda y sus tipos.
- **`ScannerConfig::cleared`** y su uso en el escáner (ADR-041). Nadie lo llena ya.

La cuarentena sigue donde se guarda texto: las ventanas de «Mi institución», los documentos y Proyectos.

Con la bóveda, marcar de más ya no cuesta en el chat. Las listas de nombres del escáner se reemplazan por un modelo local en IA-8 (ADR-046, carril `ia`).

## Terminado cuando

- [ ] 

## Plan

## Cómo probarlo

- [ ] 

## Resultado
