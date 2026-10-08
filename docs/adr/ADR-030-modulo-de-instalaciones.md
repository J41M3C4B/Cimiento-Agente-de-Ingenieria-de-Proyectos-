# ADR-030 · Módulo de Instalaciones

**Estado:** aceptada (2026-10-07). Reemplaza la lista `facility` del perfil (migración 0001).

## Contexto

Hasta ahora, las instalaciones eran una lista dentro del perfil: un tipo de espacio escrito a mano, cuántos hay, **un solo estado para todos**, si es accesible y una nota.

Con eso no se podía decir «hay 4 baños y solo 1 está mal», ni en qué piso están. Tampoco se sabía cuántos metros tiene la casa, de quién es o con qué servicios cuenta. Esos datos importan para la IA y para hacer inteligencia de datos más adelante.

La institución pidió tres cosas:

- Algo **práctico, sencillo y específico**.
- Datos relevantes para la IA y para la inteligencia de datos.
- **Desacoplarlo** como Personal (ADR-027) y Beneficiarios (ADR-029), para migrarlo después a un módulo especializado.

## Decisión

1. **Un módulo aparte** (`src-tauri/src/facilities/`, tablas `fac_*`, migración 0018) con la misma frontera que los otros dos:
   - Usa solo la bitácora y `common`, y una prueba lo vigila.
   - La app le habla por `facilities::api` (`summaries`, `indicators`) y por `facilities::service`.
   - La coordinación con el resto vive en `facilities_service.rs`: escáner, tablero y ejemplos.
   - El frontend vive en `src/features/facilities/`.
   - El tipo de institución lo pasa la app (`Flavor`) y solo ordena las sugerencias.
2. **Un inmueble, preparado para varios.** La pantalla maneja uno (`fac_site`), que se crea al guardar el primer dato; las tablas ya admiten varios.
   - **El inmueble:** nombre, m² de terreno y construidos, pisos, cómo se sube de un piso a otro (rampa, elevador, salvaescaleras o solo escaleras) y año aproximado.
   - **Tenencia:** propio, comodato, rentado, prestado o de otra forma; hasta qué año y si tiene papeles que lo acrediten.
   - **Servicios:** de dónde llega el agua, qué tan seguido falta y cuántos litros se pueden guardar; qué tan seguido se va la luz; el gas, el drenaje y el internet.
   - **Seguridad y protección civil:** extintores y su recarga, detectores de humo, salidas señaladas, luces de emergencia, botiquín, programa interno, dictamen o visto bueno (con su año) y simulacros al año.
3. **Espacios por grupo, con conteo por estado** (`fac_space`). Por ejemplo, «Baños · primer piso · 4 → 3 bien, 1 mal».
   - **Qué espacio es:** un tipo de un catálogo de 17. El tipo «otro» pide un nombre; cualquier otro grupo puede llevar un nombre propio opcional.
   - **Piso y estado:** el piso va del sótano al piso 19. Se dice cuántos están bien, regular, mal o no sirven.
   - **Lo no contado queda «sin revisar».** No saber no es «bien». Los estados nunca pueden sumar más que el total.
   - **Qué les falla**, de una lista de 13 problemas. Grietas y fallas eléctricas cuentan como riesgo estructural.
   - **Accesibilidad:** si se pueden usar en silla de ruedas.
   - **Datos de su tipo:** en dormitorio y enfermería, las camas y cuántas son de hospital; en baño, si tiene barras de apoyo y regadera accesible.
   - **Una nota.**
