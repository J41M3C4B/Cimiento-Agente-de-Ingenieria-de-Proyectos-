# Datos ficticios

Todo lo que hay en esta carpeta es **inventado**. Sirve para desarrollar y probar sin tocar información real.

| Archivo | Uso |
|---|---|
| `institucion-asilo.json` | Perfil ficticio de un asilo |
| `institucion-casa-hogar.json` | Perfil ficticio de una casa hogar de niñas |
| `caso-dorado-bano.md` | Caso de referencia del diagnóstico: entrada y resultado esperado |
| `escaner/positivos.txt` | Textos que el escáner debe detectar (formato `tipo ¦ texto` con barra vertical) |
| `escaner/negativos.txt` | Textos que el escáner NO debe detectar |
| `escaner/generar.py` | Genera los dos anteriores con dígitos verificadores válidos |
| `office/` | Excel, Word y PDF de prueba (con tablas y escaneado). Se regeneran con `python generar.py` |
| `convocatoria-ficticia-paginas.txt` | Convocatoria inventada, ya en texto por páginas (`=== PAGINA n ===`), con las mismas manchas que dejan los PDF reales (ligaduras, letras mal decodificadas, enlaces cortados, matrices con «Sí» sueltos). Alimenta las pruebas del extractor de convocatorias |

Pendientes (crear en las fases indicadas):

- Fase 3: `convocatoria-ficticia.pdf` imitando la estructura pública de convocatorias reales.

Reglas: nunca agregar datos reales aquí, ni "de prueba". Las CURP, CLABE y NSS de los casos de prueba tienen formato válido pero son inventadas.

- `padron-asilo.json` y `padron-casa-hogar.json`: personas ficticias (personal y beneficiarios) que cargan los botones de ejemplo de «Mi institución» (solo en desarrollo). El perfil de `institucion-*.json` ya no trae personal ni grupos: se calculan desde el padrón (ADR-020).
