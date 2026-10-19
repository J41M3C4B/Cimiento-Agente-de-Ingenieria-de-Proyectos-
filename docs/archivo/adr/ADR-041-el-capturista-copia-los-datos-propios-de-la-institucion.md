---
adr: 41
estado: aceptada
fecha: 2026-10-15
tema: [ia, nucleo, privacidad]
reemplaza_en_parte: ["[[ADR-033-nucleo-expediente-y-formularios-universales]]"]
tags: [adr]
---
# ADR-041 · «Llenarlo por mí» también sugiere los datos propios de la institución, copiados de las palabras de la persona

**Estado:** aceptada (2026-10-15, decisión de la institución al probar C4, fase 2). Aplicada en el Capturista.

## Contexto

Al probar «Llenarlo por mí», la institución escribió su domicilio completo (ficticio): calle, número, colonia, municipio, estado y código postal. El Capturista solo dejó el municipio y el estado.

No fue el modelo. El ADR-033 §3 marcó como «nunca a la IA» los datos propios y protegidos de la institución: teléfono, correo, calle, número, colonia, código postal, RFC, folios, claves y representante legal. El Capturista tenía prohibido sugerirlos, y el código los tiraba aunque la persona los hubiera escrito.

Siguiendo el razonamiento del ADR-040 (son datos de la institución, no de personas, y con llave de pago el proveedor no entrena con ellos), la institución eligió que se sugieran **todos**, revisados por el código.

## Decisión

1. **El Capturista puede sugerir los datos propios de la institución que la persona le cuenta en el chat** (`capturist::told_fields`): los campos que la IA puede ver más los `InstitutionalPrivate` de la ventana.
2. **Se copian, nunca se hacen.** El código revisa, además de la cita (`reader::quote`):
   - que las letras y los dígitos del valor estén en las palabras de la persona, uno por uno, sin espacios ni signos («33 1234 5678» = «3312345678»). Un dígito cambiado, un valor completado o adivinado se tira;
   - que una fecha sea una que la persona escribió («18 de enero de 2027»), leída por `reader::dates`. Esto vale para toda fecha que sugiera el Capturista;
   - que el valor quepa en su campo (la forma del RFC, del código postal), como siempre (`proposals::propose`).
3. **Lo guardado de esos campos nunca llega a la IA:** en el contexto solo dice «lleno» o «vacío».
4. **El valor sugerido no se tapa** (como lo leído por las reglas de un documento): es de la institución. Una vez que la persona decide, se olvida (`forget_protected`), como ya pasaba.
5. **Solo el Capturista.** Ningún otro agente de IA puede proponerlos (`not_for_the_ai`). La ficha que lee la IA (ADR-023), «Llenar con mis documentos» (C2) y el Asistente no cambian: esos datos solo los leen las reglas de los documentos.
6. **El escáner sigue preguntando.** Un teléfono o un RFC escritos en el chat abren la ventana de «¿es dato personal?», como cualquier texto (principio 5):
   - si la persona contesta «Taparlo», la IA no los ve;
   - si contesta «No es dato personal», el Capturista puede copiarlos. Hasta ahora la última revisión antes de la IA volvía a taparlos (medido: el modelo recibió «[RFC OCULTO]»). Ahora esa revisión recibe lo que la persona aclaró (`ScannerConfig::cleared`, `core::screen::scanner_clearing`) y no lo vuelve a tapar: solo ese texto exacto, solo en ese encargo y solo si era una advertencia.

   Un dato que el escáner bloquea (una CURP, el RFC de una persona) se tapa siempre, aunque esté en la lista.

## Consecuencias

- Oficio del Capturista v4: copiar tal cual, sin completar ni corregir; una fecha solo si la dijo completa.
- La batería (`fixtures/capturista.json`) espera ahora el domicilio, el teléfono y el RFC. Tiene trampas de un dígito cambiado y de un valor que no está en la cita, y un caso con el domicilio completo.
- «Todavía puede contarme» incluye ahora los datos propios vacíos de la ventana.
- **Medido con IA real** (2026-10-15, oficio v4, 15 casos, dos rondas, `gemini-3.5-flash-lite`): 29 de 30 casos y 187 de 188 campos bien, **nada inventado**; 30 llamadas, **$0.56 MXN**. El RFC, el teléfono y el domicilio completo llegaron al modelo sin tapar (punto 6) y se sugirieron copiados. El único fallo: una ronda no propuso «desde dos mil diez» como año de fundación (no inventó; faltó). Una primera corrida se quedó sin cuota diaria de la llave gratuita en el sexto caso; en esos casos el RFC llegaba tapado, y eso llevó al punto 6.
- A veces el modelo deja la palabra «Calle», «Colonia» o «#» dentro del valor («Calle Volcanes», «#412»); otras veces no. Se copia tal cual de lo que dijo la persona y ella lo revisa antes de usarlo; no se corrige.
- El Asistente tiene el mismo comportamiento ante «No es dato personal»: la última revisión vuelve a tapar lo aclarado. Se deja como está mientras no haga falta; usar `scanner_clearing` ahí sería otra decisión.
