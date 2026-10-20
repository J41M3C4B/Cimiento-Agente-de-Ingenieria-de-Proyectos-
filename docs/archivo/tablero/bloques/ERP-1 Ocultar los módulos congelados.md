---
tipo: bloque
id: ERP-1
carril: erp
estado: hecho
prioridad: alta
fase: MVP-1
rama: erp/erp-1-ocultar-modulos (fusionada en el PR #7 y borrada)
adr: "[[ADR-043-un-erp-con-la-ia-en-el-nucleo]]"
creado: 2026-10-18
cerrado: 2026-10-19
---

## Qué y por qué

Personal, Beneficiarios, Instalaciones, Finanzas y Donantes están congelados hasta validar el MVP (ADR-043 §3), pero la app los sigue mostrando. Hay que dejar la app enfocada en el núcleo y Proyectos sin borrar su código.

## Terminado cuando

(Aprobado por el líder, 2026-10-18.)

- [x] El riel, «Mi institución», el primer inicio (incluida la bienvenida), Inicio y el mapa de lugares del Asistente solo muestran el núcleo y Proyectos.
- [x] Nada del núcleo lee un módulo oculto para mostrarlo: lo que tomaba de ellos (ocupación, cifras rápidas, balance) se lo queda el núcleo o deja de mostrarse.
- [x] El código de los módulos compila y todas las pruebas pasan, también `cargo test architecture`.
- [x] Encenderlos de nuevo es un solo cambio, documentado.

## Plan

**Un solo interruptor, en Rust.** Una lista nueva (`core/modules.rs`) dice qué páginas de módulo están ocultas: `staff`, `people`, `facilities`, `finance` (Donantes no existe todavía). Todo lo demás se deduce de ella; volver a encender un módulo es quitarlo de la lista.

1. **Mapa de lugares y Asistente:** una página oculta no está en el mapa (`core/places.rs`). El Asistente no la nombra, no ofrece su botón y su búsqueda en el manual salta las entradas de esas pantallas. Las entradas siguen en `docs/manual/`.
2. **Resumen de la institución** (`core/overview.rs`): no lee los módulos ocultos.
   - Las personas atendidas y el personal salen de las cifras rápidas del núcleo, sin «≈».
   - Espacios y balance no se calculan.
3. **El riel** muestra solo los módulos que están en el mapa: Inicio, Mi institución, Documentos y Proyectos.
4. **Mi institución** *(decisión del líder)*: las cuatro tarjetas de módulo pasan a dos, con las cifras rápidas del núcleo: «Personas que atienden» (con su capacidad) y «Personal». Las dos abren «Capacidad y cifras rápidas». Instalaciones y Finanzas se van.
5. **Inicio** *(decisión del líder)*:
   - la tarjeta de personas se queda, con la cifra del núcleo; abre «Capacidad y cifras rápidas» y pierde «entraron o salieron este año», que es de Beneficiarios;
   - la tarjeta del balance se va;
   - «Siguientes pasos» pierde los cuatro pasos de módulos.
6. **Bienvenida del primer inicio** *(decisión del líder)*: el dibujo de módulos muestra solo Proyectos y Documentos alrededor de «Mi institución».
7. **Pruebas:** las del mapa contra el riel, las del resumen, las de pantalla que cambian, y una nueva que revisa que ninguna página oculta llegue al riel, al mapa ni al Asistente. El código de los módulos sigue compilando y sus pruebas siguen corriendo.

**Riesgos:**
- Los datos que una institución ya tenga en un módulo se quedan guardados y no se ven, sin borrarse.
- El diseño fino de las dos tarjetas y de la bienvenida queda para un bloque de diseño.

## Cómo probarlo

Tu `pnpm tauri dev` ya muestra estos cambios, porque la carpeta está en la rama `erp/erp-1-ocultar-modulos`. Si lo cierras y lo vuelves a abrir, la primera vez compila un rato.

- [x] **La barra de arriba** solo tiene Inicio, Mi institución, Documentos y Mis proyectos.
- [x] **Inicio:** la tarjeta «Personas que atendemos» muestra la cifra rápida, los lugares vacíos y el % ocupado, sin la flecha de «entraron o salieron». Al pulsarla se abre «Capacidad y cifras rápidas». No hay tarjeta de balance, y «Siguientes pasos» no ofrece Personal, Beneficiarios, Instalaciones ni Finanzas.
- [x] **Mi institución:** a la derecha hay dos tarjetas, «Personas que atienden» (con «de N lugares») y «Personal», sin «≈». Las dos abren «Capacidad y cifras rápidas». No hay tarjetas de Instalaciones ni de Finanzas.
- [x] **Cambiar una cifra rápida** (por ejemplo, las personas que atienden) y guardar: la tarjeta de Inicio y la de Mi institución muestran la cifra nueva.
- [x] **El Asistente:** pregúntale «¿Dónde registro los ingresos?» o «¿Dónde doy de alta a una persona del personal?». No debe ofrecer un botón a Finanzas ni a Personal, ni mandarte a esas páginas.
- [x] **La bienvenida**, con una cuenta nueva que no la haya visto: alrededor de «Mi institución» solo aparecen Proyectos y Documentos.

**Revisado por el CTO en la app real** (copia aparte con el ejemplo del asilo):
- la barra, Inicio, las dos tarjetas de Mi institución y la ventana que abren;
- que la búsqueda del manual y el Asistente no respondan con las pantallas ocultas.

La bienvenida solo quedó revisada en las pruebas automáticas.

**Lo que faltó (tu observación, 2026-10-19):**
- [x] **El Asistente y lo que todavía no existe:** cierra y vuelve a abrir `pnpm tauri dev` y pregúntale «¿Dónde registro los ingresos?». Debe decir que esa parte llegará después, sin ningún botón. Pregúntale también «¿Cuántos extintores tenemos?»: debe contestar con el programa de protección civil.

## Resultado
 Nos dio un comportamiento raro al preguntarle donde registro los ingresos:
 ![[{1C46CBA4-3E8C-41B9-B46D-B58918C02076}.png]]

**Respuesta del CTO (2026-10-19).**

- **La causa:** al Asistente se le quitaron las páginas ocultas, pero nadie le dijo que esas partes no existen todavía. Ante «los ingresos», eligió la ventana que más se parecía.
- **El arreglo (oficio v11):** el código le dice qué partes todavía no existen y que en esos casos no ofrezca ningún lugar. La lista sale del mismo interruptor (`core/modules.rs`): encender un módulo sigue siendo un solo cambio.
- **Lo que enseñó la medición:**
  - Una versión que además le decía qué guarda cada parte («su equipo») contestó «¿Cuántos extintores tenemos?» con «esa parte no existe», aunque el Expediente lo dice. Por eso solo lleva los nombres.
  - La batería ganó tres preguntas de partes que no existen: ingresos, gastos e inmueble. Si el Asistente ofrece cualquier lugar, la pregunta sale mal.
- **Corrida real** (modelo más barato, dos rondas, a lo más 14 llamadas por minuto): **40 de 40**, citas 18 de 18 y nada inventado.
  - Gasto: 63 llamadas del Asistente, 18 del Lector y 19 solicitudes de vectores, **$1.40 MXN** en 5.6 minutos.
  - Antes (oficio v10): 33 de 34.

**Validado por el líder (2026-10-19):** probó la casilla nueva en la app y salió bien. Se fusionó a `main` en el PR #7.
- Quedó una respuesta ambigua, que no es un error del programa. A «¿Cuántos extintores tenemos?» contestó: «Ninguno de los documentos que tiene guardados dice cuántos extintores tienen. Puede revisar en sus instalaciones o anotar ese dato cuando agreguemos la sección correspondiente».
- «Sus instalaciones» se lee como el edificio, sin «» que lo marque como una parte del programa, y «la sección correspondiente» habla de una parte que todavía no existe.
- Se anotó en [[IA-4 El Asistente decide y habla natural|IA-4]], que trata cómo habla el Asistente.