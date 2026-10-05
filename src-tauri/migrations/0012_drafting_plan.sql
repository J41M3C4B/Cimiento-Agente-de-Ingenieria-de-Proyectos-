-- What the assistant prepared when the drafting began (ADR-021): a clear title and a plain explanation for each
-- section of the proposal. The budget lines and the activities it proposed go to their own tables, marked as its own.
CREATE TABLE drafting_plan (
  project_id TEXT PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
  prompt_version TEXT NOT NULL,
  sections TEXT NOT NULL,            -- JSON: {"<section key>": {"title": "...", "plain": "..."}}
  created_at TEXT NOT NULL
);

-- Who made an activity: the person (`user`) or the assistant's proposal (`ai_assumption`).
ALTER TABLE schedule_activity ADD COLUMN origin TEXT NOT NULL DEFAULT 'user';
