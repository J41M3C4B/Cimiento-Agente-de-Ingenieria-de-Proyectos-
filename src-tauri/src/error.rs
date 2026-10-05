//! Errors sent to the UI: a friendly Spanish message plus an internal code.
//! Nothing technical reaches the person (see `docs/08-estilo-redaccion.md`).

use crate::scanner::guard::GuardError;
use crate::service::ServiceError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UiError {
    pub code: &'static str,
    pub message: String,
}

impl UiError {
    pub fn new(code: &'static str, message: &str) -> Self {
        UiError { code, message: message.to_string() }
    }

    pub fn internal() -> Self {
        UiError::new(
            "internal",
            "Algo falló de nuestro lado. Su trabajo está guardado. Intente otra vez y, si sigue pasando, avísele a la persona que le instaló el programa.",
        )
    }
}

impl From<ServiceError> for UiError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::Guard(GuardError::BlockCannotBeIgnored) => UiError::new(
                "block_cannot_be_ignored",
                "Estos datos son de una persona y no podemos guardarlos tal cual. Podemos taparlos y seguir.",
            ),
            ServiceError::ProfileInvalid(_) => UiError::new(
                "profile_invalid",
                "Faltan algunos datos o algo no cuadra. Revise los avisos en pantalla.",
            ),
            ServiceError::EmptyText => UiError::new("empty_text", "Este dato nos falta."),
            ServiceError::InvalidRoster(code) => UiError::new(
                "invalid_roster",
                match code {
                    "required" => "Falta un dato que es necesario. Revise los campos marcados con asterisco.",
                    "not_a_number" => "Escriba solo números, sin letras ni signos.",
                    "year_invalid" => "El año no parece correcto. Escríbalo con cuatro cifras, por ejemplo 2019.",
                    "email_invalid" => "Ese correo no parece correcto. Revise que tenga el @.",
                    _ => "Revise los datos de la ficha.",
                },
            ),
            ServiceError::InvalidBudgetItem => UiError::new("invalid_budget_item", "Revise la cantidad (debe ser mayor que cero) y el precio (no puede ser negativo)."),
            ServiceError::BudgetIncomplete => UiError::new("budget_incomplete", "Faltan los costos de algunas partidas. Escríbalos y después confirme el presupuesto."),
            ServiceError::InvalidActivity => UiError::new("invalid_activity", "Revise los meses: empiezan en 1 y el final no puede ser antes del inicio."),
            ServiceError::GuideHasPersonalData => UiError::new("guide_has_personal_data", "La guía traería datos que parecen de una persona, así que no se generó. Revise los textos del proyecto."),
            ServiceError::InvalidPin => UiError::new("invalid_pin", "El PIN debe tener de 4 a 8 números, sin espacios ni letras."),
            ServiceError::WrongPin => UiError::new("wrong_pin", "Ese no es el PIN actual."),
            ServiceError::Storage(crate::storage::StorageError::WrongPassword) => {
                UiError::new("wrong_backup_password", "Esa contraseña no abre este respaldo, o el archivo no es un respaldo de Cimiento.")
            }
            ServiceError::Storage(crate::storage::StorageError::WeakPassword) => {
                UiError::new("weak_backup_password", "La contraseña debe tener al menos 8 caracteres.")
            }
            ServiceError::Storage(crate::storage::StorageError::BackupExists) => UiError::new("backup_exists", "Ya existe un archivo con ese nombre."),
            ServiceError::InvalidYear => UiError::new("invalid_year", "El año no parece correcto. Escríbalo con cuatro cifras, por ejemplo 2026."),
            ServiceError::TextTooLarge => UiError::new(
                "text_too_large",
                "El texto es muy largo. Intente con una parte más corta.",
            ),
            ServiceError::UnknownKind => UiError::new("unknown_kind", "No reconocemos ese tipo de documento."),
            ServiceError::Storage(crate::storage::StorageError::NothingToConfirm) => {
                UiError::new("nothing_to_confirm", "No hay cambios pendientes por confirmar.")
            }
            ServiceError::NotFound => UiError::new("not_found", "No encontramos eso. Intente de nuevo."),
            ServiceError::WrongStage => UiError::new("wrong_stage", "Esto todavía no se puede hacer en este paso."),
            ServiceError::AlreadyRunning => UiError::new("already_running", "Ya lo estamos haciendo. En cuanto termine, se muestra aquí."),
            ServiceError::Priority(_) => UiError::new("priority", "Cada calificación debe ir del 1 al 5."),
            ServiceError::Stage(e) => {
                let code = crate::diagnosis_service::stage_error_code(&e);
                UiError { code, message: stage_message(code).to_string() }
            }
            other => {
                eprintln!("internal error: {other}");
                UiError::internal()
            }
        }
    }
}

fn stage_message(code: &str) -> &'static str {
    match code {
        "profile_not_confirmed" => "Primero hay que revisar y confirmar los datos de su institución.",
        "profile_too_old" => "Los datos de su institución tienen más de un año. Revíselos y confírmelos otra vez.",
        "diagnosis_incomplete" => "Faltan preguntas del diagnóstico por responder.",
        "summary_not_confirmed" => "Falta que usted revise y confirme el resumen del diagnóstico.",
        "no_need_selected" => "Falta elegir cuál será el objetivo del proyecto.",
        "call_not_ready" => "Falta elegir una convocatoria que sí encaje con el proyecto.",
        "sections_not_confirmed" => "Faltan secciones del proyecto por revisar.",
        "checklist_has_errors" => "Hay detalles por corregir antes de terminar.",
        "already_final" => "Este proyecto ya está en el último paso.",
        _ => "Solo se puede regresar a un paso anterior.",
    }
}
