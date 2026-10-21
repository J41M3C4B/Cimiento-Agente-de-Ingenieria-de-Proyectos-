//! Commands for the settings of the AI: provider, models, key, check, usage and cap. They are the platform's: the
//! API key never leaves Rust, and the screens only say what to change.

use super::guard;
use crate::ai::metrics::{self, UsageReport};
use crate::ai::settings::{self, AiSettings, ProviderKind};
use crate::ai::{self, AiError, AiProvider, ModelCheck, ModelTier};
use crate::core::access::service::Session;
use crate::core::error::ServiceError;
use crate::error::UiError;
use crate::storage::{self, StorageError};
use crate::Db;
use serde::Serialize;
use tauri::State;

/// Why the AI could not answer, as the screen says it («seguimos sin ayuda automática»).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Used,
    NotConfigured,
    Offline,
    Busy,
    BudgetExhausted,
    /// The provider's daily allowance is used up (a free plan has few calls per day).
    QuotaReached,
    KeyRejected,
    Unavailable,
}

impl From<&AiError> for AiStatus {
    fn from(e: &AiError) -> Self {
        match e {
            AiError::NoApiKey => AiStatus::NotConfigured,
            AiError::Offline => AiStatus::Offline,
            AiError::RateLimited | AiError::Timeout => AiStatus::Busy,
            AiError::BudgetExhausted => AiStatus::BudgetExhausted,
            AiError::QuotaReached => AiStatus::QuotaReached,
            AiError::Auth => AiStatus::KeyRejected,
            _ => AiStatus::Unavailable,
        }
    }
}

fn conn_err(e: impl Into<ServiceError>) -> UiError {
    UiError::from(e.into())
}

/// The provider chosen in the settings; it exists only if its key is saved in the keychain.
pub(crate) fn make_provider(db: &crate::storage::SharedDb) -> Option<Box<dyn AiProvider>> {
    let conn = db.lock().ok()?;
    let settings = settings::load(&conn).ok()?;
    drop(conn);
    let key = storage::get_api_key(settings.provider.as_str()).ok().flatten()?;
    Some(ai::build_provider(&settings, key))
}

#[derive(Serialize)]
pub struct KeysView {
    pub gemini: bool,
    pub anthropic: bool,
}

#[derive(Serialize)]
pub struct AiStatusView {
    pub provider: ProviderKind,
    /// A key is saved for the active provider.
    pub has_key: bool,
    pub keys: KeysView,
    pub model_light: String,
    pub model_strong: String,
    pub spent_mxn: f64,
    pub cap_mxn: f64,
    pub percent: f64,
    /// 80 % or more of the monthly cap.
    pub near_cap: bool,
    /// At the cap: AI is paused, the rest of the app keeps working.
    pub paused: bool,
}

fn has_key(provider: ProviderKind) -> bool {
    storage::get_api_key(provider.as_str()).ok().flatten().is_some()
}

