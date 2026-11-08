use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub weight: f64,
    #[serde(default)]
    pub signals: Vec<Signal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Signal {
    SameSurname { score: f64 },
    SameEmployer { employer: String },
    SameVisitZone { zone_id: Uuid },
    ConfirmedInteraction { interaction: InteractionType },
    ProfileSimilarity { cosine: f64 },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InteractionType {
    Call,
    Message,
    Visit,
    Referral,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityType {
    Purchase,
    Visit,
    Search,
}

impl ActivityType {
    pub fn intensity(self) -> f64 {
        match self {
            Self::Purchase => 1.0,
            Self::Visit => 0.3,
            Self::Search => 0.1,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientActivity {
    pub client_id: Uuid,
    pub activity: ActivityType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseEvent {
    pub client_id: Uuid,
    pub activity: ActivityType,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionRequest {
    pub client_id: Uuid,
    pub event: PurchaseEvent,
    pub edges: Vec<GraphEdge>,
    #[serde(default)]
    pub activities: Vec<ClientActivity>,
    pub client_embedding: Vec<f32>,
    pub property_embedding: Vec<f32>,
    pub ega_score: f64,
    pub delta_t: f64,
    #[serde(default)]
    pub parameters: CascadeParameters,
}

impl Drop for PredictionRequest {
    fn drop(&mut self) {
        self.client_embedding.zeroize();
        self.property_embedding.zeroize();
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeParameters {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub lambda: f64,
}

impl Default for CascadeParameters {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.3,
            gamma: 0.2,
            lambda: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionResult {
    pub cascade_score: f64,
    pub purchase_probability: f64,
    pub network_contribution: f64,
    pub geographic_contribution: f64,
    pub temporal_contribution: f64,
    pub cosine_similarity: f64,
    pub match_score: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityAssignment {
    pub client_id: Uuid,
    pub community_id: usize,
}
