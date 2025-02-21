use chrono::{DateTime, Utc};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{kosaraju_scc, connected_components};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

#[derive(Debug)]
pub struct ReputationEvent {
    from_did: String,
    to_did: String,
    score: f64,
    timestamp: DateTime<Utc>,
    action_type: String,
}

pub struct ReputationManager {
    // ...existing code...
    interaction_graph: DiGraph<String, f64>,
    node_indices: HashMap<String, NodeIndex>,
    last_update: HashMap<String, DateTime<Utc>>,
    suspicious_patterns: HashSet<String>,
    decay_rate: f64,
    sybil_threshold: f64,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            // ...existing code...
            interaction_graph: DiGraph::new(),
            node_indices: HashMap::new(),
            last_update: HashMap::new(),
            suspicious_patterns: HashSet::new(),
            decay_rate: 0.1, // 10% decay per day
            sybil_threshold: 0.8,
        }
    }

    pub fn update_reputation(&mut self, event: ReputationEvent) -> Result<(), ReputationError> {
        self.apply_time_decay(&event.to_did);
        self.update_interaction_graph(&event);
        
        if self.detect_sybil_pattern(&event) {
            self.suspicious_patterns.insert(event.from_did.clone());
            return Err(ReputationError::SuspiciousBehavior);
        }

        // Calculate adjusted score based on timing and graph metrics
        let adjusted_score = self.calculate_adjusted_score(&event);
        
        // Update reputation with adjusted score
        self.update_score(event.to_did, adjusted_score);
        
        Ok(())
    }

    fn apply_time_decay(&mut self, did: &str) {
        if let Some(last_update) = self.last_update.get(did) {
            let now = Utc::now();
            let days_elapsed = (now - *last_update).num_days() as f64;
            let decay_factor = (-self.decay_rate * days_elapsed).exp();
            
            if let Some(current_score) = self.scores.get_mut(did) {
                *current_score *= decay_factor;
            }
        }
        self.last_update.insert(did.to_string(), Utc::now());
    }

    fn update_interaction_graph(&mut self, event: &ReputationEvent) {
        // Add nodes if they don't exist
        let from_idx = self.get_or_create_node(&event.from_did);
        let to_idx = self.get_or_create_node(&event.to_did);

        // Update edge weight
        if let Some(edge) = self.interaction_graph.find_edge(from_idx, to_idx) {
            let weight = self.interaction_graph.edge_weight_mut(edge).unwrap();
            *weight += event.score;
        } else {
            self.interaction_graph.add_edge(from_idx, to_idx, event.score);
        }
    }

    fn get_or_create_node(&mut self, did: &str) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(did) {
            idx
        } else {
            let idx = self.interaction_graph.add_node(did.to_string());
            self.node_indices.insert(did.to_string(), idx);
            idx
        }
    }

    fn detect_sybil_pattern(&self, event: &ReputationEvent) -> bool {
        // Check for circular reputation boosting
        let cycles = kosaraju_scc(&self.interaction_graph);
        let suspicious_cycle = cycles.iter().any(|component| {
            component.len() > 2 && 
            component.contains(&self.node_indices[&event.from_did])
        });

        // Check for rapid reputation accumulation
        let rapid_growth = self.check_rapid_growth(&event.from_did);

        // Check for unusual clustering
        let clustering_coefficient = self.calculate_clustering_coefficient(&event.from_did);
        let unusual_clustering = clustering_coefficient > self.sybil_threshold;

        suspicious_cycle || rapid_growth || unusual_clustering
    }

    fn check_rapid_growth(&self, did: &str) -> bool {
        if let Some(idx) = self.node_indices.get(did) {
            let incoming = self.interaction_graph.neighbors_directed(*idx, petgraph::Incoming);
            let recent_interactions: Vec<_> = incoming.collect();
            
            // Check if there are too many recent interactions
            recent_interactions.len() > 10 // Configurable threshold
        } else {
            false
        }
    }

    fn calculate_clustering_coefficient(&self, did: &str) -> f64 {
        if let Some(idx) = self.node_indices.get(did) {
            let neighbors: HashSet<_> = self.interaction_graph
                .neighbors(*idx)
                .collect();
            
            if neighbors.len() < 2 {
                return 0.0;
            }

            let mut connections = 0;
            let possible_connections = (neighbors.len() * (neighbors.len() - 1)) / 2;

            for n1 in neighbors.iter() {
                for n2 in neighbors.iter() {
                    if n1 != n2 && self.interaction_graph.contains_edge(*n1, *n2) {
                        connections += 1;
                    }
                }
            }

            connections as f64 / possible_connections as f64
        } else {
            0.0
        }
    }

    fn calculate_adjusted_score(&self, event: &ReputationEvent) -> f64 {
        let base_score = event.score;
        let time_factor = self.calculate_time_factor(&event.timestamp);
        let graph_factor = self.calculate_graph_factor(&event.from_did);
        
        base_score * time_factor * graph_factor
    }

    fn calculate_time_factor(&self, timestamp: &DateTime<Utc>) -> f64 {
        let age = Utc::now() - *timestamp;
        let days = age.num_days() as f64;
        (-self.decay_rate * days).exp()
    }

    fn calculate_graph_factor(&self, did: &str) -> f64 {
        if self.suspicious_patterns.contains(did) {
            0.5 // Reduce impact of suspicious DIDs
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sybil_detection() {
        let mut manager = ReputationManager::new();
        
        // Create a circular reputation boosting pattern
        let events = vec![
            ReputationEvent {
                from_did: "did:1".to_string(),
                to_did: "did:2".to_string(),
                score: 1.0,
                timestamp: Utc::now(),
                action_type: "endorse".to_string(),
            },
            ReputationEvent {
                from_did: "did:2".to_string(),
                to_did: "did:3".to_string(),
                score: 1.0,
                timestamp: Utc::now(),
                action_type: "endorse".to_string(),
            },
            ReputationEvent {
                from_did: "did:3".to_string(),
                to_did: "did:1".to_string(),
                score: 1.0,
                timestamp: Utc::now(),
                action_type: "endorse".to_string(),
            },
        ];

        for event in events {
            let result = manager.update_reputation(event);
            assert!(result.is_err(), "Should detect Sybil pattern");
        }
    }

    #[test]
    fn test_time_decay() {
        let mut manager = ReputationManager::new();
        let did = "did:test";
        
        let event = ReputationEvent {
            from_did: "did:other".to_string(),
            to_did: did.to_string(),
            score: 1.0,
            timestamp: Utc::now() - chrono::Duration::days(10),
            action_type: "endorse".to_string(),
        };

        manager.update_reputation(event).unwrap();
        
        // Score should be decayed after 10 days
        if let Some(score) = manager.scores.get(did) {
            assert!(*score < 1.0, "Score should decay over time");
        }
    }
}
