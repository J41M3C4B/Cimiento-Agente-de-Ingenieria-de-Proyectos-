mod access_service;
mod ai;
mod audit;
mod call_service;
mod care;
mod care_service;
mod commands;
mod common;
mod conversation_service;
mod diagnosis_service;
mod documents;
mod domain;
mod facilities;
mod facilities_service;
mod drafting_service;
mod error;
mod guide_service;
mod hr;
mod institution_context;
mod jobs;
mod review_service;
mod profile_sync;
mod scanner;
mod security_service;
mod service;
mod staff_service;
mod storage;
#[cfg(test)]
mod test_support;

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Shared database connection, opened at startup.
pub struct Db(pub Arc<Mutex<rusqlite::Connection>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let key = storage::get_or_create_db_key()?;
            let conn = storage::open_encrypted(&dir.join("cimiento.db"), &key)?;
            // a reading that was running when the program was closed waits to be resumed
            let _ = storage::calls::mark_interrupted(&conn);
            let db = Arc::new(Mutex::new(conn));
            // who is using the app (ADR-028): nobody until they enter
            app.manage(access_service::Session::new(db.clone()));
            app.manage(Db(db));
            app.manage(security_service::Attempts::default());
            app.manage(jobs::Jobs::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::access::access_status,
            commands::access::access_setup_admin,
            commands::access::access_login,
            commands::access::access_recover,
            commands::access::access_unlock,
            commands::access::access_lock,
            commands::access::access_logout,
            commands::access::access_change_password,
            commands::access::admin_overview,
            commands::access::admin_user_create,
            commands::access::admin_user_update,
            commands::access::admin_user_reset_password,
            commands::access::admin_request_resolve,
            commands::access::admin_audit,
            commands::access::admin_recovery_code_new,
            commands::profile_get,
            commands::profile_save,
            commands::profile_confirm,
            commands::document_add_text,
            commands::documents_list,
            commands::document_emergency_delete,
            commands::dev_load_fixture,
            commands::care::care_overview,
            commands::care::care_person_get,
            commands::care::care_person_save,
            commands::care::care_person_delete,
            commands::care::care_person_reveal,
            commands::care::care_group_save,
            commands::care::care_field_save,
            commands::care::care_field_delete,
            commands::care::care_waitlist_save,
            commands::care::care_waitlist_admit,
            commands::care::care_waitlist_delete,
            commands::facilities::facilities_overview,
            commands::facilities::facilities_site_save,
            commands::facilities::facilities_space_save,
            commands::facilities::facilities_space_delete,
            commands::facilities::facilities_equipment_save,
            commands::facilities::facilities_equipment_delete,
            commands::hr::hr_overview,
            commands::hr::hr_person_get,
            commands::hr::hr_person_save,
            commands::hr::hr_person_delete,
            commands::hr::hr_person_reveal,
            commands::hr::hr_position_save,
            commands::hr::hr_position_set_active,
            commands::hr::hr_modality_create,
            commands::hr::hr_field_save,
            commands::hr::hr_field_delete,
            commands::calls::project_create_from_call,
            commands::calls::call_reading_get,
            commands::calls::call_reading_confirm,
            commands::calls::call_reading_retry,
            commands::calls::call_brief_make,
            commands::diagnosis::ai_status,
            commands::diagnosis::ai_set_provider,
            commands::diagnosis::ai_models,
            commands::diagnosis::ai_set_models,
            commands::diagnosis::ai_set_key,
            commands::diagnosis::ai_clear_key,
            commands::diagnosis::ai_check,
            commands::diagnosis::ai_usage_report,
            commands::diagnosis::ai_set_cap,
            commands::diagnosis::project_job,
            commands::diagnosis::project_list,
            commands::diagnosis::project_create,
            commands::diagnosis::project_delete,
            commands::diagnosis::project_set_color,
            commands::diagnosis::project_set_donor_kind,
            commands::diagnosis::project_advance,
            commands::diagnosis::project_go_back,
            commands::diagnosis::conversation_get,
            commands::diagnosis::conversation_start,
            commands::diagnosis::conversation_send,
            commands::diagnosis::conversation_retry,
            commands::diagnosis::diagnosis_summary_generate,
            commands::diagnosis::diagnosis_summary_edit,
            commands::diagnosis::diagnosis_summary_confirm,
            commands::diagnosis::needs_get,
            commands::diagnosis::needs_propose,
            commands::diagnosis::need_add,
            commands::diagnosis::need_rate,
            commands::diagnosis::need_select,
            commands::drafting::drafting_get,
            commands::drafting::drafting_set_asks,
            commands::drafting::section_draft,
            commands::drafting::drafting_prepare,
            commands::drafting::sections_draft_all,
            commands::drafting::sections_confirm_all,
            commands::drafting::section_save,
            commands::drafting::section_confirm,
            commands::drafting::budget_save_item,
            commands::drafting::budget_delete_item,
            commands::drafting::budget_confirm,
            commands::drafting::schedule_save_activity,
            commands::drafting::schedule_delete_activity,
            commands::drafting::schedule_confirm,
            commands::drafting::review_get,
            commands::drafting::guide_export,
            commands::security::security_scan,
            commands::security::backup_create,
            commands::security::backup_restore,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
