---
tipo: bloque
id: IA-1
carril: ia
estado: en-curso
prioridad: alta
caja: Registro común de capacidades; Contratos de capacidades
pedido_por:
depende_de: ERP-4, ERP-5, DIS-3
rama: ia/ia-1-contrato-de-capacidad
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

**Lo que propone el ADR-001** (el líder lo aprueba al validar):

1. **Dónde vive.** El contrato y el registro van en `src-tauri/src/capability/`, que no depende de la IA. Los módulos solo usan eso. La IA (`src-tauri/src/ai/`) pasa a ser la capa de la plataforma, y ningún módulo la puede usar.
2. **Cómo se declara.** Cada capacidad es un `static` en el código de su módulo, marcado para el registro. El registro las junta al enlazar (crate `linkme`), así que no hay ninguna lista. Un módulo nuevo es el módulo más sus capacidades: la plataforma no cambia.
3. **Qué declara:**
   - un id `módulo.nombre`, el módulo y una descripción;
   - la entrada y la salida, con un esquema JSON que sale del tipo de Rust (crate `schemars`), para que el esquema y el código no se separen;
   - el efecto: consulta, transformación, modificación, o acción sensible o externa;
   - el permiso que pide, del vocabulario de permisos de las cuentas, que es del riel ERP;
   - las clases de datos que toca. Son provisionales; el IA-3 fija las definitivas;
   - si pide aprobación;
   - sus límites: tiempo, reintentos y si es idempotente;
   - la función que la ejecuta, con la firma que usará el ejecutor del IA-2.
4. **Qué rechaza:**
   - **Al compilar:** un contrato al que le falte un campo, como el permiso o el efecto, no compila. Una prueba `compile_fail` lo demuestra.
   - **Al armar el registro:**
     - sin clases de datos;
     - un id repetido o mal formado;
     - una acción sensible sin aprobación;
     - reintentos en una operación con efectos que no es idempotente;
     - una entrada que no es un objeto.

     Una prueba recorre el registro real en `cargo test`, así que un contrato inválido no llega a `main`.
5. **La frontera.** En `architecture_tests.rs`, `ai` deja de ser base y pasa a ser la capa de la plataforma. Mi institución (`core`), los módulos y Proyectos no la pueden usar, sin lista de excepciones. Los permisos (`core::access::domain`) se clasifican como capa de control, para que el contrato los pueda usar sin depender de Mi institución.

**Los pasos:**

1. El borrador del ADR-001, para que el líder lo vea pronto.
2. Probar primero que `linkme` funciona en Windows, porque es el riesgo principal.
3. `capability/`: el contrato, el registro, la validación y un módulo de prueba que declara capacidades.
4. La prueba de arquitectura estricta.
5. **Retirar la IA que vive dentro de Mi institución** (riel IA):
   - `core/agents`: el Asistente, el Capturista y sus herramientas, con sus oficios y baterías;
   - la IA del Lector: `reader/ask`, `reader/fill` y la lectura con IA de `reader/upload`.

   Los comandos que solo envuelven eso (`commands/assistant.rs`) salen con ellos, junto con sus líneas en `src/lib/tauri.ts` y `src/lib/types.ts`. Los vectores (la búsqueda por significado) no se borran: salen de `core` hacia la plataforma, como herramienta de la capa 4. Lo que Mi institución usa de esas piezas lo quita antes el ERP-5.
6. **Sacar la IA de los servicios deterministas.** `documents/canonical/run.rs` llama a la IA: es la lectura de convocatorias, que es un operador de IA (capa 4, riel IA). Se mueve a la plataforma, y `documents` se queda solo con lo determinista (el contrato, la recuperación y el paquete). La prueba estricta también impide que un servicio determinista use la IA: solo la plataforma y los comandos la usan.
7. La suite completa, y el PR.

**Lo que se pide a otros rieles:**
- **ERP-4:** Proyectos sin IA propia.
- **ERP-5:** Mi institución sin la IA vieja: la parte del ERP que usa el Capturista, la lectura con IA, la configuración de la IA y los vectores.
- **DIS-3:** las pantallas sin la IA vieja.

**El orden para fusionar:**
1. DIS-3: si va después, las pantallas no compilan sin los comandos.
2. ERP-4 y ERP-5.
3. IA-1: su prueba estricta necesita que ya no quede IA en los módulos.

Mientras tanto, el contrato, el registro y la prueba se construyen en la rama. La prueba estricta falla y enumera lo que falta retirar.

**Lo que no entra en este bloque:**
- Que Mi institución use el escáner de datos personales (`scanner::guard`, en unos 20 archivos) no es llamar a la IA. Es política de datos repartida, y la resuelve el IA-3.
- La configuración de la IA (proveedor, clave, tope) es de la plataforma y se queda.

**Riesgos:**
- ~~`linkme` en Windows.~~ Probado el 2026-10-20 en un proyecto aparte con la misma forma de crate (`staticlib`, `cdylib` y `rlib`, más el binario). Junta las capacidades de dos módulos sin lista, en desarrollo, en pruebas y con el perfil de *release* real (LTO, `codegen-units = 1`, `strip`).
- Hasta el IA-4 y la migración de capacidades, la app queda sin el Asistente, sin la lectura con IA y sin la IA de Proyectos. Lo viejo no se conserva por existir.

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

- **La hoja de la IA (`core/ai_sheet.rs`) no llama a la IA:** solo la usa en sus pruebas. Por eso no rompe la regla del IA-1, y se quitó del Plan. Se retira cuando el gestor de contexto la sustituya. Hoy también la usan `core::api` y Proyectos (la guía exportada).
- **Las propuestas por campo (`core/agents/proposals.rs`) tampoco llaman a la IA.** Son la forma vieja de la confirmación, y se retiran cuando el IA-2 traiga la aprobación atada a los parámetros. Las usan los formularios y la lectura con reglas.

## Resultado
