use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Los datos de entrada no son válidos.")]
    InvalidInput,
    #[error("No fue posible calcular la predicción.")]
    CalculationFailed,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::InvalidInput => "Los datos de entrada no son válidos.",
            Self::CalculationFailed => "No fue posible calcular la predicción.",
        })
    }
}
