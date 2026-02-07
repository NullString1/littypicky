-- Add performance optimization indexes for frequent queries
--
-- This migration adds indexes to improve query performance for:
-- 1. Report filtering by status and timestamps
-- 2. User score queries and leaderboards
-- 3. Verification counting
-- 4. "First in area" bonus calculations

-- Index for filtering cleared reports (used in scoring_service "first in area" check)
CREATE INDEX IF NOT EXISTS idx_reports_cleared_at 
    ON litter_reports(cleared_at DESC) 
    WHERE cleared_by IS NOT NULL;

-- Composite index for geospatial + time queries (scoring service)
CREATE INDEX IF NOT EXISTS idx_reports_cleared_location_time
    ON litter_reports(cleared_at, cleared_by) 
    WHERE cleared_by IS NOT NULL;

-- Index for report status filtering (feed queries)
CREATE INDEX IF NOT EXISTS idx_reports_status
    ON litter_reports(status);

-- Composite index for verification queries
CREATE INDEX IF NOT EXISTS idx_verifications_report_verified
    ON report_verifications(report_id, is_verified);

-- Index for user score ordering (leaderboards)
CREATE INDEX IF NOT EXISTS idx_user_scores_total_points
    ON user_scores(total_points DESC);

-- Index for cleared reports count (leaderboards)
CREATE INDEX IF NOT EXISTS idx_user_scores_reports_cleared
    ON user_scores(reports_cleared DESC);

-- Index for finding user's reports
CREATE INDEX IF NOT EXISTS idx_reports_reporter_created
    ON litter_reports(reporter_id, created_at DESC);

-- Index for finding user's claimed reports
CREATE INDEX IF NOT EXISTS idx_reports_claimed_by
    ON litter_reports(claimed_by, claimed_at DESC)
    WHERE claimed_by IS NOT NULL;

-- Index for finding user's cleared reports  
CREATE INDEX IF NOT EXISTS idx_reports_cleared_by
    ON litter_reports(cleared_by, cleared_at DESC)
    WHERE cleared_by IS NOT NULL;

-- Index for email token lookups (verification and password reset)
CREATE INDEX IF NOT EXISTS idx_email_tokens_token
    ON email_verification_tokens(token);

CREATE INDEX IF NOT EXISTS idx_email_tokens_user_expires
    ON email_verification_tokens(user_id, expires_at);

-- Index for refresh token lookups
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash
    ON refresh_tokens(token_hash);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_expires
    ON refresh_tokens(user_id, expires_at);

-- Analyze tables to update statistics
ANALYZE litter_reports;
ANALYZE report_verifications;
ANALYZE user_scores;
ANALYZE email_verification_tokens;
ANALYZE refresh_tokens;
