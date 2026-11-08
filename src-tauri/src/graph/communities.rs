use std::collections::{BTreeMap, BTreeSet, HashMap};

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CommunityAssignment, GraphEdge},
};

/// Detecta comunidades mediante la fase local de optimización de modularidad
/// ponderada usada por Louvain. No realiza todavía agregación multinivel.
pub fn detect_communities(edges: &[GraphEdge]) -> Result<Vec<CommunityAssignment>, AppError> {
    let mut nodes = BTreeSet::new();
    let mut adjacency: HashMap<Uuid, Vec<(Uuid, f64)>> = HashMap::new();
    let mut seen_edges = BTreeSet::new();
    let mut total_edge_weight = 0.0;

    for edge in edges {
        if edge.source == edge.target
            || !edge.weight.is_finite()
            || !(0.0..=1.0).contains(&edge.weight)
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
        nodes.insert(edge.source);
        nodes.insert(edge.target);
        adjacency
            .entry(edge.source)
            .or_default()
            .push((edge.target, edge.weight));
        adjacency
            .entry(edge.target)
            .or_default()
            .push((edge.source, edge.weight));
        total_edge_weight += edge.weight;
    }

    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    let mut community_by_node = nodes
        .iter()
        .enumerate()
        .map(|(community_id, node)| (*node, community_id))
        .collect::<HashMap<_, _>>();
    if total_edge_weight == 0.0 {
        return Ok(nodes
            .into_iter()
            .enumerate()
            .map(|(community_id, client_id)| CommunityAssignment {
                client_id,
                community_id,
            })
            .collect());
    }

    let degree_by_node = nodes
        .iter()
        .map(|node| {
            let degree = adjacency
                .get(node)
                .into_iter()
                .flatten()
                .map(|(_, weight)| weight)
                .sum::<f64>();
            (*node, degree)
        })
        .collect::<HashMap<_, _>>();
    let mut total_degree_by_community = degree_by_node
        .iter()
        .map(|(node, degree)| (community_by_node[node], *degree))
        .fold(
            HashMap::<usize, f64>::new(),
            |mut totals, (community, degree)| {
                *totals.entry(community).or_default() += degree;
                totals
            },
        );

    let node_order = nodes.iter().copied().collect::<Vec<_>>();
    for _ in 0..100 {
        let mut moved = false;
        for node in &node_order {
            let degree = degree_by_node[node];
            let old_community = community_by_node[node];
            *total_degree_by_community
                .entry(old_community)
                .or_default() -= degree;

            let mut weight_by_community = BTreeMap::<usize, f64>::new();
            for (neighbor, weight) in adjacency.get(node).into_iter().flatten() {
                *weight_by_community
                    .entry(community_by_node[neighbor])
                    .or_default() += weight;
            }
            weight_by_community.entry(old_community).or_default();

            let mut best_community = old_community;
            let mut best_gain = 0.0;
            for (community, weight_to_community) in weight_by_community {
                let community_degree = total_degree_by_community
                    .get(&community)
                    .copied()
                    .unwrap_or_default();
                let gain = weight_to_community
                    - degree * community_degree / (2.0 * total_edge_weight);
                if gain > best_gain + 1e-12
                    || ((gain - best_gain).abs() <= 1e-12
                        && gain > 1e-12
                        && community < best_community)
                {
                    best_gain = gain;
                    best_community = community;
                }
            }

            community_by_node.insert(*node, best_community);
            *total_degree_by_community
                .entry(best_community)
                .or_default() += degree;
            moved |= best_community != old_community;
        }
        if !moved {
            break;
        }
    }

    let mut compact_ids = BTreeMap::new();
    let mut assignments = Vec::with_capacity(nodes.len());
    for node in nodes {
        let community = community_by_node[&node];
        let next_id = compact_ids.len();
        let community_id = *compact_ids.entry(community).or_insert(next_id);
        assignments.push(CommunityAssignment {
            client_id: node,
            community_id,
        });
    }
    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn groups_two_dense_pairs_separately() {
        let first = [Uuid::new_v4(), Uuid::new_v4()];
        let second = [Uuid::new_v4(), Uuid::new_v4()];
        let edges = vec![
            edge(first[0], first[1], 1.0),
            edge(second[0], second[1], 1.0),
            edge(first[1], second[0], 0.01),
        ];

        let assignments = detect_communities(&edges).expect("valid graph");
        let by_client = assignments
            .into_iter()
            .map(|assignment| (assignment.client_id, assignment.community_id))
            .collect::<HashMap<_, _>>();

        assert_eq!(by_client[&first[0]], by_client[&first[1]]);
        assert_eq!(by_client[&second[0]], by_client[&second[1]]);
        assert_ne!(by_client[&first[0]], by_client[&second[0]]);
    }

    fn edge(source: Uuid, target: Uuid, weight: f64) -> GraphEdge {
        GraphEdge {
            source,
            target,
            weight,
            signals: Vec::new(),
        }
    }
}
