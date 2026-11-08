use std::collections::{HashMap, VecDeque};

use petgraph::graph::{NodeIndex, UnGraph};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CascadeParameters, ClientActivity, GraphEdge, PurchaseEvent, Signal,
    },
};

pub struct CascadeResult {
    pub raw_score: f64,
    pub probability: f64,
    pub network_contribution: f64,
    pub geographic_contribution: f64,
    pub temporal_contribution: f64,
}

/// Calcula la propagación local desde la red del cliente.
///
/// Para relaciones indirectas, `W_ij` es el producto de pesos del camino de
/// menor número de saltos; los empates usan el camino con mayor producto.
/// El score conserva la ecuación original. La conversión a probabilidad se
/// limita por separado porque una suma de varios vecinos puede exceder uno.
pub fn cascade_score(
    client_id: Uuid,
    event: &PurchaseEvent,
    edges: &[GraphEdge],
    activities: &[ClientActivity],
    ega_score: f64,
    delta_t: f64,
    parameters: CascadeParameters,
) -> Result<CascadeResult, AppError> {
    validate_inputs(
        client_id,
        event,
        edges,
        activities,
        ega_score,
        delta_t,
        parameters,
    )?;

    let mut graph = UnGraph::<Uuid, f64>::new_undirected();
    let mut indices = HashMap::<Uuid, NodeIndex>::new();
    let mut add_node = |client_id| {
        *indices
            .entry(client_id)
            .or_insert_with(|| graph.add_node(client_id))
    };
    add_node(client_id);
    for edge in edges {
        let source = add_node(edge.source);
        let target = add_node(edge.target);
        graph.add_edge(source, target, edge.weight);
    }

    let source = indices
        .get(&client_id)
        .copied()
        .ok_or(AppError::CalculationFailed)?;
    let (index_distances, index_path_weights) = shortest_influence_paths(source, &graph);
    let distances = index_distances
        .into_iter()
        .map(|(index, distance)| (graph[index], distance))
        .collect::<HashMap<_, _>>();
    let path_weights = index_path_weights
        .into_iter()
        .map(|(index, weight)| (graph[index], weight))
        .collect::<HashMap<_, _>>();
    let mut activity_by_client = HashMap::with_capacity(activities.len() + 1);
    for activity in activities {
        activity_by_client.insert(activity.client_id, activity.activity.intensity());
    }
    activity_by_client.insert(event.client_id, event.activity.intensity());

    let network_sum = activity_by_client
        .iter()
        .filter_map(|(peer_id, intensity)| {
            if *peer_id == client_id {
                return None;
            }
            let distance = *distances.get(peer_id)?;
            let path_weight = *path_weights.get(peer_id)?;
            Some(path_weight * intensity * (-parameters.lambda * f64::from(distance)).exp())
        })
        .sum::<f64>();

    let network_contribution = parameters.alpha * network_sum;
    let geographic_contribution = parameters.beta * ega_score;
    let temporal_contribution = parameters.gamma * (-parameters.lambda * delta_t).exp();
    let raw_score = network_contribution + geographic_contribution + temporal_contribution;

    if !raw_score.is_finite() {
        return Err(AppError::CalculationFailed);
    }

    Ok(CascadeResult {
        raw_score,
        probability: raw_score.clamp(0.0, 1.0),
        network_contribution,
        geographic_contribution,
        temporal_contribution,
    })
}

fn shortest_influence_paths(
    source: NodeIndex,
    graph: &UnGraph<Uuid, f64>,
) -> (HashMap<NodeIndex, u32>, HashMap<NodeIndex, f64>) {
    let mut distances = HashMap::from([(source, 0)]);
    let mut path_weights = HashMap::from([(source, 1.0)]);
    let mut pending = VecDeque::from([source]);

    while let Some(current) = pending.pop_front() {
        let current_distance = distances[&current];
        let current_weight = path_weights[&current];
        for neighbor in graph.neighbors(current) {
            let Some(edge_weight) = graph
                .find_edge(current, neighbor)
                .and_then(|edge| graph.edge_weight(edge))
            else {
                continue;
            };
            let next_distance = current_distance + 1;
            let next_weight = current_weight * edge_weight;
            match distances.get(&neighbor) {
                None => {
                    distances.insert(neighbor, next_distance);
                    path_weights.insert(neighbor, next_weight);
                    pending.push_back(neighbor);
                }
                Some(existing_distance) if *existing_distance == next_distance => {
                    let best_weight = path_weights.entry(neighbor).or_default();
                    if next_weight > *best_weight {
                        *best_weight = next_weight;
                    }
                }
                _ => {}
            }
        }
    }

    (distances, path_weights)
}

