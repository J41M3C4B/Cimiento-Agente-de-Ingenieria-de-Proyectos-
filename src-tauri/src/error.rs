//! Errors sent to the UI: a friendly Spanish message plus an internal code.
//! Nothing technical reaches the person (see `docs/08-estilo-redaccion.md`).

use crate::scanner::guard::GuardError;
use crate::core::error::ServiceError;
use crate::modules::projects::ProjectsError;
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
            ServiceError::InvalidPin => UiError::new("invalid_pin", "El PIN debe tener de 4 a 8 números, sin espacios ni letras."),
            ServiceError::WrongPin => UiError::new("wrong_pin", "Ese no es el PIN actual."),
            ServiceError::Storage(crate::storage::StorageError::WrongPassword) => {
                UiError::new("wrong_backup_password", "Esa contraseña no abre este respaldo, o el archivo no es un respaldo de Cimiento.")
            }
            ServiceError::Storage(crate::storage::StorageError::WeakPassword) => {
                UiError::new("weak_backup_password", "La contraseña debe tener al menos 8 caracteres.")
            }
            ServiceError::Storage(crate::storage::StorageError::BackupExists) => UiError::new("backup_exists", "Ya existe un archivo con ese nombre."),
            ServiceError::TextTooLarge => UiError::new(
                "text_too_large",
                "El texto es muy largo. Intente con una parte más corta.",
            ),
            ServiceError::UnknownKind => UiError::new("unknown_kind", "No reconocemos ese tipo de documento."),
            ServiceError::Storage(crate::storage::StorageError::NothingToConfirm) => {
                UiError::new("nothing_to_confirm", "No hay cambios pendientes por confirmar.")
            }
            ServiceError::NotFound => UiError::new("not_found", "No encontramos eso. Intente de nuevo."),
            ServiceError::Hr(crate::modules::hr::HrError::NotFound) => UiError::new("not_found", "No encontramos eso. Intente de nuevo."),
            ServiceError::Hr(crate::modules::hr::HrError::DuplicateTitle) => UiError::new("duplicate_position", "Ya existe un puesto con ese nombre."),
            ServiceError::Hr(crate::modules::hr::HrError::PositionInUse) => {
                UiError::new("position_in_use", "Hay personas en este puesto. Cámbielas de puesto antes de archivarlo.")
            }
            ServiceError::Hr(crate::modules::hr::HrError::EmptyTitle) => UiError::new("empty_text", "Este dato nos falta: escriba un nombre."),
            ServiceError::Hr(crate::modules::hr::HrError::UnknownModality) => UiError::new("unknown_modality", "Elija a cuál modalidad se parece."),
            ServiceError::Access(code) => UiError::new(code, access_message(code)),
            ServiceError::Facilities(crate::modules::facilities::FacilitiesError::NotFound) => UiError::new("not_found", "No encontramos eso. Intente de nuevo."),
            ServiceError::Care(crate::modules::care::CareError::NotFound) => UiError::new("not_found", "No encontramos eso. Intente de nuevo."),
            ServiceError::Care(crate::modules::care::CareError::DuplicateTitle) => UiError::new("duplicate_group", "Ya existe un grupo con ese nombre."),
            ServiceError::Care(crate::modules::care::CareError::EmptyTitle) => UiError::new("empty_text", "Este dato nos falta: escriba un nombre."),
            ServiceError::Care(crate::modules::care::CareError::AgeNeeded) => {
                UiError::new("age_needed", "Escriba la edad aproximada en la solicitud antes de darle ingreso.")
            }
            ServiceError::OnboardingIncomplete => UiError::new("onboarding_incomplete", "Todavía faltan datos de la institución. Revise los pasos marcados."),
            ServiceError::StaffMoved => UiError::new("staff_moved", "El personal ahora se lleva en su propia sección. Vuelva a abrir la pantalla."),
            other => {
                eprintln!("internal error: {other}");
                UiError::internal()
            }
        }
    }
}

