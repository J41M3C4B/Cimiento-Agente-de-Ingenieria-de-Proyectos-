-- The audit store of the kernel (K9): what happened, with ids and codes, never data of a person.
CREATE TABLE kernel_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL,
    event TEXT NOT NULL,             -- task.opened, task.step, task.denied, task.approved, task.closed
    correlation_id TEXT NOT NULL,    -- the task
    account_id TEXT,                 -- who the task runs for
    subject TEXT,                    -- the capability
    outcome TEXT,                    -- a code
    reason TEXT                      -- a code
);
CREATE INDEX kernel_audit_by_correlation ON kernel_audit(correlation_id);
