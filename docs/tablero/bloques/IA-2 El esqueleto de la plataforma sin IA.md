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

**Acuerdos con el riel ERP (2026-10-21):**
1. **La aprobación.** El IA-2 ata la aprobación a la entrada desde ahora, como dice el ADR-001. La versión del dato y la huella del [[ERP-9 Preparar y aplicar, con historial y origen|ERP-9]] se agregan después. Su formato lo acuerdan el riel IA y el riel ERP antes de cerrar el ERP-9; está escrito en su tarjeta.
2. **Los archivos de compilación.** El ERP-8 cambia `src-tauri/Cargo.toml` y `Cargo.lock` para armar el *workspace*. Cuando el ERP-8 entre a `main`, el IA-2 hace `rebase` antes de su PR.
3. **`Permission`.** Se muda al núcleo, pero `core::access::domain` lo sigue exponiendo, así que `capability/` y `ai/` no cambian por el ERP-8. Lo que el IA-5 pide aparte son los permisos que declara cada módulo.

**Propuesta para empezar:** primero, limpiar `ai/`. Se conserva lo que dice `codigo-frente-a-los-planos.md` (proveedores, configuración, la bóveda y el ritmo) y la lectura de convocatorias detenida, con lo que usa. Se retiran las tareas viejas de Proyectos, y las pruebas del proveedor apagadas (`dry_run_tests.rs`) se portan a un impulsor propio. Después, el esqueleto.

## Terminado cuando

(Propuesta; la aprueba el líder. Ajustada el 2026-10-21 con [el plano de la IA](../../plano-de-la-ia.md): piezas P1, P2, P7, P8, P9 y P12, y decisiones A3 y A4.)

- [ ] Una petición de pantalla que nombra una capacidad recorre runtime → tarea → ejecutor → entrega, como flujo determinista, sin enrutador ni modelo y con todos los controles. El resultado llega en el contrato común de resultados, con el id de la tarea.
- [ ] Toda tarea registra su cierre: terminada, pausada, denegada, por límite o con error, con su motivo y sin datos personales.
- [ ] Los límites de pasos, reintentos, gasto y tiempo cortan una tarea que no avanza, y la misma capacidad con la misma entrada dos veces la cierra por límite. Una prueba lo demuestra.
- [ ] Una modificación se prepara, espera aprobación atada a sus parámetros, y se vuelve a pedir si los parámetros cambian. La huella del ERP-9 se agrega cuando exista.
- [ ] Las tareas se guardan en la base: una tarea pausada o esperando aprobación sigue ahí al volver a abrir la app, y se reanuda.

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
