CREATE TABLE IF NOT EXISTS client_records (
    client_id TEXT PRIMARY KEY NOT NULL,
    data_json TEXT NOT NULL,
    curp_hmac BLOB,
    consent_json TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'saved')),
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS property_records (
    property_id TEXT PRIMARY KEY NOT NULL,
    client_id TEXT,
    data_json TEXT NOT NULL,
    document_paths_json TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'saved')),
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('client', 'property')),
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('created', 'updated', 'draft_saved')),
    occurred_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS postal_code_catalog (
    postal_code TEXT NOT NULL CHECK (length(postal_code) = 5),
    colonia TEXT NOT NULL,
    municipio TEXT NOT NULL,
    estado TEXT NOT NULL,
    PRIMARY KEY (postal_code, colonia)
);

CREATE INDEX IF NOT EXISTS idx_client_records_status_updated
    ON client_records (status, updated_at);
CREATE INDEX IF NOT EXISTS idx_property_records_status_updated
    ON property_records (status, updated_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity
    ON audit_log (entity_type, entity_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_postal_code_catalog_code
    ON postal_code_catalog (postal_code);