impl From<ProjectsError> for UiError {
    fn from(e: ProjectsError) -> Self {
        use ProjectsError as P;
        match e {
            P::Core(e) => e.into(),
            P::NotFound => ServiceError::NotFound.into(),
            P::EmptyText => ServiceError::EmptyText.into(),
            P::TextTooLarge => ServiceError::TextTooLarge.into(),
            P::InvalidBudgetItem => UiError::new("invalid_budget_item", "Revise la cantidad (debe ser mayor que cero) y el precio (no puede ser negativo)."),
            P::BudgetIncomplete => UiError::new("budget_incomplete", "Faltan los costos de algunas partidas. Escríbalos y después confirme el presupuesto."),
            P::InvalidActivity => UiError::new("invalid_activity", "Revise los meses: empiezan en 1 y el final no puede ser antes del inicio."),
            P::GuideHasPersonalData => UiError::new("guide_has_personal_data", "La guía traería datos que parecen de una persona, así que no se generó. Revise los textos del proyecto."),
            P::InvalidYear => UiError::new("invalid_year", "El año no parece correcto. Escríbalo con cuatro cifras, por ejemplo 2026."),
            P::WrongStage => UiError::new("wrong_stage", "Esto todavía no se puede hacer en este paso."),
            P::AlreadyRunning => UiError::new("already_running", "Ya lo estamos haciendo. En cuanto termine, se muestra aquí."),
            P::Priority(_) => UiError::new("priority", "Cada calificación debe ir del 1 al 5."),
            P::Stage(e) => {
                let code = crate::modules::projects::diagnosis::stage_error_code(&e);
                UiError { code, message: stage_message(code).to_string() }
            }
            P::Internal(m) => {
                eprintln!("internal error: {m}");
                UiError::internal()
            }
        }
    }
}

/// The words for the rules of the accounts (ADR-028).
pub fn access_message(code: &str) -> &'static str {
    match code {
        "not_signed_in" => "Primero entre con su usuario y contraseña.",
        "session_locked" => "La pantalla está bloqueada. Escriba su contraseña para seguir.",
        "must_change_password" => "Antes de seguir, cambie la contraseña temporal por una suya.",
        "access_denied" => "Esto lo hace la persona administradora.",
        "already_set_up" => "La cuenta de administración ya existe.",
        "wrong_pin" => "Ese no es el PIN de esta computadora.",
        "pin_locked" => "Demasiados intentos con el PIN. Espere un momento.",
        "username_invalid" => "El usuario lleva de 3 a 32 letras o números, sin espacios ni acentos (puede usar punto o guion).",
        "username_taken" => "Ya hay una cuenta con ese usuario.",
        "password_short" => "La contraseña debe tener al menos 8 caracteres.",
        "password_long" => "La contraseña es demasiado larga.",
        "password_is_username" => "La contraseña no puede ser igual al usuario.",
        "password_same" => "La contraseña nueva debe ser distinta de la anterior.",
        "wrong_password" => "La contraseña actual no es correcta.",
        "recovery_wrong" => "El usuario o el código de recuperación no son correctos.",
        "recovery_wait" => "Demasiados intentos. Espere un momento antes de volver a intentar.",
        "role_not_allowed" => "Por ahora solo se dan cuentas de dirección o contaduría.",
        "last_admin" => "Debe quedar al menos una cuenta de administración activa.",
        "cannot_disable_self" => "No puede desactivar su propia cuenta.",
        "person_has_account" => "Esta persona ya tiene una cuenta.",
        "person_left" => "Esta persona ya no trabaja aquí: no se le puede dar acceso.",
        "already_requested" => "Eso ya se pidió borrar; la persona administradora lo revisará.",
        "field_builtin" => "Ese dato es de los que el programa necesita y no se puede quitar.",
        _ => "No se pudo hacer. Intente otra vez.",
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
