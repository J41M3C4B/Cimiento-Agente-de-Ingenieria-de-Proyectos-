mod ai;
mod audit;
mod commands;
mod common;
mod core;
mod documents;
mod error;
mod modules;
mod scanner;
mod storage;
#[cfg(test)]
mod architecture_tests;
#[cfg(test)]
mod test_support;

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Shared database connection, opened at startup.
pub struct Db(pub Arc<Mutex<rusqlite::Connection>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // the native window to choose a file: the record keeps where its original is (N4)
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // what reads scanned pages and photos (OCR, ADR-035) ships with the program; the screen hears how many
            // pages are read
            documents::ocr::set_home(app.path().resource_dir()?.join("ocr"));
            let handle = app.handle().clone();
            documents::reading::on_progress(move |key, done, total| {
                let _ = tauri::Emitter::emit(&handle, "reading-progress", serde_json::json!({ "key": key, "done": done, "total": total }));
            });
            #[allow(unused_mut)]
            let mut dir = app.path().app_data_dir()?;
            // development only: another folder, to try the first start from zero without touching the examples
            #[cfg(debug_assertions)]
            if let Some(other) = std::env::var_os("CIMIENTO_DATA_DIR").filter(|d| !d.is_empty()) {
                dir = other.into();
            }
            std::fs::create_dir_all(&dir)?;
            let key = storage::get_or_create_db_key()?;
            let conn = storage::open_encrypted(&dir.join("cimiento.db"), &key)?;
            // a reading that was running when the program was closed waits to be resumed
            let _ = modules::projects::storage::calls::mark_interrupted(&conn);
            // the photo of this month (ADR-033 §4): taken again at each opening until the month ends
            let _ = core::snapshot::take(&conn);
            let db = Arc::new(Mutex::new(conn));
            // who is using the app (ADR-028): nobody until they enter
            // the vectors of the documents that are missing (IA5), in the background, if the person chose Gemini
            tauri::async_runtime::spawn(core::archive::vectors::index_in_background(db.clone()));
            app.manage(core::access::service::Session::new(db.clone()));
            app.manage(Db(db));
            app.manage(core::security::Attempts::default());
            app.manage(modules::projects::jobs::Jobs::default());
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
            commands::access::access_team,
            commands::access::admin_overview,
            commands::access::admin_user_create,
            commands::access::admin_user_update,
            commands::access::admin_user_reset_password,
            commands::access::admin_request_resolve,
            commands::access::admin_audit,
            commands::access::admin_recovery_code_new,
            commands::profile_get,
            commands::profile_save,
            commands::form_get,
            commands::institution_overview,
            commands::form_save,
            commands::institution::institution_lists,
            commands::institution::institution_list_form,
            commands::institution::institution_list_save,
            commands::institution::institution_list_set_active,
            commands::institution::institution_list_delete,
            commands::records::institution_records,
            commands::records::institution_record_form,
            commands::records::institution_record_save,
            commands::records::institution_record_attach,
            commands::records::institution_record_detach,
            commands::records::institution_record_open,
            commands::institution::institution_clabe_reveal,
            commands::manual_entry,
            commands::assistant::assistant_ask,
            commands::assistant::capture_ask,
            commands::assistant::proposals_reject,
            commands::manual_search,
            commands::profile_confirm,
            commands::documents_list,
            commands::documents_search,
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
            commands::onboarding::onboarding_status,
            commands::onboarding::onboarding_save,
            commands::onboarding::onboarding_finish,
            commands::onboarding::onboarding_welcome_done,
            commands::facilities::facilities_overview,
            commands::facilities::facilities_site_save,
            commands::facilities::facilities_space_save,
            commands::facilities::facilities_space_delete,
            commands::facilities::facilities_equipment_save,
            commands::facilities::facilities_equipment_delete,
            commands::finance::finance_get,
            commands::finance::finance_save,
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
