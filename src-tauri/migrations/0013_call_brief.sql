-- The «en pocas palabras» of a call (ADR-024): three or four plain sentences the AI wrote once from what the
-- reading understood, checked against it. It is for the person; what the AI reads stays in `canonical_json`.
-- It is cleared when the call is read again (the document it summarized is no longer the one in the reading).
ALTER TABLE call_reading ADD COLUMN brief_text TEXT;
