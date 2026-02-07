CREATE TABLE feed_post_images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id UUID NOT NULL REFERENCES feed_posts(id) ON DELETE CASCADE,
    image_url VARCHAR NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_feed_post_images_post_id ON feed_post_images(post_id);
CREATE INDEX idx_feed_post_images_position ON feed_post_images(post_id, position);
