-- Real-time AI metrics (ADR-007). Migrations 0001 and 0002 are never edited.

-- How long the provider took to answer, in milliseconds (NULL for rows written before this).
ALTER TABLE ai_usage ADD COLUMN latency_ms INTEGER;
-- Tokens the model spent thinking. Already included in output_tokens (they are billed as output);
-- kept apart only so the metrics can show how much thinking costs.
ALTER TABLE ai_usage ADD COLUMN thought_tokens INTEGER NOT NULL DEFAULT 0;
-- Why an attempt failed (offline, rate_limited, quota_reached, auth, refused, truncated,
-- bad_output, http_<status>). NULL when the provider answered.
ALTER TABLE ai_usage ADD COLUMN error_kind TEXT;

-- The rate-limit guard and the metrics screen both look at "this model, recently".
CREATE INDEX idx_ai_usage_model_at ON ai_usage(model, at);
