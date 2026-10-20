---
tipo: bloque
id: IA-9
carril: ia
estado: listo
prioridad: alta
fase: F1
patron: [determinista]
rama:
adr: "[[ADR-048-el-flujo-de-datos]]"
creado: 2026-10-20
cerrado:
---

## Qué y por qué

Fase F1 del [[ADR-048-el-flujo-de-datos|ADR-048]]. Hoy cuatro piezas deciden por su cuenta qué campos ve la IA, y la que se quedó atrás falló: el Asistente mandó la representante y la calle a otra ventana (prueba de IA-2, 2026-10-20). Cada campo debe declararse una sola vez, y todas las piezas deben leer esa declaración.

Junta [[IA-3 La ficha de la IA sin los módulos congelados|IA-3]] y [[IA-7 La IA ante el núcleo genérico|IA-7]]. El catálogo de campos es del ERP, pero el líder decidió que se haga aquí, en un solo bloque, y que se le avise al ERP.

## Terminado cuando

(Aprobado por el líder en el ADR-048, 2026-10-20.)

- [ ] Cada campo de «Mi institución» declara una sola vez cómo llega a la IA:
  - **tal cual:** lo de la institución (RFC, domicilio, teléfono, correo);
  - **marca:** los nombres de personas, también la representante;
  - **no sale:** cuentas de banco, contraseñas y salud.
- [ ] Todas las piezas leen esa declaración: el mapa del Asistente, el Capturista, los Lectores, el guardado de sugerencias y la ficha de la IA. Ya no existen `open_fields`, `told_fields`, `own`, `told_own` ni `allowed`.
- [ ] Una prueba recorre todos los campos y exige que cada pieza vea cada uno según su declaración.
- [ ] La ficha de la IA sale del modelo de datos: sin lo de los módulos ocultos (IA-3) y con el núcleo genérico (IA-7).
- [ ] Una batería generada del modelo de datos: por cada campo que la IA puede tomar, alguien que cuenta ese dato, y el Asistente ofrece su ventana. Se mide en seco y en real.
- [ ] En la app, el correo y la representante van a su ventana. La calle queda para F2: el escáner la marca entera como un nombre.
- [ ] `cargo test` completo pasa, y se le avisa al ERP en su tablero.

## Plan

## Cómo probarlo

- [ ] 

## Hallazgos

## Resultado
