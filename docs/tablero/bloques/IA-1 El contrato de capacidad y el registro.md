---
tipo: bloque
id: IA-1
carril: ia
estado: backlog
prioridad: alta
caja: Registro común de capacidades; Contratos de capacidades
pedido_por:
depende_de:
rama:
adr: ADR-001
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Hoy no hay una forma común de decir qué ofrece un módulo: cada IA tiene sus propias herramientas escritas a mano, y las pantallas llaman directo a 134 comandos. Este bloque fija el **contrato de capacidad** y el **registro** que lo descubre. Todo lo demás de los planos se apoya en esto.

El contrato declara:
- nombre y módulo;
- entrada y salida, con su esquema;
- efecto: consulta, transformación, modificación, o acción sensible o externa;
- el permiso que pide;
- las clases de datos que toca;
- si pide aprobación;
- sus límites.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] El ADR-001 describe el contrato y el registro, y el líder lo aprueba.
- [ ] Un módulo de prueba declara capacidades, y el registro las descubre sin que nadie las escriba en una lista.
- [ ] El registro rechaza un contrato incompleto, como uno sin permiso o sin efecto, y una prueba lo demuestra.
- [ ] Una prueba de arquitectura impide que un módulo llame a la IA por su cuenta.

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
