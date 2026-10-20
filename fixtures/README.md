# Datos ficticios

Todo lo que hay en esta carpeta es **inventado**. Sirve para desarrollar y probar sin tocar información real.

| Archivo | Uso |
|---|---|
| `institucion-asilo.json` | Perfil ficticio de un asilo, I.A.P., con todos los datos de «Mi institución», su patronato (`board`) y su cuenta (`bank_accounts`) |
| `institucion-casa-hogar.json` | Perfil ficticio de una casa hogar de niñas, I.A.P., con los mismos datos |
| `institucion-red-comunitaria.json` | Perfil ficticio de una A.C. que no es residencia (talleres, despensas y becas en su colonia; sin Junta ni capacidad), con los mismos datos. Con ella se prueba que el núcleo sirve a cualquier organización (ERP-2) |
| `caso-dorado-bano.md` | Caso de referencia del diagnóstico: entrada y resultado esperado |
| `escaner/positivos.txt` | Textos que el escáner debe detectar (formato `tipo ¦ texto` con barra vertical) |
| `escaner/negativos.txt` | Textos que el escáner NO debe detectar |
| `escaner/generar.py` | Genera los dos anteriores con dígitos verificadores válidos |
| `office/` | Excel, Word y PDF de prueba (con tablas y escaneado). Se regeneran con `python generar.py` |
| `expediente/` | Expediente inventado de una casa de reposo (acta, poder, constancia fiscal, oficio de donataria, reglamento, aviso sanitario, estados financieros, informe, comodato, protección civil), en texto por páginas (`=== PAGINA n ===`), y `preguntas.json`: preguntas de referencia con la página o la entrada del manual que las responde. Mide la búsqueda (IA5) |
| `expediente/prueba-completa/` | La prueba completa del núcleo: los textos de 5 documentos más (reformas, licencia de uso de suelo, aviso de privacidad, presupuesto ante la Junta; el organigrama se dibuja), `generar.py`, que hace 24 archivos como los que sube una institución (PDF con texto, escaneos, una hoja girada, una copia desteñida, fotos de celular, una captura con letra chica, Word con y sin hoja escaneada, una página mixta, y con fallas: contraseña, dañado, tipo que no se usa, en el lugar equivocado, sin texto), y `esperado.json`, lo que debe pasar con cada uno (`cargo test full_expediente -- --ignored --nocapture`). En palabras: `docs/16-institucion-de-prueba.md` |
| `capturista.json` | Batería del Capturista (C4): lo que cuenta una persona en el chat y lo que debe quedar sugerido en cada ventana de «Mi institución», con trampas que el código debe tirar |
| `convocatoria-ficticia-paginas.txt` | Convocatoria inventada, ya en texto por páginas (`=== PAGINA n ===`), con las mismas manchas que dejan los PDF reales (ligaduras, letras mal decodificadas, enlaces cortados, matrices con «Sí» sueltos). Alimenta las pruebas del extractor de convocatorias |

Pendientes (crear en las fases indicadas):

- Fase 3: `convocatoria-ficticia.pdf` imitando la estructura pública de convocatorias reales.

Reglas: nunca agregar datos reales aquí, ni "de prueba". Las CURP, CLABE y NSS de los casos de prueba tienen formato válido pero son inventadas.

- `instalaciones-asilo.json`, `instalaciones-casa-hogar.json` e `instalaciones-red-comunitaria.json` (solo una oficina rentada): inmueble, espacios y equipo ficticios (ADR-030) que cargan los mismos botones de ejemplo. El perfil de `institucion-*.json` ya no trae instalaciones.
- `padron-asilo.json` y `padron-casa-hogar.json` (el de la red comunitaria va vacío): personas ficticias (personal y beneficiarios) que cargan los botones de ejemplo de «Mi institución» (solo en desarrollo). El perfil de `institucion-*.json` ya no trae personal ni grupos: se calculan desde el padrón (ADR-020).
