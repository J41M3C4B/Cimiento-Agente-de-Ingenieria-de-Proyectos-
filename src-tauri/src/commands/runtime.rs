//! The entry to the runtime of the AI platform (ADR-003, D6): the screens, and later the chat, send every request
//! here. A session is enough to come in; the permission is checked inside, per capability, and again before the
//! outcome is handed over.

use tauri::{AppHandle, Emitter, State};

use super::{guard, lock};
use crate::ai::runtime::{Authorize, Deliver, Outcome, Request, Runtime, SystemClock, Who};
use crate::capability::{Permission, Registry};
use crate::core::access::service::Session;
use crate::error::UiError;
use crate::Db;

/// One encrypted database per institution (ADR-002, D4): its id comes with the organization of the kernel (K1).
const ORGANIZATION: &str = "local";

/// The question of authorization, answered by the session until the kernel answers it (K2, ERP-10): the same person,
/// with the screen open, whose role has the permission.
struct SessionAuthority<'a>(&'a Session);

impl Authorize for SessionAuthority<'_> {
    fn may(&self, who: &Who, permission: Permission) -> bool {
        let Ok(state) = self.0.state.lock() else { return false };
        !state.locked
            && state.user.as_ref().is_some_and(|u| u.id == who.account && !u.must_change_password && u.role.can(permission))
    }
}

/// Every outcome is also heard by the window, so the person does not have to stay on the screen that asked.
struct Window(AppHandle);

impl Deliver for Window {
    fn deliver(&self, outcome: &Outcome) {
        let _ = self.0.emit("task-update", outcome);
    }
}

#[tauri::command]
pub fn runtime_submit(
    app: AppHandle,
    session: State<'_, Session>,
    db: State<'_, Db>,
    registry: State<'_, Registry>,
    request: Request,
) -> Result<Outcome, UiError> {
    let gate = guard(&session, "runtime_submit")?;
    let user = gate.user()?;
    let conn = lock(&db)?;
    let runtime = Runtime { registry: &registry, clock: &SystemClock, authorize: &SessionAuthority(&session), deliver: &Window(app) };
    runtime.submit(&conn, &Who { account: &user.id, organization: ORGANIZATION }, request).map_err(|_| UiError::internal())
}

/// The last outcome of a task of the person, to ask for it later.
#[tauri::command]
pub fn runtime_task(app: AppHandle, session: State<'_, Session>, db: State<'_, Db>, registry: State<'_, Registry>, task: String) -> Result<Option<Outcome>, UiError> {
    let gate = guard(&session, "runtime_task")?;
    let user = gate.user()?;
    let conn = lock(&db)?;
    let runtime = Runtime { registry: &registry, clock: &SystemClock, authorize: &SessionAuthority(&session), deliver: &Window(app) };
    runtime.task(&conn, &Who { account: &user.id, organization: ORGANIZATION }, &task).map_err(|_| UiError::internal())
}
