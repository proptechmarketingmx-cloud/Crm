WITH requested_client AS (
    SELECT clients.id, clients.latitude, clients.longitude, client_embeddings.embedding
    FROM clients
    JOIN client_embeddings ON client_embeddings.client_id = clients.id
    WHERE clients.id = ?1
),
scored_candidates AS (
    SELECT
        properties.id AS property_id,
        properties.zone_id,
        properties.latitude,
        properties.longitude,
        1.0 - vec_distance_cosine(
            requested_client.embedding,
            property_embeddings.embedding
        ) AS cosine_similarity,
        client_zone_cascade.score AS cascade_score,
        ega_zone_scores.score AS ega_score
    FROM requested_client
    JOIN properties ON properties.is_active = 1
    JOIN property_embeddings ON property_embeddings.property_id = properties.id
    JOIN client_zone_cascade
      ON client_zone_cascade.client_id = requested_client.id
     AND client_zone_cascade.zone_id = properties.zone_id
    JOIN ega_zone_scores ON ega_zone_scores.zone_id = properties.zone_id
    WHERE properties.latitude BETWEEN ?2 AND ?3
      AND (
          ?4 = 1
          OR (?5 = 0 AND properties.longitude BETWEEN ?6 AND ?7)
          OR (
              ?5 = 1
              AND (properties.longitude >= ?6 OR properties.longitude <= ?7)
          )
      )
)
SELECT
    property_id,
    zone_id,
    latitude,
    longitude,
    MAX(-1.0, MIN(1.0, cosine_similarity)) AS cosine_similarity,
    cascade_score,
    ega_score,
    0.4 * MAX(-1.0, MIN(1.0, cosine_similarity))
      + 0.3 * cascade_score
      + 0.3 * ega_score AS total_score
FROM scored_candidates
ORDER BY total_score DESC, property_id ASC
