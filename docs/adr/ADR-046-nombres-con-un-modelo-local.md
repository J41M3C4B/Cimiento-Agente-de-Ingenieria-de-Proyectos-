---
adr: 46
estado: propuesta
fecha: 2026-10-19
tema: [ia, privacidad, escaner]
tags: [adr]
---
# ADR-046 · Los nombres de personas los reconoce un modelo local, no una lista

**Estado:** propuesta (2026-10-19). Aplica el ADR-043 §5 (ninguna regla por caso) y la prueba de la lista del [ADR-045](ADR-045-tres-patrones-de-la-ia.md). Trabaja con la bóveda ([ADR-044](ADR-044-la-boveda.md)): este ADR decide **qué** se marca, y la bóveda decide **cómo** viaja. Se construye en el bloque IA-7.

## Contexto

El escáner reconoce a una persona con listas:
- 245 nombres de pila y 154 apellidos;
- palabras que van antes de un nombre («señora», «doña»);
- santos («San José»);
- palabras de lugar («calle», «alcaldía»).

Cada falla pidió otra palabra, y esa lista no termina nunca: hay demasiadas calles, nombres y maneras de escribir.

**La medición (2026-10-19).** Un examen de 708 frases ficticias, hechas con plantillas:
- 504 personas;
- 507 lugares con nombre de persona: alcaldías, calles, colonias, el metro, escuelas;
- cada frase escrita tres veces: normal, toda en minúsculas (la mitad sin acentos, como se escribe en un chat) y toda en mayúsculas (como un acta).

Con ese examen, el escáner de hoy cubre completas:
- el **52 %** de las personas escritas con mayúscula inicial;
- el **0 %** de las escritas en minúsculas;
- el **0 %** de las escritas en mayúsculas.

En total cubre el **17 %**. Además, toma el 6 % de los lugares por personas.

## Decisión

### 1. Un modelo local de reconocimiento de nombres

Se usa **RoBERTa BNE capitel-plus**:
- es del Plan de Tecnologías del Lenguaje del Gobierno de España (Barcelona Supercomputing Center);
- su licencia es **Apache 2.0**;
- se entrenó a propósito también con textos en minúsculas y en mayúsculas;
- va en formato **ONNX, comprimido a int8: 125 MB**.

En la app corre en Rust, con `ort` (ONNX Runtime) y `tokenizers`:
- en el equipo y sin red;
- con la misma entrada da la misma salida.

No es IA generativa ni llama a ningún servicio: reconoce lo que ya está en el texto.

### 2. Un solo paso genérico para minúsculas y mayúsculas

Un texto escrito todo en minúsculas o todo en mayúsculas se lee dos veces: como viene y con mayúscula inicial en cada palabra, como lo escribiría una persona. De la segunda lectura cuenta solo lo que el modelo halla con una seguridad de **0.9 o más**.

El umbral es un número medido, no una lista. Se cambia solo con otra medición.

### 3. Lo que tiene formato se queda en el código

En el código se quedan los formatos que se pueden revisar sin interpretar:
- CURP, RFC, CLABE, tarjeta y número de seguro social, con su dígito verificador o su fecha;
- correo y teléfono, con su forma.

Esas reglas no crecen por caso.

La salud de una sola persona se sigue detectando como hoy (ADR-044 §3). La persona de la oración la da ahora el modelo.

### 4. Se quitan las listas

Salen del escáner:
- las listas de nombres y apellidos;
- las palabras que van antes de un nombre y los títulos;
- los santos;
- las palabras de lugar (`PLACE_WORDS`).

**Desde aquí no se agrega una palabra para un caso.** Una falla nueva se vuelve un caso del examen, y se mide otro umbral u otro modelo.

### 5. Marcar de más se acepta; marcar de menos es el riesgo

- **Marca de más** el 28 % de los lugares con nombre de persona, y algunas palabras de cargo como «directora» o «Presidente Municipal». Con la bóveda eso no destruye el dato:
  - la IA lee «calle [NOMBRE_1]» y entiende que es una calle;
  - el código pone el valor en el equipo.
- **Deja pasar** el 1.0 % de las personas del examen. Es el número que se vigila.

### 6. El examen entra al repositorio

