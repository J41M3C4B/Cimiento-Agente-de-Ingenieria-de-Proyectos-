# Principios de ingeniería

Lee este documento completo antes de cualquier tarea. Para el detalle, consulta `docs/`.

## Qué es el proyecto

App de escritorio (Tauri 2) local y con IA para que asilos y casas hogar diagnostiquen necesidades y armen proyectos para solicitar donativos. Usuarios finales: directivos y religiosas **sin perfil técnico**. Visión completa en `docs/00-vision.md`.

## Principios que NO se negocian

1. **Determinista primero.** Si algo se puede resolver con una regla, va en código. La IA solo hace: preguntas de diagnóstico, razonamiento sobre necesidades, redacción y resumen. Ver `docs/07-ia-y-costos.md`.
2. **La IA nunca calcula.** Totales, porcentajes, presupuestos y validaciones se hacen en Rust/TypeScript.
3. **La IA nunca escribe archivos.** La IA devuelve JSON estructurado; el código valida y escribe en Word/Excel.
4. **Todo dato lleva origen.** Campos `origin` (`user`, `document`, `ai_assumption`, `computed`) y `confirmed_at`. Ver `docs/05-modelo-datos.md`.
5. **Cuántos, nunca quiénes, para la IA y los documentos.** El perfil, la IA, el escáner y los documentos solo manejan agregados. Las fichas individuales viven aparte, en sus módulos de este equipo (personal: ADR-027; beneficiarios: ADR-029), y nada de ellas sale hacia la IA: solo conteos, y los atributos personales solo para grupos de 3 o más. Todo texto/archivo pasa por el escáner antes de guardarse y antes de ir a la IA. Ver `docs/03-gobernanza-datos.md`, `docs/04-escaner-datos-sensibles.md`, `docs/adr/ADR-027-modulo-de-personal-base-de-rh.md` y `docs/adr/ADR-029-modulo-de-beneficiarios-e-inteligencia-de-datos.md`.
6. **El original nunca se modifica.** Los archivos subidos se procesan; se guarda solo la versión limpia. Las exportaciones son copias nuevas.
7. **Lenguaje sencillo en la UI.** Todo texto visible sigue `docs/08-estilo-redaccion.md`. Nada de jerga técnica en mensajes al usuario.
8. **Dos capas: lo que consume la IA no es lo que ve la persona.** El esquema de la convocatoria, la ficha de «Mi institución» para el prompt y los JSON de la IA son de la IA. La persona ve una **ficha** corta en sus palabras (`…Card`, compuesta en Rust, no por la pantalla); la estructura completa solo se ofrece como «Consultar el detalle». Antes de diseñar una pantalla pregunta: ¿esto es para la IA o para la persona? Ver `docs/adr/ADR-024-dos-capas-ia-y-persona.md`.

## Stack

- Tauri 2 (Rust) + React + TypeScript + Vite + Tailwind CSS
- SQLite con SQLCipher (`rusqlite` con feature `bundled-sqlcipher`), FTS5 y `sqlite-vec`
- Llave de cifrado en el llavero del sistema operativo (crate `keyring`)
- IA detrás de un trait `AiProvider` (Gemini como proveedor principal y Anthropic, ya implementados; Ollama previsto, ADR-005 y ADR-007)
- Gestor de paquetes JS: pnpm

Usa siempre la **última versión estable** de cada dependencia y verifica la documentación oficial antes de asumir APIs. Si una librería no cumple lo esperado, documéntalo en un ADR nuevo en `docs/adr/`.

## Estructura

```
src/                 # Frontend React (solo UI y estado de pantalla)
src-tauri/src/
  commands/          # Comandos Tauri: capa delgada, sin lógica de negocio
  domain/            # Tipos y reglas de negocio puras (sin E/S)
  storage/           # Base de datos, migraciones, cifrado
  scanner/           # Detección y tapado de datos sensibles
  documents/         # Lectura PDF/Excel/Word, plantillas, exportación
  ai/                # Trait AiProvider, prompts, registro de uso
  audit/             # Bitácora
  hr/                # Módulo de Personal (ADR-027): aparte, tablas hr_*, solo agregados hacia fuera
  care/              # Módulo de Beneficiarios (ADR-029): aparte, tablas care_*, solo agregados hacia fuera
  facilities/        # Módulo de Instalaciones (ADR-030): aparte, tablas fac_*; inmueble, espacios y equipo por grupo
  common/            # Validadores compartidos por los módulos (CURP, RFC, NSS, CLABE, fechas); no depende de nada
docs/                # Especificaciones (fuente de verdad)
fixtures/            # Datos ficticios
schemas/             # Esquema canónico de convocatorias (JSON Schema)
```

Detalle en `docs/01-arquitectura.md`.

## Reglas de código

- **Idioma:** identificadores, comentarios de código y commits en inglés. Textos de UI en español (archivo `src/i18n/es-MX.ts`). Documentación en español. Glosario en `glosario-dominio.md`.
- **Toda la lógica de negocio vive en Rust** (`domain/`). El frontend no decide reglas, solo muestra y captura.
- **Llamadas a la IA solo desde Rust.** La llave de API nunca llega al frontend.
- **Cada llamada a la IA** pasa por: escáner → armado de prompt → proveedor → validación de JSON contra esquema → registro en `ai_usage`.
- **Errores:** en Rust usa `thiserror` en módulos y convierte a un error serializable para la UI con un mensaje amigable en español + código interno.
- **Pruebas obligatorias** para: escáner (cada patrón, con positivos y negativos), cálculos de presupuesto, transiciones de etapa, lectura/escritura de Excel y Word con fixtures.
- **Migraciones** numeradas y nunca editadas una vez aplicadas.
- No agregues dependencias pesadas sin justificarlo en el PR/commit.
