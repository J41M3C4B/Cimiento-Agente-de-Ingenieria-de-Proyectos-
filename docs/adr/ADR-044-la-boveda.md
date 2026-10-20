---
adr: 44
estado: aceptada
fecha: 2026-10-19
tema:
  - ia
  - privacidad
  - escaner
tags:
  - adr
---
# ADR-044 · La bóveda: hacia la IA van marcas, nunca el valor de un dato personal

**Estado:** aceptada (2026-10-19; el líder la aprobó el mismo día). Aplica el primer límite del [ADR-043](ADR-043-un-erp-con-la-ia-en-el-nucleo.md) §5: «el valor de un dato personal no sale del equipo; hacia la IA va una marca». Se construye en el bloque IA-2 (fase MVP-2). La decisión sobre la salud ya la tomó el líder el 2026-10-18 (§3).

## Contexto

Hoy un dato personal se detiene en dos lugares antes de llegar a la IA:

1. **La ventana de «datos personales».** Cuando lo que la persona escribe en el Asistente o en «Llenarlo por mí» trae un nombre, un correo o un teléfono, el programa se detiene y le pide elegir entre «Tapar» y «No es dato personal».
2. **El tapado en `ai::pipeline`.** Antes de cada llamada, lo que el escáner encuentra se cambia por una etiqueta fija: `[NOMBRE OCULTO]`, `[CORREO OCULTO]`.

Funciona, pero cuesta:

- **Frena a la persona** cada vez que el escáner cree ver un dato, aunque sea el de su propia institución.
- **La etiqueta fija pierde quién es quién.** Dos personas son el mismo `[NOMBRE OCULTO]`, y la IA no puede sugerir el teléfono o la representante que la persona acaba de contar. Para eso nacieron «No es dato personal» en el chat, `ScannerConfig::cleared` y la regla de copiar letra por letra (ADR-041).
- **Un falso positivo destruye el dato.** «Alcaldía Benito Juárez» se tapaba como persona, y la salida fue una regla por caso: las palabras de lugar del escáner (`PLACE_WORDS`).

## Decisión

### 1. Marcas en lugar de valores

Todo texto que va a la IA pasa por la bóveda, en el único camino de toda llamada (`ai::pipeline`). Cada hallazgo del escáner se cambia por una **marca con su tipo y su número**:

| El escáner encuentra | Marca |
|---|---|
| Nombre de una persona | `[NOMBRE_1]` |
| Correo | `[CORREO_1]` |
| Teléfono | `[TELEFONO_1]` |
| RFC (de persona o de empresa) | `[RFC_1]` |
| CURP | `[CURP_1]` |
| CLABE, tarjeta, NSS, clave de elector | `[CLABE_1]`, `[TARJETA_1]`, `[NSS_1]`, `[CLAVE_ELECTOR_1]` |

- **El mismo valor lleva la misma marca en todo el encargo:** la pregunta, la conversación, los pasos del agente y lo que devuelven sus herramientas. Dos personas distintas son `[NOMBRE_1]` y `[NOMBRE_2]`.
- **La tabla de valor y marca vive solo en memoria, mientras dura el encargo.** Nunca se escribe en la base, en la bitácora ni en `ai_usage`. La bitácora guarda, como hoy, cuántos datos de cada tipo se marcaron.
- **Lo que hoy no se tapa sigue sin taparse:** el nombre, el teléfono, el correo y las cuentas de la institución, y los nombres de su patronato (ADR-040).

### 2. El código resuelve la marca en el equipo

La IA trabaja con la marca y el código la cambia por su valor, nunca la IA:

- **En una sugerencia, en cualquier campo** (aclaración aprobada por el líder el 2026-10-19). La IA propone `"value": "[NOMBRE_1] 12"` para la calle, y el código pone el valor. Lo que se escribe en el campo lo decide la IA por el contexto, y lo que cabe lo decide el campo:
  - **la validación del campo:** un municipio debe ser del catálogo, un RFC debe tener su forma y un correo, la suya;
  - **el único punto de guardado** (`proposals::propose`), con su escáner, como hoy: un texto libre que no es dato propio, como la misión, se sigue cubriendo ahí;
  - **la persona ve el valor real** antes de aceptarlo.

  Una marca que no existe en el encargo hace que se descarte la sugerencia. No hay una regla de «qué tipo de marca acepta cada campo»: esa regla habría sido otra lista. Así un lugar que el escáner tomó por persona («calle [NOMBRE_1]») no se pierde.
