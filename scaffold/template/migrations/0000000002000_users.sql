CREATE TABLE users (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    uuid TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    name TEXT,
    email TEXT,
    locale TEXT,
    password TEXT NOT NULL,
    country_iso2 TEXT,
    contact_number TEXT,
    introducer_user_id BIGINT,
    ban SMALLINT NOT NULL DEFAULT 0,
    credit_1 NUMERIC(18,8) NOT NULL DEFAULT 0,
    credit_2 NUMERIC(18,8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CHECK (username = lower(username)),
    CONSTRAINT fk_users_country_iso2
        FOREIGN KEY (country_iso2) REFERENCES countries(iso2),
    CONSTRAINT fk_users_introducer
        FOREIGN KEY (introducer_user_id) REFERENCES users(id)
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_uuid ON users (uuid);
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_country_iso2 ON users (country_iso2);
CREATE INDEX idx_users_introducer_user_id ON users (introducer_user_id);
CREATE INDEX idx_users_ban ON users (ban);

CREATE TABLE introducer_changes (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    user_id BIGINT NOT NULL REFERENCES users(id),
    from_user_id BIGINT REFERENCES users(id),
    to_user_id BIGINT NOT NULL REFERENCES users(id),
    admin_id BIGINT NOT NULL REFERENCES admin(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_introducer_changes_user_id ON introducer_changes(user_id);
CREATE INDEX idx_introducer_changes_admin_id ON introducer_changes(admin_id);
