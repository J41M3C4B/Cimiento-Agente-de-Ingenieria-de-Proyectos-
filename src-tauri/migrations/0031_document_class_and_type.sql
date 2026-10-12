-- The class the person chose when uploading a document (Legales, Financieros, Programas) and what the document is
-- within it, as a kind of the record (IA6, ADR-034; `core::reader::kinds`). Owner: the core.
ALTER TABLE document ADD COLUMN category TEXT CHECK (category IN ('legal', 'financial', 'programs'));
-- a code of `RECORD_TYPES` (`tax_status_certificate`…), or null when nobody knows yet
ALTER TABLE document ADD COLUMN doc_type TEXT;
-- who said it: `computed` (a rule recognized it by its words) or `user` (the person said it, or attached it to its
-- document of the record)
ALTER TABLE document ADD COLUMN doc_type_origin TEXT CHECK (doc_type_origin IN ('computed', 'user'));
