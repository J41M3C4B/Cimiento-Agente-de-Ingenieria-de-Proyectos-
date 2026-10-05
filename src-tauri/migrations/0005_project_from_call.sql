-- A project is born from its call (ADR-016). The call, its files and its reading belong to that one project;
-- the documents of the institution stay global and are never tied to a project.
--
-- `kind` says how the project began: 'call' (from a call; the only kind the screen offers today) or
-- 'internal' (an everyday need of the institution, without a call; the door is open but switched off).
-- Projects that already exist were all started from free text, so they are 'internal'.
ALTER TABLE project ADD COLUMN kind TEXT NOT NULL DEFAULT 'internal' CHECK (kind IN ('call','internal'));
ALTER TABLE project ADD COLUMN call_reading_id TEXT REFERENCES call_reading(id) ON DELETE SET NULL;

-- What the person said about the call (origin `user`): it wins over what the reading understood.
ALTER TABLE call_reading ADD COLUMN funder TEXT;
ALTER TABLE call_reading ADD COLUMN year INTEGER;

-- The character of each file inside the package, as the person marked it.
ALTER TABLE call_reading_file ADD COLUMN role TEXT NOT NULL DEFAULT 'main'
  CHECK (role IN ('main','annex','guide','form','notice','other'));
