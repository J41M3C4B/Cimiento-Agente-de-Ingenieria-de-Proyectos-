-- The note can change: its version (it rises with every change), a text and a phone, which is protected.
ALTER TABLE muestra_note ADD COLUMN body TEXT;
ALTER TABLE muestra_note ADD COLUMN phone TEXT;
ALTER TABLE muestra_note ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE muestra_note ADD COLUMN updated_at TEXT;
