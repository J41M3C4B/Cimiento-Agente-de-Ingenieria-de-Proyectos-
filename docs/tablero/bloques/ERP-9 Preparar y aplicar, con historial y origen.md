---
tipo: bloque
id: ERP-9
carril: erp
estado: backlog
prioridad: alta
caja: Unidad de trabajo (K4); Historial y origen (K5); Eventos (K8, la bandeja de hechos)
pedido_por:
depende_de: ERP-8
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El [plano del ERP](../../plano-del-erp.md) (§7) dice que todo cambio de datos se **prepara** y luego se **aplica**, venga de una persona o de la plataforma. Es el punto donde la IA entra de forma nativa sin que el ERP la conozca, y el IA-2 ata la aprobación a lo que este bloque entrega. Hoy no hay propuesta con diferencias, ni versión de cada registro, ni huella. Además, el historial y el origen solo existen para los campos de Mi institución (`core_field`, `core_change`), y cada tabla repite sus columnas de origen. Este bloque construye en el núcleo la propuesta, la huella, la versión y la unidad de trabajo, y el historial genérico.

**Lo fija el ERP y lo usa el IA-2:** la forma de la propuesta y de la huella. Este bloque va primero o a la par, y el formato se acuerda con el riel IA antes de cerrarlo.

## Terminado cuando

(Propuesta; la aprueba el líder.)

- [ ] **Un cambio del módulo de muestra se prepara** sin escribir nada y devuelve una propuesta: las diferencias campo por campo (antes y después), lo que no pasa la validación, la versión base y la huella.
- [ ] **Aplicar revisa la versión y la huella:** si el dato o la entrada cambiaron desde que se preparó, no se aplica y pide preparar de nuevo. Una prueba lo demuestra.
- [ ] **Al aplicar, una sola transacción** escribe el dato, su historial con su origen (persona, documento y página, cálculo o tarea con su id), su hecho en la bandeja y el registro del cambio. Si algo falla, no queda nada. Una prueba lo demuestra.
- [ ] **El historial sirve para cualquier entidad de cualquier módulo,** solo crece, y lo protegido dice que cambió sin guardar el valor.
- [ ] **El riel IA conoce y acepta** la forma de la propuesta y de la huella, anotada en el IA-2.
- [ ] `cargo test` y `pnpm test` pasan, sin red ni gasto.

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
