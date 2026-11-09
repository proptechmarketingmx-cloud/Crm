use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Los datos de entrada no son válidos.")]
    InvalidInput,
    #[error("No fue posible calcular la predicción.")]
    CalculationFailed,
    #[error("La base de datos local o sqlite-vec no está disponible.")]
    DatabaseUnavailable,
    #[error("La base de datos no está cifrada con SQLCipher.")]
    EncryptionUnavailable,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::InvalidInput => "Los datos de entrada no son válidos.",
            Self::CalculationFailed => "No fue posible calcular la predicción.",
            Self::DatabaseUnavailable => {
                "La base de datos local o sqlite-vec no está disponible."
            }
            Self::EncryptionUnavailable => "La base de datos no está cifrada con SQLCipher.",
        })
    }
}
