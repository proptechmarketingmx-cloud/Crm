use crate::error::AppError;

mod queries;
pub mod store;

pub const EMBEDDING_DIMENSION: usize = 384;

#[derive(Clone, Copy, Debug)]
pub struct MatchResult {
    pub cosine_similarity: f64,
    pub score: f64,
}

/// Combina las tres señales con los coeficientes del producto.
pub fn match_score(
    client_embedding: &[f32],
    property_embedding: &[f32],
    cascade_score: f64,
    ega_score: f64,
) -> Result<MatchResult, AppError> {
    if client_embedding.len() != EMBEDDING_DIMENSION
        || client_embedding.len() != property_embedding.len()
        || client_embedding.iter().any(|value| !value.is_finite())
        || property_embedding.iter().any(|value| !value.is_finite())
        || !cascade_score.is_finite()
        || !ega_score.is_finite()
        || !(0.0..=1.0).contains(&ega_score)
    {
        return Err(AppError::InvalidInput);
    }

    let dot = client_embedding
        .iter()
        .zip(property_embedding)
        .map(|(left, right)| f64::from(*left) * f64::from(*right))
        .sum::<f64>();
    let client_norm = client_embedding
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    let property_norm = property_embedding
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    if client_norm == 0.0 || property_norm == 0.0 {
        return Err(AppError::InvalidInput);
    }

    let cosine_similarity = (dot / (client_norm * property_norm)).clamp(-1.0, 1.0);
    let score = 0.4 * cosine_similarity + 0.3 * cascade_score + 0.3 * ega_score;
    if !score.is_finite() {
        return Err(AppError::CalculationFailed);
    }

    Ok(MatchResult {
        cosine_similarity,
        score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_weighted_match_score() {
        let client_embedding = embedding_with_first_value(1.0);
        let property_embedding = embedding_with_first_value(1.0);
        let result = match_score(&client_embedding, &property_embedding, 0.5, 0.8)
            .expect("valid embeddings");

        assert!((result.cosine_similarity - 1.0).abs() < 1e-12);
        assert!((result.score - 0.79).abs() < 1e-12);
    }

    #[test]
    fn rejects_zero_norm_embeddings() {
        let client_embedding = vec![0.0; EMBEDDING_DIMENSION];
        let property_embedding = embedding_with_first_value(1.0);
        assert!(match_score(&client_embedding, &property_embedding, 0.5, 0.8).is_err());
    }

    #[test]
    fn uses_unclamped_cascade_score_in_the_declared_formula() {
        let client_embedding = embedding_with_first_value(1.0);
        let property_embedding = embedding_with_first_value(1.0);
        let result = match_score(&client_embedding, &property_embedding, 1.5, 1.0)
            .expect("valid scores");
        assert!((result.score - 1.15).abs() < 1e-12);
    }

    fn embedding_with_first_value(value: f32) -> Vec<f32> {
        let mut embedding = vec![0.0; EMBEDDING_DIMENSION];
        embedding[0] = value;
        embedding
    }
}