- **En la respuesta a la persona.** Antes de llegar a la pantalla, cada marca se cambia por su valor: la persona lee «Su representante es Ana…», que nunca salió del equipo.
- **En los argumentos de una herramienta.** Una marca se resuelve antes de que la herramienta corra en el equipo, y lo que devuelve se marca otra vez antes de volver a la IA.

Si el modelo escribe mal una marca (`[NOMBRE 1]`, `NOMBRE_1`), el código la reconoce por su forma. Nunca adivina un valor.

### 3. La salud de una sola persona, como hoy

Decidido por el líder el 2026-10-18. Lo de salud de una sola persona se sigue generalizando antes de salir (`[DATO DE SALUD OCULTO]`, sin número y sin resolverse nunca), aunque el nombre vaya como marca. Solo los grupos de 3 o más llevan atributos de salud. Se puede revisar más adelante.

### 4. Lo que la IA sabe de una marca

Con cada encargo, el código agrega una **leyenda de sus marcas**, calculada sin IA: el tipo de cada una y si es igual a un valor ya guardado («`[CORREO_1]`: correo; es el mismo guardado en «Correo de contacto»»). Así la IA puede decir «ese correo ya lo tiene» sin recibirlo.

Es el nivel más bajo que resuelve (ADR-045 §3): una leyenda y no una herramienta. Si la batería muestra que hace falta preguntar más, se mide una herramienta.

### 5. Sin ventana de «datos personales» en el Asistente ni en el Capturista

Lo que la persona escribe va con sus marcas, y nada la detiene. Con eso se van, en el chat:
- la ventana;
- «No es dato personal»;
- `ScannerConfig::cleared` (ADR-041).

**La cuarentena sigue donde un texto se guarda:** las ventanas de «Mi institución», los documentos y Proyectos. Ahí el escáner es del carril `erp`. La conversación del diagnóstico de Proyectos se revisa en MVP-4.

### 6. Dónde vive

- **`ai/vault.rs`, en la base:** marca y resuelve a partir de los hallazgos del escáner, sin saber nada de la institución. `ai::pipeline` y `ai::agent` crean una bóveda por encargo.
- **En `core/agents`:** la sugerencia resuelta antes de `proposals::propose`, la leyenda y la respuesta resuelta para la pantalla.
- **El escáner no cambia en este ADR.** La bóveda usa sus hallazgos tal como son.

## Consecuencias

- **Un falso positivo deja de costar** en el chat y en los datos propios: la persona no se detiene y el dato vuelve a su lugar. En un texto libre que no es dato propio, como la misión, lo sigue cubriendo `proposals::propose`.
- **Un falso negativo sigue siendo el riesgo:** un nombre que el escáner no reconoce sale igual que hoy. La bóveda no lo empeora. Eso lo resuelve el [ADR-046](ADR-046-nombres-con-un-modelo-local.md): los nombres los reconoce un modelo local y se quitan las listas.
- **Los datos propios que cuenta la persona** (su teléfono, su RFC, su representante) ya no necesitan «No es dato personal»: viajan como marca y el código pone el valor.
- **Los oficios cambian:** el del Asistente (v12) y el del Capturista (v6) dicen qué es una marca y que se usa tal cual. Se miden contra los de hoy con el modelo más barato.
- **El principio 5** (`docs/agents/principios-de-ingenieria.md`) y `docs/04-escaner-datos-sensibles.md` dicen cómo sale un dato personal hacia la IA: como marca. Personal y Beneficiarios siguen sin mandar sus fichas.
- **La pantalla del Asistente** deja de recibir la ventana. Su código para ella queda sin usar, y lo quita el carril `erp`.
- **Lo que ya está guardado tapado** (el texto del Expediente) sigue igual: sus etiquetas viajan como están.
