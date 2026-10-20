---
tipo: bloque
id: IA-1
carril: ia
estado: listo
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

Aprobado por el líder el 2026-10-20. Las casillas se marcan cuando cada punto queda hecho.

- [ ] El ADR-001 describe el contrato y el registro, y el líder lo aprueba.
- [ ] Un módulo de prueba declara capacidades, y el registro las descubre sin que nadie las escriba en una lista.
- [ ] La clase de datos es un campo obligatorio del contrato. Las clases de IA-1 son provisionales: el ADR del IA-3 fija las definitivas y su política sin cambiar la forma del contrato.
- [ ] El registro rechaza un contrato incompleto, como uno sin permiso, sin efecto o sin clase de datos, y una prueba lo demuestra.
- [ ] Una prueba de arquitectura impide que un módulo llame a la IA por su cuenta. Es estricta y no tiene excepciones: la nueva arquitectura manda, y el código viejo que la viole no se acomoda, se retira.

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
