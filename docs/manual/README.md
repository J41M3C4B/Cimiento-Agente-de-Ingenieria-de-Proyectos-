# Manual del programa (ADR-034 §3)

Lo que la persona lee al tocar el «?» junto a un campo y lo que encuentra al buscar en Ayuda. Funciona sin internet y sin la ayuda automática. Más adelante, el asistente lo consulta para responder.

## Cómo se escribe

- Cada entrada empieza con un encabezado que termina con su clave entre llaves: `## Nombre de la institución {#institution.name}`.
- La clave de un campo es la misma de su formulario en Rust (`core/institution/forms.rs`) y de sus palabras en `src/i18n/es-MX.ts`. Una prueba exige que **cada campo del catálogo tenga su entrada** con «Qué poner:» y «Para qué sirve:», y que ninguna entrada hable de un campo que ya no existe.
- Las pantallas usan la clave `screen.<nombre>`; los formularios, su id (`institution.identity`); las dudas de uso, `howto.<tema>`.
- Cada párrafo es una línea. Se escribe según `docs/08-estilo-redaccion.md`: de usted, frases cortas, nada de jerga. Sin cifras inventadas ni nombres de personas.
- **Qué lo usa** no se escribe aquí: sale de `used_by` del catálogo, para que nunca quede desactualizado.
- Cambiar una pantalla incluye actualizar su entrada.

Este archivo no es parte del manual: el programa solo lee los demás `.md` de esta carpeta que se listan en `src-tauri/src/core/manual/mod.rs`.
