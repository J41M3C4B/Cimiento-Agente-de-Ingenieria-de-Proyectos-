-- The data «Mi institución» was missing (ADR-033, audit P1), each one a field of the catalog (`core::institution::forms`).
-- Texts as written; codes of `core::institution::catalog`; dates `YYYY-MM-DD`. What never reaches the AI is said by its
-- field in the catalog, not here.

-- who it is, legally, and whom it takes in
ALTER TABLE institution ADD COLUMN legal_name TEXT;                 -- «Asilo X, I.A.P.»
ALTER TABLE institution ADD COLUMN purpose TEXT;                    -- objeto social, as the statutes say it
ALTER TABLE institution ADD COLUMN services TEXT;                   -- services or programs it offers
ALTER TABLE institution ADD COLUMN age_min INTEGER CHECK (age_min BETWEEN 0 AND 120);
ALTER TABLE institution ADD COLUMN age_max INTEGER CHECK (age_max BETWEEN 0 AND 120);
ALTER TABLE institution ADD COLUMN admission_criteria TEXT;

-- where it is (the AI gets only the municipality and the state)
ALTER TABLE institution ADD COLUMN street TEXT;
ALTER TABLE institution ADD COLUMN ext_number TEXT;
ALTER TABLE institution ADD COLUMN int_number TEXT;
ALTER TABLE institution ADD COLUMN neighborhood TEXT;               -- colonia
ALTER TABLE institution ADD COLUMN postal_code TEXT;

-- fiscal and registries
ALTER TABLE institution ADD COLUMN tax_regime TEXT;                 -- non_profit, general, other
ALTER TABLE institution ADD COLUMN fiscal_postal_code TEXT;
ALTER TABLE institution ADD COLUMN junta_folio TEXT;                -- registry before the Junta of its state (I.A.P.)
ALTER TABLE institution ADD COLUMN donee_category TEXT;             -- rubro autorizado
ALTER TABLE institution ADD COLUMN donee_letter_number TEXT;        -- número del oficio de autorización
ALTER TABLE institution ADD COLUMN donee_letter_date TEXT;
ALTER TABLE institution ADD COLUMN cluni_key TEXT;
ALTER TABLE institution ADD COLUMN legal_rep_valid_until TEXT;      -- vigencia del poder
