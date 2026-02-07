-- Migration: Change image storage from base64 TEXT to URLs
-- This migration converts the photo storage from inline base64 to S3 URLs
--
-- WARNING: This will clear existing images. In production, you would:
-- 1. Upload existing base64 images to S3
-- 2. Update URLs
-- 3. Then change column types
--
-- For development, we're just resetting the data.

-- Drop NOT NULL constraint on photo_before (photo_after doesn't have it)
ALTER TABLE litter_reports 
    ALTER COLUMN photo_before DROP NOT NULL;

-- Clear existing photo data (development only!)
UPDATE litter_reports SET photo_before = NULL, photo_after = NULL;

-- Change columns from TEXT to VARCHAR for URLs
-- TEXT is fine for URLs, but VARCHAR(512) is more explicit about intent
ALTER TABLE litter_reports 
    ALTER COLUMN photo_before TYPE VARCHAR(512),
    ALTER COLUMN photo_after TYPE VARCHAR(512);

-- Add comments for clarity
COMMENT ON COLUMN litter_reports.photo_before IS 'S3 public URL to before cleanup image';
COMMENT ON COLUMN litter_reports.photo_after IS 'S3 public URL to after cleanup image';
