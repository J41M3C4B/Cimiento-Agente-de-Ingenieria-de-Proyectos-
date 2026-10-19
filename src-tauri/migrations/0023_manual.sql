-- The search index of the manual of the program (ADR-034 §3). The manual itself lives in the repository
-- (`docs/manual/`) and comes inside the program; this is only its index, rebuilt when the manual changes. Owner: the
-- core. Words are matched without accents («que» finds «qué»).

CREATE VIRTUAL TABLE manual_fts USING fts5(
  id UNINDEXED,
  title,
  body,
  tokenize = 'unicode61 remove_diacritics 2'
);

-- Which version of the manual is indexed (a fingerprint of its text).
CREATE TABLE manual_index (version TEXT NOT NULL);