fn validate_inputs(
    client_id: Uuid,
    event: &PurchaseEvent,
    edges: &[GraphEdge],
    activities: &[ClientActivity],
    ega_score: f64,
    delta_t: f64,
    parameters: CascadeParameters,
) -> Result<(), AppError> {
    if event.client_id == client_id
        || !ega_score.is_finite()
        || !(0.0..=1.0).contains(&ega_score)
        || !delta_t.is_finite()
        || delta_t < 0.0
        || !valid_parameters(parameters)
    {
        return Err(AppError::InvalidInput);
    }

    let mut seen_edges = std::collections::HashSet::with_capacity(edges.len());
    for edge in edges {
        if edge.source == edge.target
            || !edge.weight.is_finite()
            || !(0.0..=1.0).contains(&edge.weight)
            || edge.signals.iter().any(|signal| match signal {
                Signal::SameSurname { score } => {
                    !score.is_finite() || !(0.0..=1.0).contains(score)
                }
                Signal::ProfileSimilarity { cosine } => {
                    !cosine.is_finite() || !(-1.0..=1.0).contains(cosine)
                }
                Signal::SameEmployer { employer } => employer.trim().is_empty(),
                Signal::SameVisitZone { .. } | Signal::ConfirmedInteraction { .. } => false,
            })
        {
            return Err(AppError::InvalidInput);
        }
        let pair = if edge.source < edge.target {
            (edge.source, edge.target)
        } else {
            (edge.target, edge.source)
        };
        if !seen_edges.insert(pair) {
            return Err(AppError::InvalidInput);
        }
    }

    let mut seen_activities = std::collections::HashSet::with_capacity(activities.len());
    for activity in activities {
        if activity.client_id == event.client_id
            || !seen_activities.insert(activity.client_id)
        {
            return Err(AppError::InvalidInput);
        }
    }

    Ok(())
}

fn valid_parameters(parameters: CascadeParameters) -> bool {
    let coefficients = [parameters.alpha, parameters.beta, parameters.gamma];
    coefficients
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
        && parameters.lambda.is_finite()
        && parameters.lambda >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ActivityType, GraphEdge, PurchaseEvent, Signal};

    #[test]
    fn applies_direct_edge_weight_and_exponential_decay() {
        let client = Uuid::new_v4();
        let buyer = Uuid::new_v4();
        let edge = GraphEdge {
            source: client,
            target: buyer,
            weight: 0.8,
            signals: vec![Signal::SameEmployer {
                employer: "Empresa local".into(),
            }],
        };

        let result = cascade_score(
            client,
            &PurchaseEvent {
                client_id: buyer,
                activity: ActivityType::Purchase,
            },
            &[edge],
            &[],
            0.0,
            0.0,
            CascadeParameters::default(),
        )
        .expect("valid fixture");

        let expected = 0.5 * (0.8 * (-0.5_f64).exp()) + 0.2;
        assert!((result.raw_score - expected).abs() < 1e-12);
        assert_eq!(result.probability, expected);
    }

    #[test]
    fn bounds_probability_when_several_neighbors_raise_raw_score() {
        let client = Uuid::new_v4();
        let buyer_one = Uuid::new_v4();
        let buyer_two = Uuid::new_v4();
        let edges = [buyer_one, buyer_two]
            .into_iter()
            .map(|buyer| GraphEdge {
                source: client,
                target: buyer,
                weight: 1.0,
                signals: Vec::new(),
            })
            .collect::<Vec<_>>();
        let activities = [buyer_one, buyer_two]
            .into_iter()
            .map(|client_id| ClientActivity {
                client_id,
                activity: ActivityType::Purchase,
            })
            .collect::<Vec<_>>();

        let result = cascade_score(
            client,
            &PurchaseEvent {
                client_id: Uuid::new_v4(),
                activity: ActivityType::Purchase,
            },
            &edges,
            &activities,
            1.0,
            0.0,
            CascadeParameters::default(),
        )
        .expect("valid fixture");

        assert!(result.raw_score > 1.0);
        assert_eq!(result.probability, 1.0);
    }
}