#[tauri::command]
pub fn ai_status(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiStatusView, UiError> {
    guard(&session, "ai_status")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s: AiSettings = settings::load(&conn).map_err(conn_err)?;
    let spent = settings::month_spend_mxn(&conn).map_err(conn_err)?;
    let percent = if s.monthly_cap_mxn > 0.0 { spent * 100.0 / s.monthly_cap_mxn } else { 100.0 };
    Ok(AiStatusView {
        provider: s.provider,
        has_key: has_key(s.provider),
        keys: KeysView { gemini: has_key(ProviderKind::Gemini), anthropic: has_key(ProviderKind::Anthropic) },
        model_light: s.model_for(ModelTier::Light).to_string(),
        model_strong: s.model_for(ModelTier::Strong).to_string(),
        spent_mxn: spent,
        cap_mxn: s.monthly_cap_mxn,
        percent,
        near_cap: percent >= 80.0,
        paused: percent >= 100.0,
    })
}

#[tauri::command]
pub fn ai_set_provider(session: State<'_, Session>, db: State<'_, Db>, provider: ProviderKind) -> Result<(), UiError> {
    guard(&session, "ai_set_provider")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    s.provider = provider;
    settings::save(&conn, &s).map_err(conn_err)
}

#[derive(Serialize)]
pub struct AiModelsView {
    pub provider: ProviderKind,
    pub light: String,
    pub strong: String,
    /// Thinking depth of the strong tier; empty = provider default.
    pub effort: String,
    /// What the pickers offer for the active provider.
    pub known: Vec<String>,
    /// The strong tier's chain as it runs now (main model first, backups behind it).
    pub strong_chain: Vec<String>,
}

#[tauri::command]
pub fn ai_models(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiModelsView, UiError> {
    guard(&session, "ai_models")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s = settings::load(&conn).map_err(conn_err)?;
    Ok(AiModelsView {
        provider: s.provider,
        light: s.model_of(s.provider, ModelTier::Light).to_string(),
        strong: s.model_of(s.provider, ModelTier::Strong).to_string(),
        effort: s.effort_strong.clone(),
        known: s.known_models(s.provider),
        strong_chain: s.chain_of(s.provider, ModelTier::Strong),
    })
}

/// Swaps the models of the active provider (and the strong tier's thinking depth) without rebuilding the app.
#[tauri::command]
pub fn ai_set_models(session: State<'_, Session>, db: State<'_, Db>, light: String, strong: String, effort: String) -> Result<(), UiError> {
    guard(&session, "ai_set_models")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    let provider = s.provider;
    s.set_models(provider, &light, &strong, &effort);
    settings::save(&conn, &s).map_err(conn_err)
}

#[tauri::command]
pub fn ai_set_key(session: State<'_, Session>, db: State<'_, Db>, provider: ProviderKind, key: String) -> Result<(), UiError> {
    guard(&session, "ai_set_key")?;
    if key.trim().is_empty() {
        return Err(UiError::new("empty_text", "Este dato nos falta."));
    }
    storage::set_api_key(provider.as_str(), &key).map_err(|e: StorageError| conn_err(e))?;
    // with the key, the vectors of the documents already saved can be made (IA5)
    super::index_documents(&db.0);
    Ok(())
}

#[tauri::command]
pub fn ai_clear_key(session: State<'_, Session>, provider: ProviderKind) -> Result<(), UiError> {
    guard(&session, "ai_clear_key")?;
    storage::delete_api_key(provider.as_str()).map_err(|e: StorageError| conn_err(e))
}

#[derive(Serialize)]
pub struct AiCheckView {
    /// The key works and every configured model exists and can answer.
    pub ok: bool,
    pub models: Vec<ModelCheck>,
    /// Why it could not be checked (same plain-language keys as `AiStatus`).
    pub problem: Option<AiStatus>,
}

/// Checks the saved key and the configured models without spending a generation call.
#[tauri::command]
pub async fn ai_check(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiCheckView, UiError> {
    guard(&session, "ai_check")?;
    let db = db.0.clone();
    let provider = make_provider(&db);
    let Some(provider) = provider.as_deref() else {
        return Ok(AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::NotConfigured) });
    };
    Ok(match provider.check().await {
        Ok(models) => AiCheckView { ok: models.iter().all(|m| m.exists && m.can_generate), models, problem: None },
        Err(e @ (AiError::Auth | AiError::Offline | AiError::RateLimited)) => {
            AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::from(&e)) }
        }
        Err(_) => AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::Unavailable) },
    })
}

#[tauri::command]
pub fn ai_usage_report(session: State<'_, Session>, db: State<'_, Db>) -> Result<UsageReport, UiError> {
    guard(&session, "ai_usage_report")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s = settings::load(&conn).map_err(conn_err)?;
    metrics::usage_report(&conn, &s).map_err(conn_err)
}

#[tauri::command]
pub fn ai_set_cap(session: State<'_, Session>, db: State<'_, Db>, cap_mxn: f64) -> Result<(), UiError> {
    guard(&session, "ai_set_cap")?;
    if !cap_mxn.is_finite() || cap_mxn < 0.0 {
        return Err(UiError::new("negative_number", "Este nÃºmero no puede ser menor que cero."));
    }
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    s.monthly_cap_mxn = cap_mxn;
    settings::save(&conn, &s).map_err(conn_err)
}

