pub const RECOMMENDATIONS_SQL: &str = include_str!("recommendations.sql");

pub const AUDIT_EVIDENCE_SQL: &str = "
    SELECT evidence_type, distance_km, source_hash, captured_at, consent_id
    FROM osint_evidence
    WHERE zone_id = ?1
      AND consent_status = 'granted'
      AND source_hash IS NOT NULL
    ORDER BY distance_km ASC, captured_at DESC
    LIMIT 1
";

pub const SQLITE_VEC_HEALTHCHECK_SQL: &str = "
    SELECT vec_distance_cosine(X'0000803F', X'0000803F')
";
