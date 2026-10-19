---
tipo: bloque
id: IA-2
carril: ia
estado: backlog
prioridad: alta
fase: MVP-2
rama:
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

Hoy el escáner detiene a la persona cada vez que cree ver un dato personal, y cada error pide una regla nueva. Con la bóveda, hacia la IA van marcas tipadas (`[CORREO_1]`, `[NOMBRE_1]`) y nunca el valor. La IA decide qué es cada marca por su contexto, puede preguntar qué se sabe de una marca sin recibir el valor, y el código la resuelve en el equipo según lo que acepta cada campo. Sin la ventana de «datos personales» en el chat.

**Falta una decisión del líder para escribir el ADR-044:** si lo de salud de una sola persona puede llegar ligado a una marca o se generaliza como hoy (el CTO recomienda generalizarlo).

## Terminado cuando

(Por definir con el ADR-044.)

- [ ] Ningún valor de la bóveda aparece en ninguna solicitud a la IA, en todas las baterías.
- [ ] El Asistente y el Capturista ya no muestran la ventana de «datos personales».
- [ ] Una corrida real con el modelo más barato.

## Plan

## Cómo probarlo

## Resultado
