//! The executor (P7) and the path of a request through it: open or resume the task, run each step under its controls,
//! pause, close, and hand the outcome over. Before every invocation: the task may go on, the same step was not run
//! already, the permission, and the input fits its contract. After it: the output fits its contract, and the
//! permission once more before handing it over. Every pause and every close leaves its code in the trail.

use rusqlite::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::capability::{Approval, Call, Capability, CapabilityError, Run};

use super::outcome::{Again, Ask, Body, Next, Outcome};
use super::request::{Channel, Content, Request};
use super::store;
use super::task::{Pause, Phase, Reached, Task, TaskState, Waiting};
use super::{Runtime, RuntimeError, Who};

type Done = Result<Outcome, RuntimeError>;

/// The fingerprint of a capability and its input: what an approval is tied to, and how a repeated step is known.
/// The keys of the input are written in order, so the same input always gives the same fingerprint. With ERP-9 it
/// also takes the version of the datum.
pub fn fingerprint(capability: &str, input: &Value) -> String {
    let mut text = String::from(capability);
    text.push('\n');
    canonical(input, &mut text);
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(k.clone()).to_string());
                out.push(':');
                canonical(&map[k], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical(item, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// Whether a value fits a schema. If not, the field to correct: a name from the schema or the path, never a value.
fn fits(schema: &Value, value: &Value) -> Result<(), String> {
    let validator = jsonschema::validator_for(schema).map_err(|_| "input".to_string())?;
    let Some(error) = validator.iter_errors(value).next() else { return Ok(()) };
    if let jsonschema::error::ValidationErrorKind::Required { property } = error.kind() {
        if let Some(name) = property.as_str() {
            return Err(name.to_string());
        }
    }
    let path = error.instance_path().to_string();
    Err(match path.trim_start_matches('/') {
        "" => "input".to_string(),
        p => p.to_string(),
    })
}

fn error(code: &'static str, next: Next) -> Body {
    Body::Error { code: code.into(), next }
}

impl Runtime<'_> {
    /// Every request goes through here: a screen that names a capability, the chat, a fact of the calendar.
    pub fn submit(&self, conn: &Connection, who: &Who, request: Request) -> Done {
        let now = self.clock.now_ms();
        let Request { channel, task, content } = request;
        let mut task = match task {
            None => self.open(conn, who, channel, now)?,
            Some(id) => match store::load_task(conn, &id)? {
                Some(t) if t.account == who.account && t.organization == who.organization => t,
                // a task of another account is not touched: a new one says no
                _ => {
                    let mut t = self.open(conn, who, channel, now)?;
                    return self.close(conn, &mut t, TaskState::Denied, "not_your_task", 0, error("not_your_task", Next::StartAgain));
                }
            },
        };
        if task.state.is_closed() {
            // a closed task does not move: it says again how it ended
            return match store::last_outcome(conn, &task.id)? {
                Some(last) => Ok(last),
                None => self.respond(conn, &task, 0, Body::State { reason: task.reason.clone().unwrap_or_default() }),
            };
        }
        // an apply that started and never said how it ended is never run again (the AI blueprint §5)
        if let Some((n, capability)) = store::unfinished_apply(conn, &task.id)? {
            store::end_step(conn, &task.id, n, "uncertain", now)?;
            store::event(conn, &task.id, "step", Some(&capability), Some("uncertain"), now)?;
            return self.close(conn, &mut task, TaskState::Error, "uncertain_change", n, error("uncertain_change", Next::CheckHistory));
        }
        if task.pause_expired(now) {
            return self.limit(conn, &mut task, Reached::Pause);
        }
        task.turn_ms = now;
        match content {
            // the chat needs the router, which comes with IA-4
            Content::Text { .. } => self.close(conn, &mut task, TaskState::Error, "without_ai", 0, error("without_ai", Next::UseScreen)),
            Content::Invoke { capability, input } => self.invoke(conn, who, &mut task, &capability, input),
            Content::Answer { values } => self.answer(conn, who, &mut task, values),
            Content::Decision { fingerprint, approve } => self.decide(conn, who, &mut task, &fingerprint, approve),
        }
    }

    fn open(&self, conn: &Connection, who: &Who, channel: Channel, now: i64) -> Result<Task, RuntimeError> {
        let task = Task::open(format!("task_{}", ulid::Ulid::generate()), who.organization, who.account, channel, now);
        store::insert_task(conn, &task)?;
        store::event(conn, &task.id, "open", None, Some(task.channel.as_str()), now)?;
        Ok(task)
    }

    fn invoke(&self, conn: &Connection, who: &Who, task: &mut Task, capability: &str, input: Value) -> Done {
        let Some(cap) = self.registry.get(capability) else {
            return self.close(conn, task, TaskState::Error, "unknown_capability", 0, error("unknown_capability", Next::TryLater));
        };
        let mut again = None;
        if task.pause == Some(Pause::Approval) {
            if let Some(pending) = store::pending_approval(conn, &task.id)? {
                // the same proposal asked for again: it is shown again, nothing runs
                if pending.capability == cap.id && pending.fingerprint == fingerprint(cap.id, &input) {
                    let body = Body::Proposal { capability: pending.capability, proposal: pending.proposal, fingerprint: pending.fingerprint, again: None };
                    return self.respond(conn, task, 0, body);
                }
                // other parameters: the earlier approval no longer counts
                again = Some(Again::Changed);
            }
        }
        let phase = if cap.effect.changes() { Phase::Prepare } else { Phase::Read };
        self.step(conn, who, task, cap, phase, input, again)
    }

    fn answer(&self, conn: &Connection, who: &Who, task: &mut Task, values: Value) -> Done {
        let (Some(Pause::Person), Some(waiting)) = (task.pause, task.waiting.clone()) else {
            return Ok(self.notice(task, error("not_waiting", Next::StartAgain)));
        };
        let Some(cap) = self.registry.get(&waiting.capability) else {
            return self.close(conn, task, TaskState::Error, "unknown_capability", 0, error("unknown_capability", Next::TryLater));
        };
        let mut input = waiting.input;
        if let (Some(into), Value::Object(from)) = (input.as_object_mut(), values) {
            into.extend(from);
        }
        if waiting.correction {
            task.corrections += 1;
            if task.corrections > task.limits().corrections {
                return self.limit(conn, task, Reached::Corrections);
            }
        }
        self.step(conn, who, task, cap, waiting.phase, input, None)
    }

    fn decide(&self, conn: &Connection, who: &Who, task: &mut Task, fp: &str, approve: bool) -> Done {
        let pending = match (task.pause, store::pending_approval(conn, &task.id)?) {
            (Some(Pause::Approval), Some(p)) => p,
            _ => return Ok(self.notice(task, error("not_waiting", Next::StartAgain))),
        };
        // a decision for another proposal: the one waiting is shown again
        if pending.fingerprint != fp {
            let body = Body::Proposal { capability: pending.capability, proposal: pending.proposal, fingerprint: pending.fingerprint, again: Some(Again::Outdated) };
            return self.respond(conn, task, 0, body);
        }
        let Some(cap) = self.registry.get(&pending.capability) else {
            return self.close(conn, task, TaskState::Error, "unknown_capability", 0, error("unknown_capability", Next::TryLater));
        };
        let now = self.clock.now_ms();
        if !approve {
            store::decide_approval(conn, &task.id, fp, false, who.account, now)?;
            store::event(conn, &task.id, "approval", Some(cap.id), Some("rejected"), now)?;
            return self.close(conn, task, TaskState::Done, "rejected", 0, Body::State { reason: "rejected".into() });
        }
        // the approval is of a person with the permission, never of an agent
        if !self.authorize.may(who, cap.permission) {
            return self.deny(conn, task, cap.id);
        }
        // the input approved is the input applied
        if fingerprint(cap.id, &pending.input) != pending.fingerprint {
            return self.close(conn, task, TaskState::Error, "approval_mismatch", 0, error("approval_mismatch", Next::StartAgain));
        }
        store::decide_approval(conn, &task.id, fp, true, who.account, now)?;
        store::event(conn, &task.id, "approval", Some(cap.id), Some("approved"), now)?;
        self.step(conn, who, task, cap, Phase::Apply, pending.input, None)
    }

    /// One invocation of a capability, with its controls before and after.
    #[allow(clippy::too_many_arguments)]
    fn step(&self, conn: &Connection, who: &Who, task: &mut Task, cap: &'static Capability, phase: Phase, input: Value, again: Option<Again>) -> Done {
        let now = self.clock.now_ms();
        // the task may go on: its limits before every invocation (the blueprints §3.3)
        if let Err(reached) = task.may_step(now) {
            return self.limit(conn, task, reached);
        }
        let hash = fingerprint(cap.id, &input);
        if store::seen(conn, &task.id, cap.id, phase, &hash)? {
            return self.limit(conn, task, Reached::Repeated);
        }
        // the permission, right before the operation
        if !self.authorize.may(who, cap.permission) {
            return self.deny(conn, task, cap.id);
        }
        if let Err(field) = fits(&(cap.input)(), &input) {
            return self.pause_for_person(conn, task, cap, phase, input, 0, Ask::Correct { field, reason: "does_not_fit".into() }, true);
        }
        let run = match (cap.run, phase) {
            (Run::Read(s), Phase::Read) => s,
            (Run::Change { prepare, .. }, Phase::Prepare) => prepare,
            (Run::Change { apply, .. }, Phase::Apply) => apply,
            // the registry does not let a contract be like this
            _ => return self.close(conn, task, TaskState::Error, "run_does_not_fit", 0, error("run_does_not_fit", Next::TryLater)),
        };
        task.steps += 1;
        let n = task.steps;
        // written before it runs: if the app closes in the middle, the step stays «started» and is never repeated
        store::start_step(conn, &task.id, n, cap.id, phase, &hash, now)?;
        // the id of the task goes with everything the run leaves behind
        let result = (run.call)(&Call::new(conn, who.account, &task.id), input.clone());
        let value = match result {
            Ok(v) if fits(&(run.output)(), &v).is_ok() => v,
            Ok(_) => {
                self.ended(conn, task, n, cap.id, "failed")?;
                return self.close(conn, task, TaskState::Error, "bad_output", n, error("bad_output", Next::TryLater));
            }
            Err(CapabilityError::Invalid { field, reason }) => {
                self.ended(conn, task, n, cap.id, "invalid")?;
                return self.pause_for_person(conn, task, cap, phase, input, n, Ask::Correct { field: field.into(), reason: reason.into() }, true);
            }
            Err(CapabilityError::NeedsPerson { what }) => {
                self.ended(conn, task, n, cap.id, "needs_person")?;
                return self.pause_for_person(conn, task, cap, phase, input, n, Ask::Missing { what: what.into() }, false);
            }
            Err(CapabilityError::Failed { .. }) => {
                self.ended(conn, task, n, cap.id, "failed")?;
                return self.close(conn, task, TaskState::Error, "capability_failed", n, error("capability_failed", Next::TryLater));
            }
        };
        self.ended(conn, task, n, cap.id, "ok")?;
        match phase {
            Phase::Prepare if cap.approval == Approval::Required => {
                let now = self.clock.now_ms();
                store::ask_approval(conn, &task.id, &hash, cap.id, &input, &value, now)?;
                store::event(conn, &task.id, "approval", Some(cap.id), Some("asked"), now)?;
                task.waiting = Some(Waiting { capability: cap.id.into(), phase: Phase::Apply, input, correction: false });
                let body = Body::Proposal { capability: cap.id.into(), proposal: value, fingerprint: hash, again };
                self.pause(conn, task, Pause::Approval, "approval", n, body)
            }
            // a change that needs no approval is applied with the same input
            Phase::Prepare => self.step(conn, who, task, cap, Phase::Apply, input, None),
            Phase::Read | Phase::Apply => {
                // the permission once more, before handing it over
                if !self.authorize.may(who, cap.permission) {
                    return self.deny(conn, task, cap.id);
                }
                self.close(conn, task, TaskState::Done, "done", n, Body::Data { capability: cap.id.into(), value })
            }
        }
    }

    fn ended(&self, conn: &Connection, task: &Task, n: u32, capability: &str, result: &'static str) -> Result<(), RuntimeError> {
        let now = self.clock.now_ms();
        store::end_step(conn, &task.id, n, result, now)?;
        store::event(conn, &task.id, "step", Some(capability), Some(result), now)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn pause_for_person(&self, conn: &Connection, task: &mut Task, cap: &Capability, phase: Phase, input: Value, n: u32, ask: Ask, correction: bool) -> Done {
        // a change never resumes at its apply: what the person adds changes the input, so it is prepared and
        // approved again (the blueprints §3.5)
        let phase = if phase == Phase::Apply { Phase::Prepare } else { phase };
        task.waiting = Some(Waiting { capability: cap.id.into(), phase, input, correction });
        self.pause(conn, task, Pause::Person, "needs_person", n, Body::Clarification { capability: cap.id.into(), ask })
    }

    /// The turn ends and the task waits for the person, until its pause runs out.
    fn pause(&self, conn: &Connection, task: &mut Task, pause: Pause, reason: &'static str, n: u32, body: Body) -> Done {
        let now = self.clock.now_ms();
        task.state = TaskState::Paused;
        task.pause = Some(pause);
        task.reason = Some(reason.into());
        task.pause_until_ms = Some(now + task.limits().pause_ms);
        store::event(conn, &task.id, "pause", None, Some(reason), now)?;
        self.respond(conn, task, n, body)
    }

    fn deny(&self, conn: &Connection, task: &mut Task, capability: &str) -> Done {
        store::event(conn, &task.id, "denied", Some(capability), Some("no_permission"), self.clock.now_ms())?;
        self.close(conn, task, TaskState::Denied, "no_permission", 0, error("no_permission", Next::AskAdministrator))
    }

    fn limit(&self, conn: &Connection, task: &mut Task, reached: Reached) -> Done {
        self.close(conn, task, TaskState::Limit, reached.code(), 0, error(reached.code(), Next::StartAgain))
    }

    /// Every close is written with its reason, however the task ends (the blueprints §3.2).
    fn close(&self, conn: &Connection, task: &mut Task, state: TaskState, reason: &'static str, n: u32, body: Body) -> Done {
        let now = self.clock.now_ms();
        task.state = state;
        task.pause = None;
        task.reason = Some(reason.into());
        task.pause_until_ms = None;
        task.closed_ms = Some(now);
        task.waiting = None;
        store::event(conn, &task.id, "close", None, Some(reason), now)?;
        self.respond(conn, task, n, body)
    }

    /// The outcome, with the id of its task, kept as the last one of the task and handed over.
    fn respond(&self, conn: &Connection, task: &Task, step: u32, body: Body) -> Done {
        let outcome = self.outcome(task, step, body);
        store::save_task(conn, task, &outcome)?;
        self.deliver.deliver(&outcome);
        Ok(outcome)
    }

    /// An answer to a request that changed nothing: the task keeps what it said last.
    fn notice(&self, task: &Task, body: Body) -> Outcome {
        self.outcome(task, 0, body)
    }

    fn outcome(&self, task: &Task, step: u32, body: Body) -> Outcome {
        Outcome { task: task.id.clone(), step, channel: task.channel.clone(), state: task.state, at_ms: self.clock.now_ms(), body }
    }
}
