# 03 · Gobernanza de datos

## Principio central

**Cuántos, nunca quiénes, para la IA y los documentos.** El perfil de la institución no tiene ninguna tabla ni campo para registrar individuos. Desde el ADR-020 existe un **padrón** de beneficiarios, y desde el ADR-027 un **módulo de Personal** (tablas `hr_*`), ambos con una ficha por persona que solo vive en la base cifrada de este equipo: no pasan por la IA, no se escanean y no entran en los documentos; el perfil y la IA solo reciben lo que suman (cuántas personas por puesto o grupo, plazas sin cubrir, nómina, cuotas). Los atributos personales del personal (escolaridad, antigüedad) llegan a la IA solo como rangos de grupos de **3 o más personas**.

**Identificadores del personal (ADR-027):** CURP, RFC, NSS y CLABE se guardan en la base cifrada, se validan con su dígito verificador, nunca salen del servicio en claro y en pantalla se ven tapados. Mostrarlos queda en la bitácora (`hr.sensitive_viewed`, con el campo y nunca el valor). Borrar a una persona elimina todo lo suyo y queda registrado (`hr.person_deleted`). Los datos de salud del personal no se recogen.

## Niveles de datos

| Nivel | Qué incluye | Se guarda | Se envía a la IA |
|---|---|---|---|
| 🟢 Verde | Cifras agregadas (18 adultos mayores, 3 enfermeras), descripción de instalaciones, convocatorias públicas | Sí, cifrado | Sí |
| 🟡 Amarillo | Finanzas internas, presupuestos, cotizaciones, contratos, documentos internos | Sí, cifrado | Solo los fragmentos necesarios para la tarea |
| 🔴 Rojo | Cualquier dato que identifique a una persona: nombres de residentes/niñas/familiares/personal, CURP, RFC personal, INE, teléfonos y correos personales, datos de salud individuales, fotos de personas | **No** | **No** |

Excepción controlada: datos de contacto **institucionales** (teléfono y correo de la institución, nombre de la representante legal cuando la convocatoria lo exija) se capturan en campos específicos marcados `institutional_contact = true`, que el escáner ignora, y **nunca se envían a la IA**: se insertan por código en la exportación final.

## Marco legal (a verificar antes de usar datos reales)

En México, los datos de menores de edad y de salud son datos personales sensibles. La ley federal de protección de datos en posesión de particulares se actualizó en 2025. Antes de la Fase 7 (datos reales):

- [ ] Revisar con una persona con conocimiento legal el aviso de privacidad aplicable.
- [ ] Confirmar con la institución quién es responsable del tratamiento.
- [ ] Revisar los términos de uso de datos del proveedor de IA elegido (retención, uso para entrenamiento).

El diseño de "nivel rojo fuera" reduce mucho el riesgo, pero no sustituye esa revisión.

## Barreras (defensa en capas)

1. **Diseño de formularios:** solo se piden cifras y descripciones generales. Textos de ayuda recuerdan no escribir nombres.
2. **Escáner al capturar:** campos de texto libre y archivos pasan por el escáner antes de guardarse.
3. **Cuarentena:** si hay hallazgos, nada se guarda hasta que el usuario decide. Ver `04-escaner-datos-sensibles.md`.
4. **Solo versión limpia:** el original nunca se escribe en disco.
5. **Escáner antes de la IA:** segunda pasada sobre el prompt completo. Si encuentra algo, se tapa y se registra un evento `scanner.leak_prevented`.
6. **Escáner al exportar:** tercera pasada sobre el documento final.
7. **Escaneo periódico de la base:** comando de mantenimiento que recorre todos los textos guardados y reporta hallazgos.

## Cifrado y secretos

- Base de datos completa cifrada con SQLCipher.
- La llave de la base se genera aleatoriamente en la instalación y se guarda en el llavero del sistema operativo (Windows Credential Manager).
- La llave de API del proveedor de IA también va en el llavero; nunca en archivos ni en el frontend.
- Respaldo: exportación cifrada con contraseña que elige el usuario (Fase 6). Sin la contraseña, el respaldo no se puede abrir; la app lo explica con claridad.

## Retención y borrado

| Dato | Retención |
|---|---|
| Perfil de la institución | Mientras se use; versiones anteriores se conservan 3 años |
| Documentos de convocatorias | Mientras exista el `call_template` |
| Proyectos | Hasta que el usuario los archive o borre |
| Conversación del diagnóstico | Se resume al cerrar la etapa; el detalle se borra a los 90 días |
| Bitácora | 2 años |
| Registro de uso de IA | 2 años (solo métricas) |

**Botón de emergencia** ("Borrar esto y todo lo que salió de ahí"): borra el documento, sus fragmentos, entradas FTS, vectores y cualquier texto derivado que lo cite. Después ejecuta `VACUUM` para que no queden restos en el archivo. Registra en bitácora solo que hubo un borrado de emergencia.

## Bitácora

Registra **eventos**, nunca contenido:

```json
{ "at": "2026-10-01T10:00:00Z", "event": "scanner.quarantine", "entity": "document", "entity_id": "doc_123", "details": { "findings": { "curp": 3, "phone": 1 }, "decision": "redacted" } }
```

Eventos mínimos: `document.uploaded`, `scanner.quarantine`, `scanner.override`, `scanner.leak_prevented`, `emergency.delete`, `ai.call`, `export.created`, `profile.confirmed`, `stage.changed`, `backup.created`.

## Roles

En la versión local hay una sola cuenta por instalación (la computadora es el límite de acceso). Se prepara el modelo para roles futuros:

| Rol | Puede |
|---|---|
| `admin` | Configuración, llaves, borrados de emergencia, mantenimiento |
| `manager` | Todo lo del proyecto y confirmar datos |
| `staff` | Capturar y editar; no confirmar ni exportar |

Opcional en Fase 6: PIN para abrir la app.

## Metadatos de archivos

Al subir: se descartan metadatos (autor, empresa, rutas, historial). Al exportar: los documentos generados salen sin metadatos personales; autor = nombre de la institución. Fotos: se elimina EXIF (incluye ubicación GPS). En la versión 1 solo se aceptan fotos de instalaciones, con un aviso que lo recuerda.
