use chrono::{DateTime, Utc};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{kosaraju_scc, connected_components};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crossbeam_channel::Sender;

#[derive(Debug, Clone)]
pub struct ReputationEvent {
    from_did: String,
    to_did: String,
    score: f64,
    timestamp: DateTime<Utc>,
    action_type: String,
    pub category: ReputationCategory,
    pub weight: f64,
    pub timestamp: u64,
    federation_id: Option<String>, 
    cross_federation_id: Option<String>, 
    event_type: ReputationEventType, 
    audit_proof: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum ReputationCategory {
    Governance,
    ResourceSharing,
    DisputeResolution,
    TechnicalContribution,
    CommunityEngagement,
}

pub enum ReputationEventType {
    Contribution,
    Governance,
    CrossFederationAction,
    DisputeResolution,
    ResourceSharing,
}

pub struct ReputationManager {
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

    pub fn calculate_reputation(&self, did: &str, category: Option<ReputationCategory>) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let events = match category {
            Some(cat) => self.get_events_by_category(did, cat),
            None => self.get_all_events(did),
        };

        events.iter()
            .map(|event| {
                let age = now - event.timestamp;
                let decay = self.calculate_decay(age);
                event.weight * decay 
            })
            .sum()
    }

    fn calculate_decay(&self, age: u64) -> f64 {
        // Dynamic decay rate based on activity level
        let base_decay = 0.1;
        let activity_modifier = self.get_activity_modifier();
        (1.0 - base_decay * activity_modifier).powf((age as f64) / (30.0 * 24.0 * 60.0 * 60.0))
    }

    pub async fn verify_cross_federation_action(&self, event: &ReputationEvent) -> Result<bool, ReputationError> {
        // Verify action is valid across both federations
        if let (Some(fed1), Some(fed2)) = (&event.federation_id, &event.cross_federation_id) {
            self.verify_federation_pair(fed1, fed2).await?;
            self.check_sybil_resistance(event).await?;
        }
        Ok(true)
    }

    pub fn adjust_federation_reputation(&mut self, federation_id: &str, change: f64) -> Result<(), ReputationError> {
        let current_score = self.federation_scores.entry(federation_id.to_string()).or_insert(0.0);
        *current_score += change;
        
        // Dynamic sybil threshold adjustment based on federation activity
        self.sybil_threshold = self.calculate_dynamic_threshold(federation_id);
        Ok(())
    }

    fn calculate_dynamic_threshold(&self, federation_id: &str) -> f64 {
        let base_threshold = 0.8;
        let activity_factor = self.get_federation_activity_score(federation_id);
        let network_health = self.calculate_network_health();
        
        base_threshold * activity_factor * network_health
    }

    fn detect_federation_collusion(&self, federation_id: &str) -> bool {
        let suspicious_patterns = self.analyze_cross_federation_patterns(federation_id);
        let rapid_growth = self.check_federation_growth_rate(federation_id);
        let voting_patterns = self.analyze_voting_patterns(federation_id);
        
        suspicious_patterns || rapid_growth || voting_patterns.is_suspicious
    }

    pub fn process_cross_federation_action(&mut self, from_fed: &str, to_fed: &str, action: &str) -> Result<(), ReputationError> {
        if self.detect_federation_collusion(from_fed) {
            self.adjust_federation_reputation(from_fed, -0.5)?;
            return Err(ReputationError::CollusionDetected);
        }
        
        // Process legitimate cross-federation action
        self.record_cross_federation_interaction(from_fed, to_fed, action);
        Ok(())
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
                category: ReputationCategory::Governance,
                weight: 1.0,
                timestamp: Utc::now().timestamp() as u64,
                federation_id: None,
                cross_federation_id: None,
                event_type: ReputationEventType::Governance,
                audit_proof: None,
            },
            ReputationEvent {
                from_did: "did:2".to_string(),
                to_did: "did:3".to_string(),
                score: 1.0,
                timestamp: Utc::now(),
                action_type: "endorse".to_string(),
                category: ReputationCategory::Governance,
                weight: 1.0,
                timestamp: Utc::now().timestamp() as u64,
                federation_id: None,
                cross_federation_id: None,
                event_type: ReputationEventType::Governance,
                audit_proof: None,
            },
            ReputationEvent {
                from_did: "did:3".to_string(),
                to_did: "did:1".to_string(),
                score: 1.0,
                timestamp: Utc::now(),
                action_type: "endorse".to_string(),
                category: ReputationCategory::Governance,
                weight: 1.0,
                timestamp: Utc::now().timestamp() as u64,
                federation_id: None,
                cross_federation_id: None,
                event_type: ReputationEventType::Governance,
                audit_proof: None,
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
            category: ReputationCategory::Governance,
            weight: 1.0,
            timestamp: (Utc::now() - chrono::Duration::days(10)).timestamp() as u64,
            federation_id: None,
            cross_federation_id: None,
            event_type: ReputationEventType::Governance,
            audit_proof: None,
        };

        manager.update_reputation(event).unwrap();
        
        // Score should be decayed after 10 days
        if let Some(score) = manager.scores.get(did) {
            assert!(*score < 1.0, "Score should decay over time");
        }
    }
}
