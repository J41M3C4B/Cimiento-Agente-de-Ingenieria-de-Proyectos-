---
tipo: bloque
id: IA-2
carril: ia
estado: backlog
prioridad: alta
caja: Runtime; Ejecutor; Estado, límites y ciclo de vida; Entrega de resultados
pedido_por:
depende_de: IA-1
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

La columna vertebral del plano operativo, **todavía sin IA**:
- el runtime que recibe toda petición;
- la tarea con su estado, sus límites y su id de correlación;
- un solo ejecutor que aplica los controles antes y después de cada invocación;
- la entrega común de resultados.

Se arma primero sin modelo para probar el camino con un flujo determinista, sin gastar nada. Así el ciclo sin fin que hoy tiene el agente queda imposible por diseño.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] Una petición recorre runtime → tarea → ejecutor → entrega con una capacidad determinista de prueba, y el resultado llega con el id de la tarea.
- [ ] Toda tarea registra su cierre: terminada, pausada, denegada, por límite o con error, con su motivo y sin datos personales.
- [ ] Los límites de pasos, reintentos, gasto y tiempo cortan una tarea que no avanza. Una prueba lo demuestra.
- [ ] Una modificación se prepara, espera aprobación atada a sus parámetros, y se vuelve a pedir si los parámetros cambian.

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
