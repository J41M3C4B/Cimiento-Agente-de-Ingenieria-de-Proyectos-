---
tipo: bloque
id: DIS-3
carril: diseno
estado: backlog
prioridad: alta
caja: Chat global; Interfaces de los módulos
pedido_por: IA-1
depende_de:
rama:
adr:
creado: 2026-10-20
cerrado:
---

## Qué y por qué

El IA-1 retira la IA vieja que vive dentro de Mi institución, porque ningún módulo puede llamar a la IA por su cuenta. Con ella desaparecen estos comandos:
- `assistant_ask`: el Asistente;
- `capture_ask`: el Capturista;
- `institution_fill_from_documents` e `institution_fill_all_from_documents`: llenar desde documentos;
- `proposals_reject`: las propuestas por campo;
- la lectura con IA que sigue a adjuntar un documento: el evento `record-suggested`;
- en la vista de un formulario, el dato `fill` (qué ventana llena el Capturista), que el ERP-5 quita.

Hoy los usan `src/core/assistant/Assistant.tsx`, `src/core/profile/FormWindow.tsx` y `src/core/profile/ProfilePage.tsx`, con sus pruebas. También desaparecen los comandos de IA de Proyectos que retire el ERP-4.

Se necesita que **las pantallas dejen de usarlos** antes de que se retiren. Lo que hacían lo sustituyen el chat global (DIS-1) y las capacidades nuevas; no se conserva.

## Terminado cuando

(Lo escribe el riel de diseño con el líder.)

- [ ] 

## Plan

## Cómo probarlo

**Cómo correrlo:**
- Carpeta y rama:
- Antes:
- Arrancar:
- Al terminar:

**Qué hacer y qué debe pasar:**

- [ ] 

## Hallazgos

## Resultado
