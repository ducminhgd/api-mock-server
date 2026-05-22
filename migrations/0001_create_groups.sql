CREATE TABLE IF NOT EXISTS groups (
    id          TEXT     NOT NULL PRIMARY KEY,
    name        TEXT     NOT NULL,
    description TEXT,
    status      TEXT     NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_groups_name ON groups (name);
