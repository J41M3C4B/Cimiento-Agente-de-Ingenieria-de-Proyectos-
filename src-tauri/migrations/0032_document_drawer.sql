-- The drawer each document belongs to (ADR-036): exactly one, with one owner. `record`: the Expediente of the
-- institution (the core); `projects`: the files of a call. The drawers of the modules come with them. A document of
-- before ADR-036 that is in no drawer stays without one: no Reader reads it and no agent searches it (the institution
-- uploads them again, decision of 2026-10-12). Owner: the core.
ALTER TABLE document ADD COLUMN drawer TEXT;
UPDATE document SET drawer = 'record' WHERE id IN (SELECT document_id FROM core_record WHERE document_id IS NOT NULL);
UPDATE document SET drawer = 'projects' WHERE kind = 'call';
CREATE INDEX document_by_drawer ON document (drawer);