El generador de frases y lo esperado de cada una van a `fixtures/`, solo con datos ficticios. Una prueba en seco lo corre con el modelo local, sin red ni costo, y falla si:
- las personas cubiertas completas bajan del valor medido;
- las personas no vistas suben del valor medido.

### 7. Textos largos

El modelo lee hasta 512 piezas de una vez, así que un documento se lee por tramos que se enciman un poco. Esa lectura va en segundo plano, como la de hoy (ADR-038).

## Medición

Mismo examen, ONNX en un núcleo, en Python. Los tiempos incluyen la segunda lectura. «Personas» cuenta las cubiertas completas; «no vistas» las que no tocó en ninguna letra.

| Modelo | Tamaño | Por frase | Personas | No vistas | Minúsculas | Mayúsculas | Normal | Lugar tomado por persona |
|---|---|---|---|---|---|---|---|---|
| Escáner de hoy (listas) | — | — | 17.3 % | 81.7 % | 0 % | 0 % | 51.8 % | 6.3 % |
| BETO, ajuste de mrm8488, int8 | 110 MB | 101 ms | 90.3 % | 7.7 % | 83.9 % | 87.5 % | 99.4 % | 21.1 % |
| mBERT multilingüe (Davlan), sin comprimir* | 709 MB | 106 ms | 88.9 % | 7.1 % | 84.5 % | 88.7 % | 93.5 % | 29.8 % |
| OpenAI privacy-filter, q4 | 917 MB | 325 ms | 85.9 % | 11.9 % | 83.9 % | 88.7 % | 85.1 % | 41.0 % |
| capitel-plus int8, sin el paso 2 | 125 MB | 52 ms | 96.0 % | 2.6 % | 97.0 % | 91.7 % | 99.4 % | 26.4 % |
| **capitel-plus int8, con el paso 2 (0.9)** | **125 MB** | **~88 ms** | **98.4 %** | **1.0 %** | **98.8 %** | **97.0 %** | **99.4 %** | **27.8 %** |

\* Comprimidos a int8, mBERT y wikineural (multilingüe) dejan de reconocer nombres (0 %).

Otras mediciones:
- **En Rust,** capitel-plus da el mismo 98.8 % que en Python (sin el umbral del paso 2), así que la app vería lo mismo que la medición.
- **Tiempos en Rust:** 90 ms por frase con un núcleo y 33 ms con cuatro. Un texto de 236 palabras tarda 1.5 s con un núcleo y 0.4 s con cuatro.
- **Umbral del paso 2:**
  - sin umbral marca algo de más en 73 de las 708 frases;
  - con 0.9, en 36;
  - con 0.98, en 15, y las personas cubiertas bajan a 97.8 %.

Lo que todavía se le escapa a capitel-plus con el paso 2:
- un nombre que también es palabra («la casa de Socorro Cruz»);
- tres frases en mayúsculas de una plantilla poco natural («MI HIJO DULCE TIENE EPILEPSIA»).

## Consecuencias

- **Instalador:** crece unos 125 MB del modelo, más el motor de ONNX Runtime, que en la prueba dejó un ejecutable de 25 MB.
- **Equipo:** sin tarjeta de video. La memoria y el tiempo se miden en un equipo modesto antes de cerrar el bloque.
- **Licencia:** el modelo se publica con Apache 2.0.
  - Su repositorio original pide hoy iniciar sesión. La medición usó una copia en ONNX de la comunidad (`SantiMB/roberta-base-bne-capitel-ner-plus-ONNX`).
  - Antes de distribuirlo, se obtienen los pesos oficiales, se convierten nosotros a ONNX, se guarda su huella y se confirma la licencia del corpus CAPITEL con el que se entrenó.
  - BETO se descartó también por su licencia: sus autores no garantizan el uso comercial de sus datos de entrenamiento.
- **El escáner se divide:**
  - los formatos siguen siendo código del ERP;
  - el reconocimiento de nombres es un modelo con su batería.

  Quién es dueño de cada parte lo decide el líder (`roles-y-tablero.md`).
- **La bóveda (IA-2) no depende de este ADR:** marca lo que el escáner encuentre, con listas o con el modelo. Los dos bloques pueden avanzar a la vez.
- **Los documentos del Expediente:** los nombres de la institución siguen sin taparse (ADR-040). Con el modelo, ese criterio se aplica a lo que el modelo reconoce.
