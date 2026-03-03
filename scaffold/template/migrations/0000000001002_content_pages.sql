CREATE TABLE IF NOT EXISTS content_pages (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    tag TEXT NOT NULL,
    is_system SMALLINT NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (tag = lower(tag)),
    CHECK (tag ~ '^[a-z0-9]+(_[a-z0-9]+)*$')
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_content_pages_tag_live
    ON content_pages (tag)
    WHERE deleted_at IS NULL;
