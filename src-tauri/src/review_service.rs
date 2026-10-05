//! The automatic review before a project is ready (docs/02-flujo-funcional.md). No AI: the code gathers the facts
//! of the project and `domain::checklist` decides. It is recomputed every time it is asked for.

use crate::domain::checklist::Report;
use crate::guide_service::{gather, review_report};
use crate::service::ServiceError;
use crate::storage::projects::ProjectRow;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ReviewView {
    pub project: ProjectRow,
    pub report: Report,
}

pub fn review(conn: &Connection, project_id: &str) -> Result<ReviewView, ServiceError> {
    let data = gather(conn, project_id)?;
    let report = review_report(conn, &data)?;
    Ok(ReviewView { project: data.project, report })
}
