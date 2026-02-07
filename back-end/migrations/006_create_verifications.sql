CREATE TABLE report_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id UUID NOT NULL REFERENCES litter_reports(id) ON DELETE CASCADE,
    verifier_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_verified BOOLEAN NOT NULL,
    comment TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(report_id, verifier_id)
);

CREATE INDEX idx_verifications_report ON report_verifications(report_id);
CREATE INDEX idx_verifications_verifier ON report_verifications(verifier_id);
