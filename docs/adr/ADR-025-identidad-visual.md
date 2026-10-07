# ADR-025 · Identidad visual: bandejas sobre el fondo de la ventana, color con significado y tokens como única fuente

**Estado:** aceptada.

## Contexto

La interfaz anterior (tinta, un azul, hoja blanca, Inter) era correcta pero genérica, y el color no ayudaba a las personas a ubicarse. La institución eligió unas referencias (tablero de bandejas blancas sobre un marco claro, color sólido y tranquilo, pestañas de carpeta) y se validó un prototipo navegable de todas las pantallas. Además, la interfaz había acumulado valores sueltos (`text-[13px]`, hexadecimales, tres alturas de campo) que hacían imposible mantenerla homogénea.

## Decisión

1. Se adopta el sistema descrito en `docs/13-sistema-visual.md`: riel de íconos y cápsula superior con bandejas de 28 px directamente sobre el fondo de la ventana (sin marco ni hoja intermedia, que parecía una aplicación dentro de otra), ocho acentos sólidos iguales en claro y oscuro con letra negra, «un proyecto, un color» y Plus Jakarta Sans.
2. **Los tokens son la única fuente de verdad** y viven en `src/index.css` en tres niveles (primitivos, semánticos, de componente). Se vacían las escalas por defecto de Tailwind (colores, tamaños de letra, radios, sombras) para que solo existan los tokens del sistema.
3. **Las pantallas componen componentes**, que viven en `src/components/ui/`. Cada componente tiene una sola medida (tabla 7.1 del documento).
4. **La regla se hace cumplir con una prueba** (`src/design.test.ts`) que falla si una pantalla usa colores sueltos, tamaños arbitrarios o las escalas de Tailwind eliminadas.
5. El color de proyecto sigue guardándose con los nombres anteriores (`blue`, `pink`, `orange`…); una sola tabla en `src/lib/palette.ts` los traduce a los acentos nuevos. No se toca la base ni el backend.
6. Modo claro y oscuro con los mismos tokens: sigue al sistema y se puede forzar desde la barra superior.

## Consecuencias

- Cambiar un radio, una altura o un color es editar un token; la interfaz entera lo toma.
- Escribir una pantalla es componer; añadir una variación obliga a pasar por el documento, que es el punto.
- Costo: una vez migrada, toda interfaz nueva debe encajar en el catálogo; lo que no encaje abre una discusión de diseño en lugar de un estilo suelto.
- La fuente (`@fontsource-variable/plus-jakarta-sans`) se empaqueta con la app: funciona sin red.

## Alternativas descartadas

- **Solo cambiar valores de color:** deja la estructura genérica y sin garantía de homogeneidad.
- **Paleta «profunda» con letra blanca:** obligaba a oscurecer los colores; el amarillo se volvía cobre y se sentía pesado.
- **Renombrar los colores guardados de proyecto:** exigía migración de base y de `src-tauri` por una cuestión de nombres.
