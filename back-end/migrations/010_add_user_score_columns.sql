-- Add additional columns to user_scores table to track detailed statistics

-- Add columns for total reports created, cleared, and verifications
ALTER TABLE user_scores
ADD COLUMN total_reports INTEGER NOT NULL DEFAULT 0,
ADD COLUMN total_clears INTEGER NOT NULL DEFAULT 0,
ADD COLUMN total_verifications INTEGER NOT NULL DEFAULT 0;

-- Populate these columns with existing data
UPDATE user_scores us
SET total_reports = (
    SELECT COUNT(*)
    FROM litter_reports
    WHERE reporter_id = us.user_id
);

UPDATE user_scores us
SET total_clears = (
    SELECT COUNT(*)
    FROM litter_reports
    WHERE cleared_by = us.user_id AND status = 'cleared'
);

UPDATE user_scores us
SET total_verifications = (
    SELECT COUNT(*)
    FROM report_verifications
    WHERE verifier_id = us.user_id
);

-- Note: reports_cleared column is now redundant with total_clears, but we'll keep it for backward compatibility
-- We can remove it in a future migration if needed
