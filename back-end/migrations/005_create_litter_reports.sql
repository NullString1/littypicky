CREATE TYPE report_status AS ENUM ('pending', 'claimed', 'cleared', 'verified');

CREATE TABLE litter_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    location GEOMETRY(POINT, 4326) NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    description TEXT,
    photo_before TEXT NOT NULL,
    status report_status NOT NULL DEFAULT 'pending',
    claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    claimed_at TIMESTAMP,
    cleared_by UUID REFERENCES users(id) ON DELETE SET NULL,
    cleared_at TIMESTAMP,
    photo_after TEXT,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reports_location ON litter_reports USING GIST(location);
CREATE INDEX idx_reports_status ON litter_reports(status);
CREATE INDEX idx_reports_reporter ON litter_reports(reporter_id);
CREATE INDEX idx_reports_clearer ON litter_reports(cleared_by);
CREATE INDEX idx_reports_city ON litter_reports(city);
CREATE INDEX idx_reports_country ON litter_reports(country);
CREATE INDEX idx_reports_created_at ON litter_reports(created_at);
