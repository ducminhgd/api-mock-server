CREATE TABLE IF NOT EXISTS collections (
    id          TEXT     NOT NULL PRIMARY KEY,
    name        TEXT     NOT NULL,
    description TEXT,
    owner_id    TEXT     NOT NULL REFERENCES users (id) ON DELETE CASCADE ON UPDATE RESTRICT,
    status      TEXT     NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    visibility  TEXT     NOT NULL DEFAULT 'private' CHECK (visibility IN ('private', 'public')),
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_collections_owner_id ON collections (owner_id);
CREATE INDEX IF NOT EXISTS idx_collections_status    ON collections (status);
