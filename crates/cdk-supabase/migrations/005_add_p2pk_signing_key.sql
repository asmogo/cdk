CREATE TABLE IF NOT EXISTS p2pk_signing_key (
    pubkey TEXT PRIMARY KEY,
    derivation_index INTEGER NOT NULL,
    derivation_path TEXT NOT NULL,
    created_time BIGINT NOT NULL
);

-- Bump schema version
INSERT INTO schema_info (key, value) VALUES ('schema_version', '5')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
