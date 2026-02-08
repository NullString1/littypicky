CREATE TABLE score_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    points INTEGER NOT NULL,
    kind TEXT NOT NULL,
    report_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_score_events_user_id ON score_events(user_id);
CREATE INDEX idx_score_events_created_at ON score_events(created_at);
CREATE INDEX idx_score_events_user_time ON score_events(user_id, created_at);
