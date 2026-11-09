use std::{collections::HashSet, sync::Mutex};

use chrono::Utc;
use hmac::{Hmac, Mac};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tauri::State;
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::AppError;

type HmacSha256 = Hmac<Sha256>;
const MIGRATION: &str = include_str!("../../migrations/002_data_entry.sql");
const MAX_DOCUMENTS: usize = 20;
const MAX_DOCUMENT_PATH_LENGTH: usize = 4096;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Consentimientos capturados de forma explícita para el cliente.
pub struct ConsentimientoFlags {
    pub privacy_notice: bool,
    pub osint: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Datos temporales del formulario; la CURP se elimina antes de persistirlos.
pub struct ClientFormState {
    pub id: Option<Uuid>,
    pub nombre: String,
    pub apellido_paterno: String,
    pub apellido_materno: String,
    pub telefono: String,
    pub email: String,
    pub curp: String,
    pub presupuesto: String,
    #[serde(default)]
    pub zona_interes: String,
    pub tipo_credito: String,
    pub empresa: String,
    pub ingreso_mensual: String,
    pub edad: String,
    pub edad_laboral: String,
    pub notas: String,
    pub consentimientos: ConsentimientoFlags,
}

impl Drop for ClientFormState {
    fn drop(&mut self) {
        self.curp.zeroize();
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Datos de una propiedad y referencias a documentos que permanecen locales.
pub struct PropertyFormState {
    pub id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub tipo: String,
    pub precio: String,
    pub recamaras: String,
    pub banos: String,
    pub niveles: String,
    pub calle: String,
    pub colonia: String,
    pub codigo_postal: String,
    pub municipio: String,
    pub estado: String,
    pub latitude: String,
    pub longitude: String,
    pub notas: String,
    pub document_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Solicitud validada para guardar un cliente o un borrador local.
pub struct SaveClientRequest {
    pub client: ClientFormState,
    pub draft: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Solicitud validada para guardar una propiedad o un borrador local.
pub struct SavePropertyRequest {
    pub property: PropertyFormState,
    pub draft: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Identificador local y fecha de confirmación de una escritura.
pub struct SaveRecordResponse {
    pub id: Uuid,
    pub draft: bool,
    pub saved_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Sugerencia devuelta por el catálogo postal cifrado.
pub struct LocationSuggestion {
    pub postal_code: String,
    pub colonia: String,
    pub municipio: String,
    pub estado: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Resumen local sin información sensible como CURP o datos de contacto.
pub struct DraftSummary {
    pub id: Uuid,
    pub kind: String,
    pub label: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Colección acotada de borradores disponibles para recuperar.
pub struct DraftListResponse {
    pub drafts: Vec<DraftSummary>,
}

/// Almacén local que requiere una conexión SQLCipher abierta y una clave HMAC protegida.
pub struct ClientDataStore {
    connection: Mutex<Connection>,
    curp_hmac_key: Mutex<Zeroizing<[u8; 32]>>,
}

impl ClientDataStore {
    /// Desbloquea y prepara una conexión SQLCipher antes de exponer el almacén.
    ///
    /// El host debe obtener ambas claves de un almacén seguro del sistema.
    /// La conexión no debe haberse usado ni compartido antes de esta llamada.
    pub fn initialize(
        connection: Connection,
        database_key: &str,
        curp_hmac_key: [u8; 32],
    ) -> Result<Self, AppError> {
        if database_key.is_empty() {
            return Err(AppError::EncryptionUnavailable);
        }
        let cipher_version = connection
            .query_row("PRAGMA cipher_version", [], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|_| AppError::EncryptionUnavailable)?;
        if cipher_version.as_deref().map_or(true, str::is_empty) {
            return Err(AppError::EncryptionUnavailable);
        }

        connection
            .pragma_update(None, "key", database_key)
            .map_err(|_| AppError::EncryptionUnavailable)?;
        connection
            .query_row("SELECT count(*) FROM sqlite_master", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|_| AppError::EncryptionUnavailable)?;
        connection
            .execute_batch(MIGRATION)
            .map_err(|_| AppError::DatabaseUnavailable)?;
        Ok(Self {
            connection: Mutex::new(connection),
            curp_hmac_key: Mutex::new(Zeroizing::new(curp_hmac_key)),
        })
    }

    fn save_client(&self, mut request: SaveClientRequest) -> Result<SaveRecordResponse, AppError> {
        validate_client(&request.client, request.draft)?;
        let client_id = request.client.id.unwrap_or_else(Uuid::now_v7);
        let curp_hmac = if request.client.curp.trim().is_empty() {
            None
        } else {
            let normalized_curp = Zeroizing::new(normalize_curp(&request.client.curp));
            let key = self
                .curp_hmac_key
                .lock()
                .map_err(|_| AppError::EncryptionUnavailable)?;
            let mut mac = HmacSha256::new_from_slice(&key[..])
                .map_err(|_| AppError::EncryptionUnavailable)?;
            mac.update(normalized_curp.as_bytes());
            Some(mac.finalize().into_bytes().to_vec())
        };
        request.client.curp.zeroize();

        let consent_json = serde_json::to_string(&request.client.consentimientos)
            .map_err(|_| AppError::InvalidInput)?;
        let data_json = serde_json::to_string(&request.client)
            .map_err(|_| AppError::InvalidInput)?;
        let saved_at = Utc::now().to_rfc3339();
        let status = if request.draft { "draft" } else { "saved" };
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let existed = transaction
            .query_row(
                "SELECT 1 FROM client_records WHERE client_id = ?1",
                [client_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|_| AppError::DatabaseUnavailable)?
            .is_some();
        transaction
            .execute(
                "INSERT INTO client_records
                    (client_id, data_json, curp_hmac, consent_json, status, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(client_id) DO UPDATE SET
                    data_json = excluded.data_json,
                    curp_hmac = COALESCE(excluded.curp_hmac, client_records.curp_hmac),
                    consent_json = excluded.consent_json,
                    status = CASE
                        WHEN client_records.status = 'saved' THEN 'saved'
                        ELSE excluded.status
                    END,
                    updated_at = excluded.updated_at",
                params![
                    client_id.to_string(),
                    data_json,
                    curp_hmac,
                    consent_json,
                    status,
                    saved_at
                ],
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let is_draft = transaction
            .query_row(
                "SELECT status = 'draft' FROM client_records WHERE client_id = ?1",
                [client_id.to_string()],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        write_audit(
            &transaction,
            "client",
            client_id,
            if existed {
                "updated"
            } else if request.draft {
                "draft_saved"
            } else {
                "created"
            },
            &saved_at,
        )?;
        transaction
            .commit()
            .map_err(|_| AppError::DatabaseUnavailable)?;

        Ok(SaveRecordResponse {
            id: client_id,
            draft: is_draft,
            saved_at,
        })
    }

    fn save_property(
        &self,
        request: SavePropertyRequest,
    ) -> Result<SaveRecordResponse, AppError> {
        validate_property(&request.property, request.draft)?;
        let property_id = request.property.id.unwrap_or_else(Uuid::now_v7);
        let property_json =
            serde_json::to_string(&request.property).map_err(|_| AppError::InvalidInput)?;
        let document_paths_json = serde_json::to_string(&request.property.document_paths)
            .map_err(|_| AppError::InvalidInput)?;
        let saved_at = Utc::now().to_rfc3339();
        let status = if request.draft { "draft" } else { "saved" };
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let existed = transaction
            .query_row(
                "SELECT 1 FROM property_records WHERE property_id = ?1",
                [property_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(|_| AppError::DatabaseUnavailable)?
            .is_some();
        transaction
            .execute(
                "INSERT INTO property_records
                    (property_id, client_id, data_json, document_paths_json, status, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(property_id) DO UPDATE SET
                    client_id = excluded.client_id,
                    data_json = excluded.data_json,
                    document_paths_json = excluded.document_paths_json,
                    status = CASE
                        WHEN property_records.status = 'saved' THEN 'saved'
                        ELSE excluded.status
                    END,
                    updated_at = excluded.updated_at",
                params![
                    property_id.to_string(),
                    request.property.client_id.map(|id| id.to_string()),
                    property_json,
                    document_paths_json,
                    status,
                    saved_at
                ],
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let is_draft = transaction
            .query_row(
                "SELECT status = 'draft' FROM property_records WHERE property_id = ?1",
                [property_id.to_string()],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        write_audit(
            &transaction,
            "property",
            property_id,
            if existed {
                "updated"
            } else if request.draft {
                "draft_saved"
            } else {
                "created"
            },
            &saved_at,
        )?;
        transaction
            .commit()
            .map_err(|_| AppError::DatabaseUnavailable)?;

        Ok(SaveRecordResponse {
            id: property_id,
            draft: is_draft,
            saved_at,
        })
    }

    fn suggest_locations(&self, postal_code: &str) -> Result<Vec<LocationSuggestion>, AppError> {
        if postal_code.len() > 5 || !postal_code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(AppError::InvalidInput);
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let mut statement = connection
            .prepare(
                "SELECT postal_code, colonia, municipio, estado
                 FROM postal_code_catalog
                 WHERE postal_code LIKE ?1 || '%'
                 ORDER BY postal_code, colonia
                 LIMIT 30",
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let rows = statement
            .query_map([postal_code], |row| {
                Ok(LocationSuggestion {
                    postal_code: row.get(0)?,
                    colonia: row.get(1)?,
                    municipio: row.get(2)?,
                    estado: row.get(3)?,
                })
            })
            .map_err(|_| AppError::DatabaseUnavailable)?;
        rows.map(|row| row.map_err(|_| AppError::DatabaseUnavailable))
            .collect()
    }

    fn list_local_drafts(&self) -> Result<DraftListResponse, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let mut drafts = Vec::new();

        let mut clients = connection
            .prepare(
                "SELECT client_id, data_json, updated_at
                 FROM client_records WHERE status = 'draft'
                 ORDER BY updated_at DESC LIMIT 100",
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let client_rows = clients
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| AppError::DatabaseUnavailable)?;
        for row in client_rows {
            let (id, data_json, updated_at) =
                row.map_err(|_| AppError::DatabaseUnavailable)?;
            let client: ClientFormState =
                serde_json::from_str(&data_json).map_err(|_| AppError::DatabaseUnavailable)?;
            drafts.push(DraftSummary {
                id: Uuid::parse_str(&id).map_err(|_| AppError::DatabaseUnavailable)?,
                kind: "client".to_owned(),
                label: format!("{} {}", client.nombre, client.apellido_paterno)
                    .trim()
                    .to_owned(),
                updated_at,
            });
        }

        let mut properties = connection
            .prepare(
                "SELECT property_id, data_json, updated_at
                 FROM property_records WHERE status = 'draft'
                 ORDER BY updated_at DESC LIMIT 100",
            )
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let property_rows = properties
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| AppError::DatabaseUnavailable)?;
        for row in property_rows {
            let (id, data_json, updated_at) =
                row.map_err(|_| AppError::DatabaseUnavailable)?;
            let property: PropertyFormState =
                serde_json::from_str(&data_json).map_err(|_| AppError::DatabaseUnavailable)?;
            drafts.push(DraftSummary {
                id: Uuid::parse_str(&id).map_err(|_| AppError::DatabaseUnavailable)?,
                kind: "property".to_owned(),
                label: format!("{} · {}", property.tipo, property.colonia)
                    .trim()
                    .to_owned(),
                updated_at,
            });
        }
        drafts.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        drafts.truncate(100);

        Ok(DraftListResponse { drafts })
    }

    fn load_client_draft(&self, id: Uuid) -> Result<ClientFormState, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let data_json = connection
            .query_row(
                "SELECT data_json FROM client_records
                 WHERE client_id = ?1 AND status = 'draft'",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| AppError::DatabaseUnavailable)?
            .ok_or(AppError::InvalidInput)?;
        serde_json::from_str(&data_json).map_err(|_| AppError::DatabaseUnavailable)
    }

    fn load_property_draft(&self, id: Uuid) -> Result<PropertyFormState, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::DatabaseUnavailable)?;
        let data_json = connection
            .query_row(
                "SELECT data_json FROM property_records
                 WHERE property_id = ?1 AND status = 'draft'",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| AppError::DatabaseUnavailable)?
            .ok_or(AppError::InvalidInput)?;
        serde_json::from_str(&data_json).map_err(|_| AppError::DatabaseUnavailable)
    }
}

/// Guarda un cliente o borrador dentro de SQLCipher y almacena solo HMAC de CURP.
#[tauri::command]
pub fn save_client_record(
    request: SaveClientRequest,
    store: State<'_, ClientDataStore>,
) -> Result<SaveRecordResponse, AppError> {
    store.save_client(request)
}

/// Guarda una propiedad o borrador y sus rutas locales en SQLCipher.
#[tauri::command]
pub fn save_property_record(
    request: SavePropertyRequest,
    store: State<'_, ClientDataStore>,
) -> Result<SaveRecordResponse, AppError> {
    store.save_property(request)
}

/// Busca colonias y códigos postales en el catálogo local cifrado.
#[tauri::command]
pub fn suggest_locations(
    postal_code: String,
    store: State<'_, ClientDataStore>,
) -> Result<Vec<LocationSuggestion>, AppError> {
    store.suggest_locations(&postal_code)
}

/// Enumera borradores recientes cifrados para recuperación local.
#[tauri::command]
pub fn list_local_drafts(
    store: State<'_, ClientDataStore>,
) -> Result<DraftListResponse, AppError> {
    store.list_local_drafts()
}

/// Recupera un borrador de cliente sin recuperar su CURP original.
#[tauri::command]
pub fn load_client_draft(
    id: Uuid,
    store: State<'_, ClientDataStore>,
) -> Result<ClientFormState, AppError> {
    store.load_client_draft(id)
}

/// Recupera un borrador de propiedad y sus rutas locales.
#[tauri::command]
pub fn load_property_draft(
    id: Uuid,
    store: State<'_, ClientDataStore>,
) -> Result<PropertyFormState, AppError> {
    store.load_property_draft(id)
}

fn write_audit(
    transaction: &Transaction<'_>,
    entity_type: &str,
    entity_id: Uuid,
    status: &str,
    saved_at: &str,
) -> Result<(), AppError> {
    let action = status;
    transaction
        .execute(
            "INSERT INTO audit_log (id, entity_type, entity_id, action, occurred_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Uuid::now_v7().to_string(),
                entity_type,
                entity_id.to_string(),
                action,
                saved_at
            ],
        )
        .map_err(|_| AppError::DatabaseUnavailable)?;
    Ok(())
}

fn validate_client(client: &ClientFormState, draft: bool) -> Result<(), AppError> {
    if client.nombre.len() > 120
        || client.apellido_paterno.len() > 120
        || client.apellido_materno.len() > 120
        || client.telefono.len() > 30
        || client.email.len() > 254
        || client.curp.len() > 18
        || client.presupuesto.len() > 32
        || client.zona_interes.len() > 160
        || client.ingreso_mensual.len() > 32
        || client.tipo_credito.len() > 40
        || client.empresa.len() > 200
        || client.edad.len() > 3
        || client.edad_laboral.len() > 3
        || client.notas.len() > 10_000
    {
        return Err(AppError::InvalidInput);
    }
    if !draft
        && (client.nombre.trim().is_empty()
            || client.apellido_paterno.trim().is_empty()
            || client.telefono.trim().is_empty()
            || client.presupuesto.trim().is_empty())
    {
        return Err(AppError::InvalidInput);
    }
    if !client.telefono.trim().is_empty()
        && (!client.telefono.chars().all(|character| {
            character.is_ascii_digit() || matches!(character, '+' | '(' | ')' | ' ' | '.' | '-')
        }) || client.telefono.chars().filter(char::is_ascii_digit).count() < 7)
    {
        return Err(AppError::InvalidInput);
    }
    if !client.curp.trim().is_empty() && !valid_curp(&client.curp) {
        return Err(AppError::InvalidInput);
    }
    if !client.email.trim().is_empty()
        && (!client.email.contains('@') || client.email.chars().any(char::is_whitespace))
    {
        return Err(AppError::InvalidInput);
    }
    if !draft {
        validate_optional_number(&client.presupuesto, 0.0, f64::MAX)?;
        validate_optional_number(&client.ingreso_mensual, 0.0, f64::MAX)?;
        validate_optional_number(&client.edad, 18.0, 120.0)?;
        validate_optional_number(&client.edad_laboral, 0.0, 100.0)?;
    }
    if !draft && !client.consentimientos.privacy_notice {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}

fn validate_property(property: &PropertyFormState, draft: bool) -> Result<(), AppError> {
    if property.tipo.len() > 40
        || property.precio.len() > 32
        || property.calle.len() > 300
        || property.colonia.len() > 120
        || property.codigo_postal.len() > 5
        || property.municipio.len() > 120
        || property.estado.len() > 120
        || property.latitude.len() > 32
        || property.longitude.len() > 32
        || property.notas.len() > 10_000
        || property.document_paths.len() > MAX_DOCUMENTS
        || property.document_paths.iter().any(|path| {
            path.is_empty()
                || path.len() > MAX_DOCUMENT_PATH_LENGTH
                || !std::path::Path::new(path).is_absolute()
                || path.contains('\0')
                || !matches!(
                    std::path::Path::new(path)
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("pdf" | "jpg" | "jpeg" | "png" | "webp" | "doc" | "docx")
                )
                || !std::path::Path::new(path).is_file()
        })
    {
        return Err(AppError::InvalidInput);
    }
    if !draft {
        let price = property
            .precio
            .parse::<f64>()
            .map_err(|_| AppError::InvalidInput)?;
        let latitude = property
            .latitude
            .parse::<f64>()
            .map_err(|_| AppError::InvalidInput)?;
        let longitude = property
            .longitude
            .parse::<f64>()
            .map_err(|_| AppError::InvalidInput)?;
        if !price.is_finite()
            || price <= 0.0
            || !latitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !longitude.is_finite()
            || !(-180.0..=180.0).contains(&longitude)
            || property.colonia.trim().is_empty()
            || property.calle.trim().is_empty()
            || property.municipio.trim().is_empty()
            || property.estado.trim().is_empty()
            || property.tipo.trim().is_empty()
            || !valid_postal_code(&property.codigo_postal)
        {
            return Err(AppError::InvalidInput);
        }
        validate_optional_number(&property.recamaras, 0.0, 100.0)?;
        validate_optional_number(&property.banos, 0.0, 100.0)?;
        validate_optional_number(&property.niveles, 0.0, 100.0)?;
    }
    Ok(())
}

fn validate_optional_number(
    value: &str,
    minimum: f64,
    maximum: f64,
) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Ok(());
    }
    let parsed = value.parse::<f64>().map_err(|_| AppError::InvalidInput)?;
    if !parsed.is_finite() || !(minimum..=maximum).contains(&parsed) {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}

fn normalize_curp(curp: &str) -> String {
    curp.trim().to_ascii_uppercase()
}

fn valid_curp(curp: &str) -> bool {
    let value = normalize_curp(curp);
    let bytes = value.as_bytes();
    if bytes.len() != 18
        || !bytes[0].is_ascii_alphabetic()
        || !matches!(bytes[1], b'A' | b'E' | b'I' | b'O' | b'U' | b'X')
        || !bytes[2..4].iter().all(u8::is_ascii_alphabetic)
        || !bytes[11..13].iter().all(u8::is_ascii_alphabetic)
        || !bytes[4..10].iter().all(u8::is_ascii_digit)
        || !matches!(bytes[10], b'H' | b'M')
        || !bytes[13..16].iter().all(|byte| {
            byte.is_ascii_alphabetic() && !matches!(byte, b'A' | b'E' | b'I' | b'O' | b'U')
        })
        || !bytes[16].is_ascii_alphanumeric()
        || !bytes[17].is_ascii_digit()
    {
        return false;
    }

    let month = u16::from(bytes[6] - b'0') * 10 + u16::from(bytes[7] - b'0');
    let day = u16::from(bytes[8] - b'0') * 10 + u16::from(bytes[9] - b'0');
    let state = &value[11..13];
    (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && matches!(
            state,
            "AS" | "BC" | "BS" | "CC" | "CL" | "CM" | "CS" | "CH" | "DF" | "DG"
                | "GT" | "GR" | "HG" | "JC" | "MC" | "MN" | "MS" | "NT" | "NL"
                | "OC" | "PL" | "QT" | "QR" | "SP" | "SL" | "SR" | "TC" | "TS"
                | "TL" | "VZ" | "YN" | "ZS"
        )
}

fn valid_postal_code(postal_code: &str) -> bool {
    postal_code.len() == 5 && postal_code.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_client() -> ClientFormState {
        ClientFormState {
            id: None,
            nombre: "Ana".to_owned(),
            apellido_paterno: "Lopez".to_owned(),
            apellido_materno: String::new(),
            telefono: "5551234567".to_owned(),
            email: String::new(),
            curp: String::new(),
            presupuesto: "0".to_owned(),
            zona_interes: String::new(),
            tipo_credito: String::new(),
            empresa: String::new(),
            ingreso_mensual: String::new(),
            edad: String::new(),
            edad_laboral: String::new(),
            notas: String::new(),
            consentimientos: ConsentimientoFlags {
                privacy_notice: true,
                osint: false,
            },
        }
    }

    #[test]
    fn validates_curp_format_without_accepting_bad_characters() {
        assert!(valid_curp("GODE561231HDFRRN09"));
        assert!(valid_curp("gode561231hdfrrn09"));
        assert!(!valid_curp("GODE561231HDFRRN0!"));
        assert!(!valid_curp("GODE561231XDFRRN09"));
        assert!(!valid_curp("GODE561231HDFRRN0"));
    }

    #[test]
    fn validates_mexican_postal_code_shape() {
        assert!(valid_postal_code("03100"));
        assert!(!valid_postal_code("3100"));
        assert!(!valid_postal_code("ABCDE"));
    }

    #[test]
    fn final_client_requires_budget_and_local_privacy_notice() {
        let mut client = valid_client();
        client.presupuesto.clear();
        assert!(validate_client(&client, false).is_err());

        client.presupuesto = "0".to_owned();
        client.consentimientos.privacy_notice = false;
        assert!(validate_client(&client, false).is_err());

        client.consentimientos.privacy_notice = true;
        assert!(validate_client(&client, false).is_ok());
    }

    #[test]
    fn client_drafts_allow_incomplete_required_fields() {
        let mut client = valid_client();
        client.presupuesto.clear();
        client.telefono.clear();
        client.consentimientos.privacy_notice = false;
        assert!(validate_client(&client, true).is_ok());
    }

    #[test]
    fn older_client_records_default_to_no_interest_zone() {
        let client: ClientFormState = serde_json::from_str(
            r#"{
                "id": null,
                "nombre": "Ana",
                "apellidoPaterno": "Lopez",
                "apellidoMaterno": "",
                "telefono": "",
                "email": "",
                "curp": "",
                "presupuesto": "",
                "tipoCredito": "",
                "empresa": "",
                "ingresoMensual": "",
                "edad": "",
                "edadLaboral": "",
                "notas": "",
                "consentimientos": { "privacyNotice": false, "osint": false }
            }"#,
        )
        .expect("registro previo debe seguir siendo legible");
        assert!(client.zona_interes.is_empty());
    }
}
