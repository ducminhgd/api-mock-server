CREATE TABLE IF NOT EXISTS endpoints (
    id                   TEXT    NOT NULL PRIMARY KEY,
    collection_id        TEXT    NOT NULL REFERENCES collections(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    name                 TEXT    NOT NULL,
    method               TEXT    NOT NULL CHECK(method IN ('GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS')),
    path                 TEXT    NOT NULL,
    status_code          INTEGER NOT NULL DEFAULT 200,
    response_headers     TEXT,
    response_body        TEXT,
    response_content_type TEXT,
    delay_ms             INTEGER NOT NULL DEFAULT 0,
    status               TEXT    NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive')),
    created_at           DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_endpoints_collection_id ON endpoints(collection_id);
CREATE INDEX IF NOT EXISTS idx_endpoints_status        ON endpoints(status);
CREATE INDEX IF NOT EXISTS idx_endpoints_method        ON endpoints(method);
