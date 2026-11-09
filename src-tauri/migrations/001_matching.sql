PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY NOT NULL,
    latitude REAL NOT NULL CHECK (latitude BETWEEN -90.0 AND 90.0),
    longitude REAL NOT NULL CHECK (longitude BETWEEN -180.0 AND 180.0)
);

CREATE TABLE IF NOT EXISTS client_embeddings (
    client_id TEXT PRIMARY KEY NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    embedding BLOB NOT NULL CHECK (length(embedding) = 1536)
);

CREATE TABLE IF NOT EXISTS properties (
    id TEXT PRIMARY KEY NOT NULL,
    zone_id TEXT NOT NULL,
    latitude REAL NOT NULL CHECK (latitude BETWEEN -90.0 AND 90.0),
    longitude REAL NOT NULL CHECK (longitude BETWEEN -180.0 AND 180.0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE IF NOT EXISTS property_embeddings (
    property_id TEXT PRIMARY KEY NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    embedding BLOB NOT NULL CHECK (length(embedding) = 1536)
);

CREATE TABLE IF NOT EXISTS client_zone_cascade (
    client_id TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    zone_id TEXT NOT NULL,
    score REAL NOT NULL CHECK (score = score AND abs(score) < 1.0e308),
    PRIMARY KEY (client_id, zone_id)
);

CREATE TABLE IF NOT EXISTS graph_edges (
    source_id TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    weight REAL NOT NULL CHECK (weight BETWEEN 0.0 AND 1.0),
    PRIMARY KEY (source_id, target_id),
    CHECK (source_id <> target_id)
);

CREATE TABLE IF NOT EXISTS client_activities (
    client_id TEXT PRIMARY KEY NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    activity_type TEXT NOT NULL CHECK (
        activity_type IN ('purchase', 'visit', 'search', 'none')
    )
);

CREATE TABLE IF NOT EXISTS ega_zone_scores (
    zone_id TEXT PRIMARY KEY NOT NULL,
    score REAL NOT NULL CHECK (score BETWEEN 0.0 AND 1.0)
);

CREATE TABLE IF NOT EXISTS ega_heatmap_cells (
    zone_id TEXT PRIMARY KEY NOT NULL REFERENCES ega_zone_scores(zone_id) ON DELETE CASCADE,
    geometry_json TEXT NOT NULL,
    activity REAL NOT NULL CHECK (activity >= 0.0),
    density REAL NOT NULL CHECK (density >= 0.0)
);

CREATE TABLE IF NOT EXISTS osint_evidence (
    id INTEGER PRIMARY KEY,
    zone_id TEXT NOT NULL,
    evidence_type TEXT NOT NULL CHECK (
        evidence_type IN ('school', 'hospital', 'metro', 'shopping_center', 'industrial_park')
    ),
    distance_km REAL NOT NULL CHECK (distance_km >= 0.0),
    source_hash TEXT NOT NULL CHECK (length(source_hash) = 64),
    consent_id TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    consent_status TEXT NOT NULL CHECK (consent_status IN ('granted', 'revoked'))
);

CREATE INDEX IF NOT EXISTS idx_properties_active_location
    ON properties (is_active, latitude, longitude);
CREATE INDEX IF NOT EXISTS idx_properties_zone
    ON properties (zone_id);
CREATE INDEX IF NOT EXISTS idx_cascade_client_zone
    ON client_zone_cascade (client_id, zone_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_source
    ON graph_edges (source_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target
    ON graph_edges (target_id);
CREATE INDEX IF NOT EXISTS idx_osint_evidence_zone_consent_distance
    ON osint_evidence (zone_id, consent_status, distance_km);
