ALTER TABLE collections ADD COLUMN code TEXT;
UPDATE collections SET code = id WHERE code IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_collections_code ON collections (code);
