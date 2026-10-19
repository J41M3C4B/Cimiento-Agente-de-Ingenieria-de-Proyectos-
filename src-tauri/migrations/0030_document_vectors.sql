-- The vectors of the fragments of the documents of the institution (IA5, ADR-034 §3), to find by meaning what a
-- person asks with other words. One vector per fragment, by its `document_chunk.rowid`; only the documents of the
-- institution, never the files of a call. They are made from the clean text (already through the scanner) and only
-- when the person chose Gemini and saved its key. Owner: the core.
CREATE VIRTUAL TABLE document_chunk_vec USING vec0(embedding float[768] distance_metric=cosine);

-- With which model the vectors of each kind were made: if the model changes, they are made again.
CREATE TABLE vec_model (
  scope TEXT PRIMARY KEY,          -- 'records'
  model TEXT NOT NULL,             -- 'gemini:gemini-embedding-2'
  dims INTEGER NOT NULL,
  set_at TEXT NOT NULL
);
