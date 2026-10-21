---
adr: 2
estado: aceptada
fecha: 2026-10-20
tema: [erp, nucleo, modulos, datos, arquitectura]
bloque: ERP-7
tags: [adr]
---
# ADR-002 · El núcleo del ERP y la forma de un módulo

**Estado:** aceptada (2026-10-20). El líder aprobó el plano y sus seis decisiones, y validó el bloque ERP-7.

## Contexto

Los [planos](../planos-de-sociai.md) dicen qué partes tiene SociAI y cómo pasa una petición, pero no cómo se arma por dentro el ERP: qué es común a todos los módulos, qué forma tiene un módulo y cómo se reparten los datos entre ellos.

En el código de hoy, `core/` es Mi institución más Inicio, y está **encima** de los módulos: los lee por su `api`, sus `bridge` y sus `insights`. Cada módulo nuevo hace crecer el núcleo, y cada módulo tiene su propia forma. Es la filosofía que no funcionó.

Este ADR fija la arquitectura del ERP desde los planos, sin acomodar lo que ya existe. El detalle, con sus diagramas, está en [el plano del ERP](../plano-del-erp.md).

## Decisión

### 1. Mi institución es un módulo; el núcleo va aparte (D1)

- **El núcleo** es lo que todos los módulos necesitan y ninguno posee:
  - K1, la organización;
  - K2, las cuentas, los roles y los permisos;
  - K3, el diccionario de datos con la clase de cada dato;
  - K4, el almacenamiento y la unidad de trabajo;
  - K5, el historial y el origen;
  - K6, los documentos;
  - K7, el directorio de personas y organizaciones;
  - K8, el calendario, los vencimientos y los eventos;
  - K9, el almacén de la auditoría;
  - K10, la búsqueda;
  - K11, los catálogos de referencia;
  - K12, el registro de módulos.
- **El núcleo está debajo de los módulos y no conoce a ninguno.** Lo que necesita de ellos se lo declaran.
- **Mi institución es el primer módulo.** Con él se prueba el núcleo.

### 2. Todos los módulos tienen la misma forma

Un módulo tiene `manifest`, `contract`, `domain`, `catalog`, `storage`, `migrations`, `service`, `capabilities` y `events` (plano, §4). Una **prueba de conformidad** corre sobre cada módulo registrado, y un módulo que no cumple no entra al registro. Un módulo nuevo es llenar esa forma: ni el núcleo ni la plataforma cambian.

### 3. Los datos entre módulos

- **Hacia adentro,** cada módulo es dueño de sus tablas, con su prefijo, y solo él escribe SQL sobre ellas.
- **Hacia afuera** hay cuatro puertas: sus capacidades, sus tipos publicados, sus hechos y su parte del diccionario.
- **Un dato vive en un solo lugar.** Otro módulo guarda la referencia, no una copia, y sin llaves foráneas entre módulos.
- **Una vista de varios módulos** es una capacidad que combina otras, nunca SQL que cruce tablas.

### 4. Todo cambio se prepara y se aplica

Una modificación, venga de una persona o de la plataforma:
1. se **prepara**: una propuesta del ERP con las diferencias, la validación, la versión base y una huella;
2. se **aprueba**, si el contrato lo pide;
3. se **aplica** en una sola transacción del núcleo, que escribe el dato, su historial y origen, su hecho y el registro del cambio. Antes revisa que la versión y la huella no hayan cambiado.

Sustituye a las propuestas por campo (`ai_proposal`) y cumple la §3.5 de los planos sin que el ERP conozca la IA.

### 5. Las demás decisiones

| | Decisión |
|---|---|
| **D2** | Un **directorio común** de personas y organizaciones en el núcleo, con una ficha mínima. Lo propio de cada papel (donante, beneficiario, empleado) vive en su módulo. Cada rol ve solo los papeles de sus módulos. |
| **D3** | **El dinero en centavos**, con su moneda, en todo el ERP. |
| **D4** | **Una base cifrada por institución,** con la organización explícita en cada petición. |
| **D5** | **Un *workspace* de Cargo:** el núcleo y cada módulo nuevo son un *crate*, y una dependencia prohibida no compila. El código viejo se queda en la app hasta retirarse. |
| **D6** | **Las pantallas no tienen comandos por módulo:** todo pasa por el runtime, también lo determinista. Solo quedan la entrada al runtime y lo de antes de la sesión. |

## Consecuencias

- **El orden del riel ERP** es el de la §11 del plano: la forma de un módulo (ERP-8), preparar y aplicar (ERP-9), el diccionario (ERP-2), Mi institución sobre el núcleo (ERP-1), documentos, calendario (ERP-3) y directorio.
- **El IA-4 espera más.** Depende del ERP-1, y el ERP-1 ahora se construye sobre el núcleo en lugar de envolver el código viejo.
- **Se pide al riel IA** que el contrato de capacidad se pueda usar desde un *crate* de módulo, con los permisos que declara el módulo (IA-5). Con el ERP-2 se le pedirá que «falta información de la persona» nombre campos del diccionario.
- **El código de hoy se depura** con la §10 del plano. `core/bridge`, `core/insights`, `core/overview.rs`, `core/snapshot.rs`, `core/modules.rs`, `core/proposals.rs` y `core/reader` se retiran cuando su sustituto exista.
- **Las migraciones pasan a ser por módulo.** Las 34 de hoy quedan como la historia de la app: una base existente debe seguir abriendo.
- **El *workspace* cambia la compilación de las tres carpetas.** Se avisa a los otros rieles en el PR del ERP-8.

## Alternativas descartadas

- **Mi institución como núcleo.** Cada módulo dependería de lo de adentro de Mi institución, y el núcleo crecería con cada módulo. Es la forma de hoy.
- **Cada módulo con sus propias personas.** Es la doble captura, una de las primeras causas de abandono de un sistema ([investigación](../investigacion/investigacion-externa.md), §1.4).
- **Seguir con una sola *crate* y una prueba que lee el texto.** Funciona, pero se puede burlar y no escala con muchos módulos.