4. **Equipo por grupo** (`fac_equipment`), con el mismo conteo por estado. Tiene un catálogo de 16: sillas de ruedas, grúa, colchones antiescaras, oxígeno, lavadoras, refrigeradores, calentadores, planta de luz, paneles solares, computadoras, vehículos…
5. **Indicadores y hallazgos con reglas fijas** (`facilities/domain/aggregate.rs` y `domain/facility_insights.rs`).
   - **Indicadores:** m² construidos por persona atendida, personas por baño, camas contra personas y capacidad, estado por tipo de espacio, lo que más falla y lo que falta capturar.
   - **Hallazgos:**
     - Espacios y equipo en mal estado, con nombre y piso: «1 de 4 baños (primer piso)».
     - Riesgo estructural.
     - Varios pisos con solo escaleras, cruzado con las personas en silla de ruedas o en cama (Beneficiarios).
     - Baños sin barras de apoyo, en un asilo o con personas de movilidad limitada.
     - Menos camas que personas o que capacidad.
     - Inmueble que no es propio ni está en comodato, un comodato que termina en menos de 5 años, o la falta de papeles.
     - Protección civil incompleta y seguridad contra incendios.
     - Falta de agua, y apagones sin una planta de luz que funcione.
     - Espacios sin revisar (solo para la persona).
   - **No se califica contra la norma.** Las proporciones se muestran sin comparar con la NOM-031-SSA3 ni la NOM-032-SSA3: sus umbrales se programan solo cuando se confirmen en el texto oficial.
6. **Lo que llega a la IA.** Una instalación no tiene datos personales, así que la ficha recibe todo:
   - El inmueble, cada grupo de espacios y de equipo en palabras, y las proporciones calculadas.
   - Los hallazgos marcados `for_ai`.
   - **Excepción:** un conteo de personas con un atributo (en silla de ruedas, en cama) solo llega con 3 o más personas, como en el ADR-029.
   - **El escáner:** los textos libres (nombre del inmueble, nombres propios y notas) pasan por él antes de guardarse, porque llegan a la IA.
   - **Las cifras de la ficha:** ninguna lleva «.0», porque la revisión de cifras la leería como un 0.
7. **Beneficiarios** toma del módulo los espacios a los que no se puede entrar en silla de ruedas, ahora con cuántos son y en qué piso: «3 baños (planta baja)».
8. **La guía en Word** lista el inmueble, sus espacios y su equipo con las mismas frases que lee la IA.
9. **Accesos (ADR-028).** Quitar un grupo de espacios o de equipo no se vuelve solicitud para el administrador: no tiene datos personales y se vuelve a capturar en un minuto. Todos los comandos piden el permiso `Use`.
10. **Traslado** (migración 0018 + `storage::migrations::move_profile_facilities`):
    - Pasan las filas de la **última versión** del perfil.
    - El tipo escrito pasa a un código del catálogo («Baño» → `bathroom`). Las palabras propias se conservan como nombre cuando dicen más («Baños de mujeres»), y lo que no se reconoce pasa como «otro» con su nombre.
    - El único estado viejo se aplica a todos los espacios del grupo («muy grave» pasa a «no sirve»); sin estado, quedan sin revisar.
    - Todo pasa a la planta baja del inmueble principal y conserva su `origin` y su `source_ref`.
    - Se registra `facilities.imported` y se elimina la tabla `facility`.

## Consecuencias

- **El perfil ya no tiene instalaciones.** Desaparecen `FacilityInput`, `Condition` y la lista `facilities`, y las versiones anteriores del perfil pierden su copia; el módulo guarda el estado actual.
- **El borrado de emergencia de un documento** (ADR-019) sigue alcanzando a lo que venga de él: `fac_space` y `fac_equipment` llevan `source_ref`.
- **Ejemplos:** `fixtures/instalaciones-asilo.json` y `fixtures/instalaciones-casa-hogar.json` se cargan con los botones de ejemplo de «Mi institución».
- **Frontend:** hay un frontend que funciona (espacios, inmueble con servicios y seguridad, equipo y tablero). El diseño fino queda para la sesión de diseño.
- **Pendiente para el módulo especializado:**
  - Fotos de cada grupo (cifradas y sin EXIF).
  - Historial de mantenimiento y de reparaciones.
  - Presupuestos de obra por espacio.
  - Varios inmuebles en pantalla.
  - Umbrales de la norma oficial.
  - Exportar a Excel.

## Alternativas descartadas

- **Un registro por espacio** («Baño 1», «Baño 2»…): es más preciso, pero se captura mucho más. El grupo con conteo por estado basta para la IA y los indicadores.
- **Un solo estado por grupo**, como antes: esconde justo lo que importa, cuántos están mal.
- **Tomar lo no contado como «bien»**: inventaría datos. «Sin revisar» le dice a la persona qué le falta.
- **Calificar ya contra la norma**: los umbrales no están confirmados en el texto oficial, y un hallazgo equivocado restaría confianza.
