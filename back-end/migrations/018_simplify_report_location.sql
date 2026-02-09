-- Remove redundant location fields from litter_reports
-- Keep location geometry as the canonical source of truth

ALTER TABLE litter_reports
    DROP COLUMN IF EXISTS latitude,
    DROP COLUMN IF EXISTS longitude,
    DROP COLUMN IF EXISTS city,
    DROP COLUMN IF EXISTS country;

DROP INDEX IF EXISTS idx_reports_city;
DROP INDEX IF EXISTS idx_reports_country;
