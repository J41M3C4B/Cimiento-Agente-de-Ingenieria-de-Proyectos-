# 04 · Escáner de datos sensibles

Módulo `src-tauri/src/scanner/`. 100 % local, sin red. Es la pieza más importante para la confianza en la herramienta.

## Interfaz

```rust
pub struct Finding {
    pub kind: FindingKind,       // Curp, Rfc, VoterKey, Phone, Email, Clabe, Card, Nss, PersonName, HealthContext
    pub severity: Severity,      // Block | Warn
    pub span: Range<usize>,      // posición en el texto (bytes)
    pub confidence: f32,         // 0.0 a 1.0
    // ¡Nunca guardar el valor encontrado fuera de memoria!
}

pub struct ScanReport { pub findings: Vec<Finding> }

pub trait SensitiveScanner {
    fn scan(&self, text: &str) -> ScanReport;
    fn redact(&self, text: &str, report: &ScanReport) -> String;
}
```

## Patrones (versión 1)

Todos los patrones se aplican sobre el texto normalizado a mayúsculas y sin acentos (manteniendo el mapeo de posiciones al original). Donde existe dígito verificador, **se valida** para reducir falsos positivos.

| Tipo | Patrón base | Validación extra | Severidad |
|---|---|---|---|
| CURP | `\b[A-Z][AEIOUX][A-Z]{2}\d{2}(0[1-9]\|1[0-2])(0[1-9]\|[12]\d\|3[01])[HMX](AS\|BC\|BS\|CC\|CL\|CM\|CS\|CH\|DF\|DG\|GT\|GR\|HG\|JC\|MC\|MN\|MS\|NT\|NL\|OC\|PL\|QT\|QR\|SP\|SL\|SR\|TC\|TS\|TL\|VZ\|YN\|ZS\|NE)[B-DF-HJ-NP-TV-Z]{3}[A-Z\d]\d\b` | Dígito verificador | Block |
| RFC persona física | `\b[A-ZÑ&]{4}\d{6}[A-Z\d]{3}\b` | Fecha válida | Block |
| RFC persona moral | `\b[A-ZÑ&]{3}\d{6}[A-Z\d]{3}\b` | Fecha válida | Warn (puede ser de la institución o un proveedor) |
| Clave de elector | `\b[A-Z]{6}\d{8}[HMX]\d{3}\b` | — | Block |
| Teléfono MX | `(\+?52[\s-]?)?(\(?\d{2,3}\)?[\s-]?)\d{3,4}[\s-]?\d{4}` que sume 10 dígitos | Excluir si coincide con el teléfono institucional registrado | Warn |
| Correo | Patrón estándar de correo | Excluir correo institucional registrado | Warn |
| CLABE | `\b\d{18}\b` | Dígito de control CLABE | Block |
| Tarjeta | 13 a 19 dígitos con separadores opcionales | Algoritmo de Luhn | Block |
| NSS | `\b\d{11}\b` | Dígito verificador (Luhn) | Block |
| Nombre de persona | Heurística (ver abajo) | Contexto | Warn |
| Contexto de salud | Palabras clave cerca de un nombre o pronombre singular | Contexto | Warn |

### Nombres de persona (v1: heurística)

- Lista local de nombres de pila frecuentes en México (en `scanner/data/given_names.txt`, ~2,000) y apellidos frecuentes.
- Coincidencia: secuencia de 2 a 4 palabras capitalizadas donde al menos una es nombre de pila y otra es apellido de la lista.
- Disparadores de contexto que suben la confianza: "la niña", "el señor", "la señora", "residente", "doña", "don", "paciente", "su mamá".
- Lista de exclusión: nombres de la institución, santos y advocaciones en nombres de lugares ("Casa Hogar San José", "Asilo Santa María"), nombres de fundaciones y de sus representantes cuando son públicos.

Versión 2 (Fase 8, opcional): modelo NER local pequeño en español.

### Contexto de salud

Palabras clave: diagnóstico, padece, medicamento, dosis, alzheimer, demencia, diabetes, hipertensión, VIH, epilepsia, discapacidad, psiquiátrico, expediente, alergia, tratamiento. Se marca **solo** si aparecen en una oración junto a un nombre detectado o a un sujeto individual ("la señora de la cama 4", "una de las niñas tiene"). Las frases agregadas ("12 de los 18 residentes viven con diabetes") son válidas y deseables para el proyecto.

## Cuarentena

```
scan() con hallazgos
  → el documento/texto se mantiene en memoria (nunca en disco)
  → la UI muestra un resumen: "Encontramos 3 posibles CURP y 1 teléfono"
     con vista previa del texto TAPADO (no del original)
  → opciones:
     [Tapar y continuar]          → redact() → se guarda la versión tapada
     [Cancelar]                   → se descarta todo
     [Esto no es dato personal]   → solo para severidad Warn; requiere confirmar;
                                     se registra scanner.override en bitácora
```

Los hallazgos `Block` **no se pueden ignorar**. Solo se tapan o se cancela.

Formato de tapado: `[CURP OCULTA]`, `[TELÉFONO OCULTO]`, `[NOMBRE OCULTO]`, etc. El texto mantiene sentido para la IA y para el usuario.

## Documentos de tipo "expediente"

Si un archivo tiene más de N hallazgos `Block` (inicial: 5) o un patrón de tabla con columnas tipo "Nombre / Edad / Diagnóstico", la app lo rechaza completo con el mensaje: *"Este archivo parece una lista de personas. Para cuidarlas, no lo guardamos. Si necesitas esos datos para el proyecto, mejor escribe solo los totales."*

## Pruebas obligatorias

- `fixtures/escaner/positivos.txt`: un caso por línea que **debe** detectarse.
- `fixtures/escaner/negativos.txt`: frases que **no** deben detectarse (cifras agregadas, nombres de instituciones, folios de convocatorias).
- Prueba de que `redact()` nunca deja el valor original en la salida.
- Prueba de rendimiento: 100 páginas de texto en menos de 1 segundo en una laptop modesta.
- Métrica de seguimiento: cualquier falso negativo reportado se convierte en un caso nuevo en `positivos.txt`.

**Todos los datos de prueba son inventados.** Nunca uses una CURP o teléfono reales, aunque sea "de prueba".

## Estado de la implementación (Fase 1)

- Código en `src-tauri/src/scanner/`: `mod.rs` (patrones y tapado), `checks.rs` (dígitos verificadores), `names.rs` (nombres y contexto de salud), `guard.rs` (cuarentena y decisiones), `normalize.rs`.
- Una CURP con forma correcta pero dígito verificador inválido se marca como `Warn` (probable error de captura), no se descarta.
- Los teléfonos pegados a guiones o diagonales (folios, fechas) no se marcan.
- Las listas `scanner/data/given_names.txt` (~245) y `surnames.txt` (~155) son una primera versión, **más cortas que las ~2,000 previstas**. Ampliarlas es trabajo pendiente; cada falso negativo reportado se agrega a `fixtures/escaner/positivos.txt`.
- Rendimiento: ~520 KB (unas 100 páginas) en menos de 1 s incluso en compilación de desarrollo.
- Los datos de contacto institucionales (teléfono, correo, RFC, representante) no se escanean y su teléfono/correo no se marcan en otros textos.
