mod ai;
mod audit;
mod call_service;
mod commands;
mod conversation_service;
mod diagnosis_service;
mod documents;
mod domain;
mod drafting_service;
mod error;
mod guide_service;
mod institution_context;
mod jobs;
mod review_service;
mod roster_service;
mod scanner;
mod security_service;
mod service;
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
            app.manage(Db(Arc::new(Mutex::new(conn))));
            app.manage(security_service::Attempts::default());
            app.manage(jobs::Jobs::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::profile_get,
            commands::profile_save,
            commands::profile_confirm,
            commands::document_add_text,
            commands::documents_list,
            commands::document_emergency_delete,
            commands::dev_load_fixture,
            commands::roster::roster_overview,
            commands::roster::roster_field_save,
            commands::roster::roster_field_delete,
            commands::roster::roster_entry_save,
            commands::roster::roster_entry_delete,
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
            commands::security::pin_status,
            commands::security::pin_set,
            commands::security::pin_clear,
            commands::security::pin_verify,
            commands::security::security_scan,
            commands::security::backup_create,
            commands::security::backup_restore,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
