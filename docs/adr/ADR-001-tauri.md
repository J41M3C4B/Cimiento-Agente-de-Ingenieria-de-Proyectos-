# ADR-001 · Tauri 2 como contenedor de escritorio

**Estado:** Aceptada

## Contexto
La app debe instalarse localmente en computadoras modestas de las instituciones, ser segura por defecto y permitir una migración futura a web.

## Decisión
Tauri 2 con frontend React + TypeScript y lógica en Rust.

## Consecuencias
- Instalador ligero, usa WebView2 en Windows.
- Modelo de permisos explícito: solo se habilitan los plugins necesarios (diálogo de archivos, rutas de la app).
- El frontend es reutilizable en web. La lógica en Rust **no** se migra sola: se mitiga manteniendo `domain/` independiente de Tauri y `commands/` como capa delgada.
