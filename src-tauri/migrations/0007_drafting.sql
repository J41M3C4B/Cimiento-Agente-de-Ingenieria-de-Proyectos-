-- Drafting and review (ADR-018). `project_section`, `budget_item` and `schedule_activity` come from migration 0001.

-- Whether the call asks for a project proposal as a document of its own: the code proposes it from the reading
-- and the person confirms it. NULL = not answered yet.
ALTER TABLE project ADD COLUMN asks_for_proposal INTEGER CHECK (asks_for_proposal IN (0,1));

-- An administrative or indirect expense: the call may cap them as a share of what is requested.
ALTER TABLE budget_item ADD COLUMN administrative INTEGER NOT NULL DEFAULT 0 CHECK (administrative IN (0,1));
