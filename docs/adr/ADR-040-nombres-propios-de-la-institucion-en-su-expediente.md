---
adr: 40
estado: aceptada
fecha: 2026-10-15
tema: [ia, nucleo, privacidad]
reemplaza_en_parte: ["[[ADR-033-nucleo-expediente-y-formularios-universales]]"]
tags: [adr]
---
# ADR-040 · Los nombres de la propia institución en su Expediente sí llegan a la IA

**Estado:** aceptada (2026-10-15, decisión de la institución tras probar C4, fase 1). No cambia código: registra cómo ya funciona y por qué se queda así.

## Contexto

Al probar C4, la institución preguntó al Asistente «Según el registro ante la JAP, ¿quién es el presidente del patronato?». El Lector contestó con el nombre de la persona, citando el documento y la página.

Eso contradecía la decisión 1 del ADR-033: los integrantes del patronato «se guardan con su nombre, sin llegar nunca a la IA». Encontramos dos causas:

1. **Los nombres propios de la institución no se tapan.** `scanner_config` (`core/profile/storage.rs`) pone en `safe_phrases` el nombre de la institución, el del representante legal y los del patronato, para que el escáner no los mande a cuarentena. Con eso tampoco se tapan en el texto del Expediente, y llegan a la IA en los pasajes del Lector.
2. **La lista de nombres del escáner es corta** (unos 245 nombres de pila). Un nombre que no está en ella pasa aunque no sea del patronato.

## Decisión

1. **Los nombres de la propia institución que vienen en sus documentos del Expediente llegan a la IA**: patronato, representante legal, fundadores, notario. La dirección tiene derecho a consultarlos, y el Asistente es su ayudante. No se tapan por su cargo ni se ocultan en la respuesta.
2. **La razón de la regla anterior era el proveedor, no la consulta.** Con una llave de pago, los términos de los proveedores (Gemini y Anthropic) dicen que los datos viajan al servidor para contestar, pero no se usan para entrenar.
3. **Lo que sí es responsabilidad del programa: que no haya fugas de nuestro lado.** Revisado el 2026-10-15:
   - `ai_usage` guarda solo tokens, costo y modelo;
   - la bitácora de cada llamada (`ai.run`) guarda solo conteos, herramientas y resultado, nunca texto;
   - la conversación del Asistente se borra al cerrar la app (ADR-034);
   - la base va cifrada con SQLCipher;
   - las pruebas que leen archivos reales (`reader_on_a_file`, `pdf_report`) no imprimen su texto.

## Lo que no cambia

- **Personal y Beneficiarios** (principio 5, ADR-027 y ADR-029): nada de sus fichas sale hacia la IA, solo conteos. Abrirlo sería otra decisión, con su ADR.
- **La ficha de «Mi institución» que lee la IA** (ADR-023): del patronato lleva cuántas personas y qué cargos, no los nombres; las cuentas, tapadas.
- **El escáner** sigue tapando en el Expediente lo que no es de la institución: CURP, teléfonos y correos ajenos, CLABE ajenas, y nombres de personas que no son de la institución, si los reconoce.

## Condición pendiente

**Mientras la llave sea gratuita**, Google puede usar lo que se le manda para mejorar sus productos. Antes de que una institución use la app con sus documentos reales, su llave debe ser de pago (facturación activa en Google AI Studio, o Anthropic). Hasta entonces, con la llave gratuita se prueba de preferencia con el expediente ficticio de `fixtures/expediente/`.

## Consecuencias

- La decisión 1 del ADR-033 queda así: el patronato se guarda con su nombre; su nombre llega a la IA solo cuando viene en un documento del Expediente que el Lector lee, nunca desde la tabla `core_board_member`.
- `docs/04-escaner-datos-sensibles.md` aclara que `safe_phrases` deja pasar estos nombres también hacia la IA.
