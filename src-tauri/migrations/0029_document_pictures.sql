-- The pages of a file whose text was read from a picture (OCR, ADR-035), and of those the ones read with doubts,
-- as JSON arrays of page numbers (from 1). The text itself is in `document_chunk`, cleaned and through the scanner
-- like any text. Owner: the core.
ALTER TABLE document ADD COLUMN picture_pages TEXT NOT NULL DEFAULT '[]';
ALTER TABLE document ADD COLUMN doubtful_pages TEXT NOT NULL DEFAULT '[]';
