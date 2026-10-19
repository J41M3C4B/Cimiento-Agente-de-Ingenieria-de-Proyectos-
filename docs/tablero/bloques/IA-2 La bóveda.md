---
tipo: bloque
id: IA-2
carril: ia
estado: backlog
prioridad: alta
fase: MVP-2
patron: [determinista]
rama:
adr: "[[ADR-044-la-boveda]]"
creado: 2026-10-18
cerrado:
---

## Qué y por qué

Hoy el escáner detiene a la persona cada vez que cree ver un dato personal, y cada error pide una regla nueva. Con la bóveda, hacia la IA van marcas tipadas (`[CORREO_1]`, `[NOMBRE_1]`) y nunca el valor. La IA decide qué es cada marca por su contexto, puede preguntar qué se sabe de una marca sin recibir el valor, y el código la resuelve en el equipo según lo que acepta cada campo. Sin la ventana de «datos personales» en el chat.

**Decidido por el líder (2026-10-18):** lo de salud de una sola persona se sigue generalizando antes de salir, como hoy, aunque el nombre vaya como marca. Solo los grupos de 3 o más llevan atributos de salud. Se puede revisar más adelante.

**Patrón (ADR-045):** determinista. La bóveda no usa IA: marca, resuelve y escribe la leyenda con código. Es del carril `ia` porque decide lo que recibe la IA ([[roles-y-tablero|Roles y tablero]]); el escáner sigue siendo del carril `erp` y no cambia.

**La propuesta:** [[ADR-044-la-boveda|ADR-044]].

## Terminado cuando

(Propuesto por el CTO de la IA el 2026-10-19; falta la aprobación del líder.)

- [ ] **Ningún valor marcado aparece en ninguna solicitud a la IA, en todas las baterías en seco:** Asistente, Capturista, los tres Lectores y la conversación de Proyectos. Una revisión en el proveedor simulado lo comprueba en cada solicitud.
- [ ] **El Asistente y el Capturista ya no se detienen por un dato personal:** el servicio ya no devuelve la ventana, y «No es dato personal» y `cleared` salieron del chat.
- [ ] **Una marca se resuelve solo donde el campo la acepta.** Pruebas: el correo y el teléfono en su campo; la representante y el RFC que cuenta la persona en «Datos legales»; «alcaldía Benito Juárez» en el municipio aunque el escáner lo tome por persona; un nombre en la misión vuelve a `[NOMBRE OCULTO]`; una marca inventada se descarta.
- [ ] **La persona lee sus valores:** la respuesta llega a la pantalla con cada marca resuelta.
- [ ] **La salud de una persona sale generalizada**, sin número, y nunca se resuelve.
- [ ] **Las baterías con casos nuevos** (dos personas distintas, el correo y la representante de la institución, la alcaldía, una CURP en la pregunta, salud de una persona), primero en seco y después en una corrida real con el modelo más barato. Asistente y Capturista no empeoran contra el v11 y el v5 en aciertos, inventados ni costo.
- [ ] El principio 5, `04-escaner-datos-sensibles.md`, `07-ia-y-costos.md` y la metodología de los patrones dicen cómo sale un dato personal: como marca.

## Plan

(Se detalla al pasar a «En curso».)

1. `ai/vault.rs`: marcar y resolver, con sus pruebas (mismo valor, misma marca; la salud sin número; marcas mal escritas; nada en disco).
2. `ai::pipeline` y `ai::agent` con una bóveda por encargo; herramientas con sus argumentos resueltos y su resultado marcado.
3. En `core/agents`: qué acepta cada campo, la leyenda, la respuesta resuelta para la pantalla. El Asistente y el Capturista sin la ventana.
4. Oficios v12 (Asistente) y v6 (Capturista), con los casos nuevos de las baterías en seco.
5. La revisión «ningún valor en ninguna solicitud» en el proveedor simulado, para todas las baterías.
6. Corrida real: Asistente y Capturista, dos rondas, a 14 por minuto, salida a archivo.
7. Documentos. Una nota al carril `erp`: el código de la ventana del Asistente queda sin usar, y con la bóveda puede medir si quita reglas por caso del escáner.

## Cómo probarlo

## Resultado
