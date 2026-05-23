CREATE TABLE IF NOT EXISTS users (
    id            TEXT     NOT NULL PRIMARY KEY,
    username      TEXT     NOT NULL,
    password_hash TEXT     NOT NULL,
    group_id      TEXT     REFERENCES groups (id) ON DELETE SET NULL ON UPDATE RESTRICT,
    role          TEXT     NOT NULL DEFAULT 'regular' CHECK (role IN ('admin', 'regular')),
    status        TEXT     NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_group_id ON users (group_id);
