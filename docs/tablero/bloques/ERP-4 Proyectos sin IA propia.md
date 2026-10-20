---
tipo: bloque
id: ERP-4
carril: erp
estado: backlog
prioridad: alta
caja: Proyectos
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El IA-1 fija la regla de los planos: **ningún módulo llama a la IA por su cuenta**. Una prueba de arquitectura estricta lo comprueba, sin lista de excepciones, y la nueva arquitectura manda sobre el código viejo.

Hoy Proyectos tiene sus propias tareas de IA (conversación, diagnóstico, prioridades, redacción y lectura de convocatorias), y por eso usa `crate::ai` en su código y en sus comandos. Se necesita que **Proyectos no use la IA de ninguna forma**, para que la prueba del IA-1 pase. Lo que hacían esas tareas se vuelve a construir después como capacidades de la plataforma, no se conserva.

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
