/// Dependency graph generation and analysis
/// Produces node/link data for D3.js visualization of card relationships
use crate::models::{ArchitectureModel, CardType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub card_type: String,
    pub group: usize,
}

/// Represents a link in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub distance: f64,
}

/// Complete dependency graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
    pub node_count: usize,
    pub link_count: usize,
    pub density: f64,
}

impl DependencyGraph {
    /// Generate dependency graph from architecture model
    pub fn generate(model: &ArchitectureModel) -> Self {
        // Collect all cards as nodes
        let mut nodes = Vec::new();
        let mut node_map = HashMap::new();

        // Group cards by type
        let mut type_groups = HashMap::new();
        for (idx, card_type) in [
            CardType::Driver,
            CardType::Requirement,
            CardType::Behavior,
            CardType::Constraint,
            CardType::Interface,
            CardType::Actor,
            CardType::LogicalComponent,
            CardType::DeployableNode,
            CardType::Artifact,
            CardType::Test,
            CardType::View,
            CardType::Note,
        ]
        .iter()
        .enumerate()
        {
            type_groups.insert(card_type.clone(), idx);
        }

        for card in model.cards.values() {
            let group = *type_groups.get(&card.r#type).unwrap_or(&0);
            node_map.insert(card.id.clone(), nodes.len());
            nodes.push(GraphNode {
                id: card.id.clone(),
                name: card.name.clone(),
                card_type: format!("{:?}", card.r#type),
                group,
            });
        }

        // Collect all links as graph edges
        let mut links = Vec::new();
        let mut unique_links = HashSet::new();

        for link in &model.links {
            if let Some(target_id) = &link.target_id {
                if node_map.contains_key(&link.source_id) && node_map.contains_key(target_id) {
                    let link_key = (link.source_id.clone(), target_id.clone());
                    if unique_links.insert(link_key.clone()) {
                        links.push(GraphLink {
                            source: link.source_id.clone(),
                            target: target_id.clone(),
                            distance: 30.0,
                        });
                    }
                }
            }
        }

        let node_count = nodes.len();
        let link_count = links.len();
        let max_possible_links = (node_count * (node_count - 1)) / 2;
        let density = if max_possible_links > 0 {
            (link_count as f64 / max_possible_links as f64) * 100.0
        } else {
            0.0
        };

        DependencyGraph {
            nodes,
            links,
            node_count,
            link_count,
            density,
        }
    }

    /// Get incoming links to a node
    pub fn incoming_links(&self, node_id: &str) -> Vec<&GraphLink> {
        self.links.iter().filter(|l| l.target == node_id).collect()
    }

    /// Get outgoing links from a node
    pub fn outgoing_links(&self, node_id: &str) -> Vec<&GraphLink> {
        self.links.iter().filter(|l| l.source == node_id).collect()
    }

    /// Get connected component (all nodes reachable from a given node)
    pub fn connected_component(&self, start_id: &str) -> HashSet<String> {
        let mut component = HashSet::new();
        let mut to_visit = vec![start_id.to_string()];

        while let Some(current) = to_visit.pop() {
            if component.contains(&current) {
                continue;
            }
            component.insert(current.clone());

            // Add outgoing neighbors
            for link in self.outgoing_links(&current) {
                if !component.contains(&link.target) {
                    to_visit.push(link.target.clone());
                }
            }

            // Add incoming neighbors
            for link in self.incoming_links(&current) {
                if !component.contains(&link.source) {
                    to_visit.push(link.source.clone());
                }
            }
        }

        component
    }

    /// Get isolated nodes (nodes with no connections)
    pub fn isolated_nodes(&self) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|node| {
                self.incoming_links(&node.id).is_empty() && self.outgoing_links(&node.id).is_empty()
            })
            .collect()
    }

    /// Get strongly connected components (if using directed graph semantics)
    pub fn root_nodes(&self) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|node| self.incoming_links(&node.id).is_empty())
            .collect()
    }

    /// Get leaf nodes (nodes with no outgoing links)
    pub fn leaf_nodes(&self) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|node| self.outgoing_links(&node.id).is_empty())
            .collect()
    }
}

/// Graph metrics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub node_count: usize,
    pub link_count: usize,
    pub density: f64,
    pub isolated_node_count: usize,
    pub root_node_count: usize,
    pub leaf_node_count: usize,
    pub avg_degree: f64,
    pub max_degree: usize,
}

impl GraphMetrics {
    pub fn calculate(graph: &DependencyGraph) -> Self {
        let node_count = graph.node_count;
        let link_count = graph.link_count;
        let isolated_node_count = graph.isolated_nodes().len();
        let root_node_count = graph.root_nodes().len();
        let leaf_node_count = graph.leaf_nodes().len();

        let mut degree_sum = 0;
        let mut max_degree = 0;

        for node in &graph.nodes {
            let degree =
                graph.incoming_links(&node.id).len() + graph.outgoing_links(&node.id).len();
            degree_sum += degree;
            max_degree = max_degree.max(degree);
        }

        let avg_degree = if node_count > 0 {
            degree_sum as f64 / node_count as f64
        } else {
            0.0
        };

        GraphMetrics {
            node_count,
            link_count,
            density: graph.density,
            isolated_node_count,
            root_node_count,
            leaf_node_count,
            avg_degree,
            max_degree,
        }
    }
}
