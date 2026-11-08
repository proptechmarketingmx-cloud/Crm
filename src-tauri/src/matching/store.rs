use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::error::AppError;

use super::queries::{AUDIT_EVIDENCE_SQL, RECOMMENDATIONS_SQL, SQLITE_VEC_HEALTHCHECK_SQL};

const MATCHING_SCHEMA: &str = include_str!("../../migrations/001_matching.sql");
const MAX_TOP_K: u8 = 50;
const MAX_DISTANCE_KM: f64 = 100.0;
const EARTH_RADIUS_KM: f64 = 6_371.0088;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchRequest {
    pub client_id: Uuid,
    pub top_k: u8,
    pub max_distance_km: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchResponse {
    pub matches: Vec<PropertyMatch>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyMatch {
    pub property_id: Uuid,
    pub total_score: f64,
    pub distance_km: f64,
    pub breakdown: MatchScoreBreakdown,
    pub explanation: String,
    pub evidence: Option<AuditedEvidence>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchScoreBreakdown {
    pub content_similarity: f64,
    pub cascade_score: f64,
    pub ega_score: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditedEvidence {
    pub evidence_type: String,
    pub distance_km: f64,
    pub source_hash: String,
    pub captured_at: String,
    pub consent_id: String,
}

#[derive(Clone, Debug)]
struct Candidate {
    property_id: Uuid,
    zone_id: String,
    latitude: f64,
    longitude: f64,
    cosine_similarity: f64,
    cascade_score: f64,
    ega_score: f64,
    total_score: f64,
    distance_km: f64,
}

pub struct MatchingStore {
    connection: Mutex<Connection>,
}

impl MatchingStore {
    /// Inicializa el esquema de matching y comprueba la extensión sqlite-vec.
    ///
    /// La conexión debe ser local y tener cargada una copia confiable de
    /// sqlite-vec antes de construir el store.
    pub fn initialize(connection: Connection) -> Result<Self, AppError> {
        connection
            .execute_batch(MATCHING_SCHEMA)
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let vec_distance = connection
            .query_row(SQLITE_VEC_HEALTHCHECK_SQL, [], |row| row.get::<_, f64>(0))
            .map_err(|_| AppError::DatabaseUnavailable)?;
        if !vec_distance.is_finite() || vec_distance.abs() >= f64::EPSILON {
            return Err(AppError::DatabaseUnavailable);
        }

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn recommend(&self, request: &MatchRequest) -> Result<MatchResponse, AppError> {
        validate_request(request)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;

        let (latitude, longitude) = connection
            .query_row(
                "SELECT clients.latitude, clients.longitude
                 FROM clients
                 JOIN client_embeddings ON client_embeddings.client_id = clients.id
                 WHERE clients.id = ?1",
                [request.client_id.to_string()],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?)),
            )
            .optional()
            .map_err(|_| AppError::DatabaseUnavailable)?
            .ok_or(AppError::InvalidInput)?;
        validate_coordinates(latitude, longitude)?;

        let bounds = BoundingBox::around(latitude, longitude, request.max_distance_km);
        let mut statement = connection
            .prepare(RECOMMENDATIONS_SQL)
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let rows = statement
            .query_map(
                params![
                    request.client_id.to_string(),
                    bounds.min_latitude,
                    bounds.max_latitude,
                    if bounds.all_longitudes { 1_i64 } else { 0_i64 },
                    if bounds.crosses_antimeridian {
                        1_i64
                    } else {
                        0_i64
                    },
                    bounds.min_longitude,
                    bounds.max_longitude,
                ],
                |row| {
                    let property_id: String = row.get(0)?;
                    Ok(Candidate {
                        property_id: Uuid::parse_str(&property_id)
                            .map_err(|error| rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            ))?,
                        zone_id: row.get(1)?,
                        latitude: row.get(2)?,
                        longitude: row.get(3)?,
                        cosine_similarity: row.get(4)?,
                        cascade_score: row.get(5)?,
                        ega_score: row.get(6)?,
                        total_score: row.get(7)?,
                        distance_km: 0.0,
                    })
                },
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let mut candidates = rows
            .filter_map(|row| match row {
                Ok(mut candidate) => {
                    if !candidate.cosine_similarity.is_finite()
                        || !(-1.0..=1.0).contains(&candidate.cosine_similarity)
                        || !candidate.cascade_score.is_finite()
                        || !candidate.ega_score.is_finite()
                        || !(0.0..=1.0).contains(&candidate.ega_score)
                        || !candidate.total_score.is_finite()
                    {
                        return Some(Err(AppError::CalculationFailed));
                    }
                    match haversine_distance_km(
                        latitude,
                        longitude,
                        candidate.latitude,
                        candidate.longitude,
                    ) {
                        Ok(distance) if distance <= request.max_distance_km => {
                            candidate.distance_km = distance;
                            Some(Ok(candidate))
                        }
                        Ok(_) => None,
                        Err(error) => Some(Err(error)),
                    }
                }
                Err(_) => Some(Err(AppError::DatabaseUnavailable)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        candidates.sort_by(|left, right| {
            right
                .total_score
                .total_cmp(&left.total_score)
                .then_with(|| left.property_id.cmp(&right.property_id))
        });
        candidates.truncate(usize::from(request.top_k));

        let matches = candidates
            .into_iter()
            .map(|candidate| {
                let evidence = load_audited_evidence(&connection, &candidate.zone_id)?;
                let breakdown = MatchScoreBreakdown {
                    content_similarity: 0.4 * candidate.cosine_similarity,
                    cascade_score: 0.3 * candidate.cascade_score,
                    ega_score: 0.3 * candidate.ega_score,
                };
                let explanation = build_explanation(breakdown, evidence.as_ref());

                Ok(PropertyMatch {
                    property_id: candidate.property_id,
                    total_score: candidate.total_score,
                    distance_km: candidate.distance_km,
                    breakdown,
                    explanation,
                    evidence,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(MatchResponse { matches })
    }
}

/// Recomienda propiedades próximas usando señales calculadas dentro de SQLite.
///
/// sqlite-vec calcula coseno directamente en la base local; Rust recibe solo
/// puntuaciones y metadatos de los candidatos, no los vectores completos.
#[tauri::command]
pub fn recommend_properties(
    request: MatchRequest,
    store: State<'_, MatchingStore>,
) -> Result<MatchResponse, AppError> {
    store.recommend(&request)
}

fn validate_request(request: &MatchRequest) -> Result<(), AppError> {
    if request.top_k == 0
        || request.top_k > MAX_TOP_K
        || !request.max_distance_km.is_finite()
        || !(0.0..=MAX_DISTANCE_KM).contains(&request.max_distance_km)
    {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}

fn validate_coordinates(latitude: f64, longitude: f64) -> Result<(), AppError> {
    if !latitude.is_finite()
        || !longitude.is_finite()
        || !(-90.0..=90.0).contains(&latitude)
        || !(-180.0..=180.0).contains(&longitude)
    {
        return Err(AppError::DatabaseUnavailable);
    }
    Ok(())
}

fn haversine_distance_km(
    latitude_a: f64,
    longitude_a: f64,
    latitude_b: f64,
    longitude_b: f64,
) -> Result<f64, AppError> {
    validate_coordinates(latitude_a, longitude_a)?;
    validate_coordinates(latitude_b, longitude_b)?;

    let latitude_delta = (latitude_b - latitude_a).to_radians();
    let longitude_delta = (longitude_b - longitude_a).to_radians();
    let a = (latitude_delta / 2.0).sin().powi(2)
        + latitude_a.to_radians().cos()
            * latitude_b.to_radians().cos()
            * (longitude_delta / 2.0).sin().powi(2);
    let a = a.clamp(0.0, 1.0);
    Ok(2.0 * EARTH_RADIUS_KM * a.sqrt().atan2((1.0 - a).sqrt()))
}

fn load_audited_evidence(
    connection: &Connection,
    zone_id: &str,
) -> Result<Option<AuditedEvidence>, AppError> {
    connection
        .query_row(AUDIT_EVIDENCE_SQL, [zone_id], |row| {
            Ok(AuditedEvidence {
                evidence_type: row.get(0)?,
                distance_km: row.get(1)?,
                source_hash: row.get(2)?,
                captured_at: row.get(3)?,
                consent_id: row.get(4)?,
            })
        })
        .optional()
        .map_err(|_| AppError::DatabaseUnavailable)
}

fn build_explanation(
    breakdown: MatchScoreBreakdown,
    evidence: Option<&AuditedEvidence>,
) -> String {
    let primary_driver = [
        ("contenido", breakdown.content_similarity),
        ("red CNA", breakdown.cascade_score),
        ("gravedad EGA", breakdown.ega_score),
    ]
    .into_iter()
    .max_by(|left, right| left.1.total_cmp(&right.1))
    .map(|(label, contribution)| format!("El mayor aporte es {label} ({contribution:.3})."));
    let primary_driver = match primary_driver {
        Some(explanation) => explanation,
        None => "No hay señales suficientes para explicar el resultado.".to_owned(),
    };

    match evidence {
        Some(evidence) => format!(
            "{primary_driver} Se encontró evidencia local de tipo {} a {:.1} km; \
             referencia de fuente {} (capturada {}, consentimiento {}).",
            evidence.evidence_type,
            evidence.distance_km,
            evidence.source_hash,
            evidence.captured_at,
            evidence.consent_id
        ),
        None => format!(
            "{primary_driver} No hay evidencia OSINT consentida y auditable para esta zona."
        ),
    }
}

#[derive(Clone, Copy)]
struct BoundingBox {
    min_latitude: f64,
    max_latitude: f64,
    min_longitude: f64,
    max_longitude: f64,
    all_longitudes: bool,
    crosses_antimeridian: bool,
}

impl BoundingBox {
    fn around(latitude: f64, longitude: f64, radius_km: f64) -> Self {
        let latitude_delta = radius_km / 111.32;
        let min_latitude = (latitude - latitude_delta).max(-90.0);
        let max_latitude = (latitude + latitude_delta).min(90.0);
        let longitude_scale = 111.32 * latitude.to_radians().cos().abs();
        let longitude_delta = if longitude_scale < 1e-9 {
            360.0
        } else {
            let max_absolute_latitude = latitude
                .abs()
                .max(min_latitude.abs())
                .max(max_latitude.abs());
            let conservative_scale = 111.32 * max_absolute_latitude.to_radians().cos();
            if conservative_scale < 1e-9 {
                360.0
            } else {
                radius_km / conservative_scale
            }
        };
        let all_longitudes = longitude_delta >= 180.0;
        let crosses_antimeridian =
            !all_longitudes && (longitude - longitude_delta < -180.0
                || longitude + longitude_delta > 180.0);
        let min_longitude = longitude - longitude_delta;
        let max_longitude = longitude + longitude_delta;
        let (min_longitude, max_longitude) = if crosses_antimeridian
            && min_longitude < -180.0
        {
            (min_longitude + 360.0, max_longitude)
        } else if crosses_antimeridian && max_longitude > 180.0 {
            (min_longitude, max_longitude - 360.0)
        } else {
            (min_longitude, max_longitude)
        };

        Self {
            min_latitude,
            max_latitude,
            min_longitude,
            max_longitude,
            all_longitudes,
            crosses_antimeridian,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_or_empty_requests() {
        assert!(validate_request(&MatchRequest {
            client_id: Uuid::nil(),
            top_k: 0,
            max_distance_km: 10.0,
        })
        .is_err());
        assert!(validate_request(&MatchRequest {
            client_id: Uuid::nil(),
            top_k: MAX_TOP_K + 1,
            max_distance_km: 10.0,
        })
        .is_err());
        assert!(validate_request(&MatchRequest {
            client_id: Uuid::nil(),
            top_k: 10,
            max_distance_km: f64::NAN,
        })
        .is_err());
    }

    #[test]
    fn bounding_box_wraps_longitudes_at_antimeridian() {
        let bounds = BoundingBox::around(0.0, 179.9, 50.0);
        assert!(bounds.crosses_antimeridian);
        assert!(!bounds.all_longitudes);
        assert!(bounds.min_longitude > 0.0);
        assert!(bounds.max_longitude < 0.0);
    }

    #[test]
    fn haversine_handles_same_point_and_antimeridian() {
        assert!(haversine_distance_km(19.4, -99.1, 19.4, -99.1)
            .expect("valid coordinates")
            .abs()
            < 1e-9);
        let distance = haversine_distance_km(0.0, 179.9, 0.0, -179.9)
            .expect("valid coordinates");
        assert!((22.0..23.0).contains(&distance));
    }
}
