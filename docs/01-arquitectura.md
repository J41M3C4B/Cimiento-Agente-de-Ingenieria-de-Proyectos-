# 01 · Arquitectura

## Stack

| Capa | Tecnología | Notas |
|---|---|---|
| Contenedor | Tauri 2 | Instalador para Windows (WebView2). Ver ADR-001 |
| UI | React + TypeScript + Vite | |
| Estilos | Tailwind CSS | Componentes accesibles: letra grande, alto contraste |
| Estado UI | Zustand + TanStack Query | Query envuelve los comandos Tauri |
| Formularios | react-hook-form + zod | Los esquemas zod reflejan los tipos de Rust |
| Lógica | Rust | Todo el dominio vive aquí |
| Base de datos | SQLite + SQLCipher vía `rusqlite` (`bundled-sqlcipher`) | Ver ADR-002 |
| Búsqueda | FTS5 (texto) + `sqlite-vec` (semántica) | En la misma base |
| Secretos | `keyring` (llavero del SO) | Llave de la base y llave de API |
| PDF | Evaluar `pdf-extract` / `pdfium-render` en Fase 0 | Solo lectura de texto |
| Excel | `calamine` (lectura rápida) + `umya-spreadsheet` (edición conservando formato) | Validar en Fase 0. Ver ADR-004 |
| Word | Plantillas .docx manipuladas como ZIP + XML (`zip`, `quick-xml`); `docx-rs` para documentos nuevos | Validar en Fase 0 |
| IA | Trait `AiProvider`: Gemini (principal) y Anthropic implementados; Ollama previsto | Ver `07-ia-y-costos.md`, ADR-005 y ADR-007 |
| Pruebas | `cargo test`, Vitest | |

Si en la Fase 0 una librería de Office no conserva formato, se activa el plan B: sidecar en Python (openpyxl, python-docx) solo para ese módulo. Ver ADR-004.

## Capas (puertos y adaptadores)

```
┌─────────────────────────── Frontend (React) ───────────────────────────┐
│ Pantallas · formularios · estado de UI · textos es-MX                  │
└───────────────────────────────┬────────────────────────────────────────┘
                                │ invoke() — comandos Tauri tipados
┌───────────────────────────────▼────────────────────────────────────────┐
│ commands/   capa delgada: valida entrada, llama al dominio, mapea error│
├────────────────────────────────────────────────────────────────────────┤
│ domain/     reglas puras: etapas, cálculos, validaciones, checklist    │
├──────────┬──────────────┬───────────────┬───────────────┬──────────────┤
│ storage/ │ scanner/     │ documents/    │ ai/           │ audit/       │
│ SQLCipher│ patrones,    │ PDF, Excel,   │ AiProvider,   │ bitácora sin │
│ FTS, vec │ cuarentena   │ Word          │ prompts, uso  │ contenido    │
└──────────┴──────────────┴───────────────┴───────────────┴──────────────┘
```

Cada módulo de la fila inferior se expone al dominio mediante un **trait**. Esto permite:

- Cambiar SQLite por un servidor (migración a la nube) sin tocar el dominio.
- Cambiar proveedor de IA (nube ↔ local).
- Probar el dominio con implementaciones falsas.

Traits mínimos:

```rust
trait Repository { /* CRUD por agregado */ }
trait SensitiveScanner { fn scan(&self, text: &str) -> ScanReport; fn redact(&self, text: &str, report: &ScanReport) -> String; }
trait DocumentReader { fn read(&self, path: &Path) -> Result<ExtractedDocument>; }
trait AiProvider { async fn complete(&self, req: AiRequest) -> Result<AiResponse>; }
trait AuditSink { fn record(&self, event: AuditEvent); }
```

## Flujo de un archivo subido

```
Archivo del usuario
  → documents::read (en memoria, sin escribir a disco)
  → scanner::scan
      ├─ limpio → guardar versión extraída + fragmentos + índices
      └─ hallazgos → quarantine (en memoria) → decisión del usuario
            ├─ tapar → scanner::redact → guardar versión tapada
            └─ cancelar → descartar
  → audit::record (tipo de evento, conteos; nunca contenido)
```

## Flujo de una llamada a la IA

```
domain arma la tarea
  → recupera contexto mínimo (FTS / vectores, solo fragmentos relevantes)
  → scanner::scan sobre el prompt completo (segunda barrera)
  → ai::provider.complete (modelo según tipo de tarea)
  → validar respuesta contra esquema JSON
  → guardar con origin = ai_assumption, confirmed_at = NULL
  → registrar tokens y costo estimado en ai_usage
```

## Estructura de carpetas objetivo

```
cimiento/
├── README.md
├── docs/                 # especificaciones; docs/agents/ = guía operativa para agentes de desarrollo
├── schemas/              # esquema canónico de convocatorias
├── fixtures/
├── src/
│   ├── app/              # rutas y layout
│   ├── features/         # una carpeta por etapa: profile, diagnosis, calls, drafting, questionnaires
│   ├── components/       # componentes compartidos
│   ├── lib/tauri.ts      # wrappers tipados de invoke()
│   └── i18n/es-MX.ts     # TODOS los textos visibles
└── src-tauri/
    ├── migrations/
    └── src/
        ├── main.rs
        ├── commands/
        ├── domain/
        ├── storage/
        ├── scanner/
        ├── documents/
        ├── ai/
        │   └── prompts/  # prompts versionados como archivos
        └── audit/
```

## Rutas de datos en la máquina

- Base de datos: carpeta de datos de la app (API de rutas de Tauri), archivo `cimiento.db` cifrado.
- Exportaciones: carpeta que elija el usuario mediante diálogo; nunca automática.
- Sin archivos temporales con contenido en claro. Si una librería exige archivo temporal, se crea dentro de la carpeta de la app y se borra al terminar.

## Migración futura a la nube

El frontend se reutiliza casi completo. Lo que hay que mover:

1. `commands/` → endpoints HTTP.
2. `storage/` → implementación del trait `Repository` para un servidor (p. ej. PostgreSQL).
3. Autenticación y multiusuario (hoy fuera de alcance).

Por eso la regla: **el dominio no conoce Tauri ni SQLite.**
