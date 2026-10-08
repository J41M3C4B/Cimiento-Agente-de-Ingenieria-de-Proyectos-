//! What the AI is doing right now, per project. The calls themselves already run to the end and save their
//! result whatever the screen does; what was missing was knowing, from the screen, that one is in flight. With
//! this the person can change section and come back (or reopen the project) and see the work still going, and a
//! second click can never start a second call that spends the allowance twice or overwrites the first.
//!
//! It lives in memory on purpose: a call dies with the program, so after a restart there is nothing to recover.

use crate::diagnosis_service::AiStatus;
use crate::core::error::ServiceError;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// What kind of work the AI is doing for a project (the screen picks its words from this).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    /// A message of the diagnosis conversation.
    Turn,
    /// The summary of the diagnosis.
    Summary,
    /// The objectives that the AI suggests.
    Needs,
    /// The «en pocas palabras» of a call (ADR-024). Its slot is `call:<reading id>`, not a project's.
    Brief,
}

/// How the last job of a project ended. It is handed over once, to whoever asks first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Finished {
    pub kind: JobKind,
    pub ai: AiStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct JobStatus {
    pub running: Option<JobKind>,
    pub finished: Option<Finished>,
}

#[derive(Default)]
struct Inner {
    running: HashMap<String, JobKind>,
    finished: HashMap<String, Finished>,
}

/// Tauri state: one AI job per project at a time.
#[derive(Clone, Default)]
pub struct Jobs(Arc<Mutex<Inner>>);

fn lock(m: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    // a panic elsewhere must not leave the project blocked for good
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Jobs {
    /// Takes the project's slot, or says another job is already running on it.
    pub fn claim(&self, project_id: &str, kind: JobKind) -> Result<JobGuard, ServiceError> {
        let mut inner = lock(&self.0);
        if inner.running.contains_key(project_id) {
            return Err(ServiceError::AlreadyRunning);
        }
        inner.running.insert(project_id.to_string(), kind);
        inner.finished.remove(project_id);
        Ok(JobGuard { jobs: self.clone(), project_id: project_id.to_string(), kind, ai: None })
    }

    /// What is running for the project, or, if nothing is, how the last job ended (once).
    pub fn status(&self, project_id: &str) -> JobStatus {
        let mut inner = lock(&self.0);
        let running = inner.running.get(project_id).copied();
        let finished = if running.is_none() { inner.finished.remove(project_id) } else { None };
        JobStatus { running, finished }
    }
}

/// Holds a project's slot while its job runs. The slot is freed when this is dropped, also on an error or a panic.
pub struct JobGuard {
    jobs: Jobs,
    project_id: String,
    kind: JobKind,
    ai: Option<AiStatus>,
}

impl JobGuard {
    /// Records how the AI part went; the slot is freed right after, when the guard goes out of scope.
    pub fn finish(mut self, ai: AiStatus) {
        self.ai = Some(ai);
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        let mut inner = lock(&self.jobs.0);
        inner.running.remove(&self.project_id);
        if let Some(ai) = self.ai {
            inner.finished.insert(self.project_id.clone(), Finished { kind: self.kind, ai });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_job_on_the_same_project_is_refused_until_the_first_ends() {
        let jobs = Jobs::default();
        let first = jobs.claim("p1", JobKind::Summary).unwrap();
        assert!(matches!(jobs.claim("p1", JobKind::Turn), Err(ServiceError::AlreadyRunning)));
        assert_eq!(jobs.status("p1").running, Some(JobKind::Summary));
        // another project is not affected
        let other = jobs.claim("p2", JobKind::Needs).unwrap();
        drop(first);
        assert_eq!(jobs.status("p1").running, None);
        assert!(jobs.claim("p1", JobKind::Turn).is_ok());
        drop(other);
    }

    #[test]
    fn the_slot_is_freed_when_the_job_fails_or_panics() {
        let jobs = Jobs::default();
        let failed = |jobs: &Jobs| -> Result<(), ServiceError> {
            let _job = jobs.claim("p1", JobKind::Summary)?;
            Err(ServiceError::NotFound) // leaves without calling finish
        };
        assert!(failed(&jobs).is_err());
        assert_eq!(jobs.status("p1"), JobStatus { running: None, finished: None }, "a failure with no AI outcome leaves nothing to report");

        let again = jobs.clone();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _job = again.claim("p1", JobKind::Turn).unwrap();
            panic!("boom");
        }));
        assert!(jobs.claim("p1", JobKind::Turn).is_ok(), "the slot is freed when the job unwinds");
    }

    #[test]
    fn how_a_job_ended_is_handed_over_once_and_cleared_by_the_next_job() {
        let jobs = Jobs::default();
        jobs.claim("p1", JobKind::Summary).unwrap().finish(AiStatus::QuotaReached);
        let first = jobs.status("p1");
        assert_eq!(first.running, None);
        assert_eq!(first.finished, Some(Finished { kind: JobKind::Summary, ai: AiStatus::QuotaReached }));
        assert_eq!(jobs.status("p1").finished, None, "only the first to ask gets it");

        jobs.claim("p1", JobKind::Needs).unwrap().finish(AiStatus::Used);
        let guard = jobs.claim("p1", JobKind::Turn).unwrap();
        assert_eq!(jobs.status("p1").finished, None, "while a job runs nothing is handed over, and the earlier outcome is gone");
        drop(guard);
    }
}
