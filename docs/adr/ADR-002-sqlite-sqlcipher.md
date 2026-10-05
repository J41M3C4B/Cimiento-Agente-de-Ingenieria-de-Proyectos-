# ADR-002 · SQLite + SQLCipher, local primero

**Estado:** Aceptada

## Contexto
Se manejan datos de instituciones que atienden a menores y adultos mayores. Se quiere observar el comportamiento de los datos antes de considerar la nube.

## Decisión
Una sola base SQLite cifrada con SQLCipher (`rusqlite` con `bundled-sqlcipher`), con FTS5 y `sqlite-vec` para búsqueda. Llave en el llavero del SO.

## Consecuencias
- Sin servidor, respaldo = un archivo cifrado.
- La compilación de SQLCipher agrega complejidad al build en Windows: validar en Fase 0.
- Validar que `sqlite-vec` carga sobre SQLCipher; si no, búsqueda solo con FTS5 en v1.
- El acceso a datos pasa por el trait `Repository` para permitir cambiar de motor.

## Resultado de la prueba técnica (Fase 0)
- `rusqlite 0.40` con `bundled-sqlcipher` **no compila** en Windows: exige `OPENSSL_DIR`.
- Funciona con `bundled-sqlcipher-vendored-openssl`, que compila OpenSSL desde fuente. Requiere Strawberry Perl (el `perl` de Git Bash falla por módulos faltantes). Primera compilación ~11 min; después usa caché.
- Llave de 256 bits aleatoria guardada en el llavero de Windows con `keyring-core` + `windows-native-keyring-store` (el crate `keyring` 4.x ya no expone la API de `Entry` directamente).
- Verificado con pruebas: el archivo no empieza con el encabezado de SQLite, no abre sin llave, rechaza llave incorrecta, aplica `0001` una sola vez y FTS5 funciona.
- `sqlite-vec 0.1.9` (crate `sqlite-vec`, registrado con `sqlite3_auto_extension`) carga sobre SQLCipher: tabla `vec0`, búsqueda KNN y persistencia verificadas, y el nombre de la tabla no aparece en claro en el archivo. No hace falta el plan de solo FTS5.
