CREATE TABLE IF NOT EXISTS collection_shares (
    id            TEXT     NOT NULL PRIMARY KEY,
    collection_id TEXT     NOT NULL REFERENCES collections (id) ON DELETE CASCADE ON UPDATE RESTRICT,
    user_id       TEXT     REFERENCES users  (id) ON DELETE CASCADE ON UPDATE RESTRICT,
    group_id      TEXT     REFERENCES groups (id) ON DELETE CASCADE ON UPDATE RESTRICT,
    role          TEXT     NOT NULL DEFAULT 'viewer' CHECK (role IN ('viewer', 'editor')),
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (user_id IS NOT NULL AND group_id IS NULL) OR
        (user_id IS NULL     AND group_id IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_collection_shares_collection_id ON collection_shares (collection_id);
CREATE INDEX IF NOT EXISTS idx_collection_shares_user_id       ON collection_shares (user_id);
CREATE INDEX IF NOT EXISTS idx_collection_shares_group_id      ON collection_shares (group_id);

-- Each user/group may only appear once per collection.
CREATE UNIQUE INDEX IF NOT EXISTS uq_collection_shares_collection_user
    ON collection_shares (collection_id, user_id)  WHERE user_id  IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_collection_shares_collection_group
    ON collection_shares (collection_id, group_id) WHERE group_id IS NOT NULL;
