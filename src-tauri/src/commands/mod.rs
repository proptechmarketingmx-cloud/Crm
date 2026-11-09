use crate::{
    cascade::cascade_score,
    data_entry::{
        list_local_drafts, load_client_draft, load_property_draft, save_client_record,
        save_property_record, suggest_locations, ClientDataStore,
    },
    error::AppError,
    graph::communities,
    matching::{
        match_score,
        store::{
            get_client_network, get_ega_heatmap, recommend_properties, MatchingStore,
        },
    },
    models::{CommunityAssignment, PredictionRequest, PredictionResult},
};

/// Registra los comandos locales y su conexión con sqlite-vec ya inicializada.
pub fn register<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    matching_store: MatchingStore,
    client_data_store: ClientDataStore,
) -> tauri::Builder<R> {
    builder
        .manage(matching_store)
        .manage(client_data_store)
        .invoke_handler(tauri::generate_handler![
            predict_match,
            detect_communities,
            recommend_properties,
            get_client_network,
            get_ega_heatmap,
            save_client_record,
            save_property_record,
            suggest_locations,
            list_local_drafts,
            load_client_draft,
            load_property_draft
        ])
}

/// Calcula score CNA y matching híbrido sin registrar IDs ni datos de entrada.
#[tauri::command]
pub fn predict_match(request: PredictionRequest) -> Result<PredictionResult, AppError> {
    let cascade = cascade_score(
        request.client_id,
        &request.event,
        &request.edges,
        &request.activities,
        request.ega_score,
        request.delta_t,
        request.parameters,
    )?;
    let matching = match_score(
        &request.client_embedding,
        &request.property_embedding,
        cascade.raw_score,
        request.ega_score,
    )?;

    Ok(PredictionResult {
        cascade_score: cascade.raw_score,
        purchase_probability: cascade.probability,
        network_contribution: cascade.network_contribution,
        geographic_contribution: cascade.geographic_contribution,
        temporal_contribution: cascade.temporal_contribution,
        cosine_similarity: matching.cosine_similarity,
        match_score: matching.score,
    })
}

/// Devuelve asignaciones locales de comunidad sin emitir datos en logs.
#[tauri::command]
pub fn detect_communities(
    edges: Vec<crate::models::GraphEdge>,
) -> Result<Vec<CommunityAssignment>, AppError> {
    communities::detect_communities(&edges)
}
