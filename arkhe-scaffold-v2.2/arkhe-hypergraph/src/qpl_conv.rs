//! QPL (Quantum Predicate Logic) simulated as a convolutional layer.
//!
//! HONESTY NOTE
//! ────────────
//! This is NOT a quantum simulation. There is no Hilbert space, no
//! superposition, no entanglement. The "QPL" name is retained from
//! the project's historical vocabulary, but the implementation is
//! purely classical: weighted neighborhood aggregation over the
//! hypergraph, structurally analogous to a graph convolution.
//!
//! WHAT IT ACTUALLY DOES
//! ─────────────────────
//! For each node, aggregate features from neighbors at EACH hop distance
//! separately, using per-hop kernel weights. This is equivalent to a single graph convolution layer with hop-aware weighting.
//!
//! v20.2 FIX: Neighbors are now grouped by actual hop distance using
//! BFS, eliminating the non-deterministic HashSet enumeration bug.

use crate::hypergraph::{Hypergraph, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Configuration for the QPL convolution layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QPLConfig {
    /// Number of hops for neighborhood aggregation.
    pub hops: usize,
    /// Kernel weights per hop (fixed — not trained).
    /// kernels[i] is applied to all neighbors at distance i+1.
    pub kernels: Vec<f64>,
    /// Whether to normalize by neighborhood size per hop.
    pub normalize: bool,
    /// Activation applied after aggregation.
    pub activation: Option<arkhe_core::calibration::ActivationRegime>,
}

impl Default for QPLConfig {
    fn default() -> Self {
        Self {
            hops: 2,
            kernels: vec![1.0, 0.5],
            normalize: true,
            activation: Some(arkhe_core::calibration::ActivationRegime::ReLU),
        }
    }
}

impl QPLConfig {
    pub fn single_hop() -> Self {
        Self {
            hops: 1,
            kernels: vec![1.0],
            ..Self::default()
        }
    }
}

/// Result of QPL convolution on a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QPLResult {
    pub node_id: NodeId,
    pub input_value: f64,
    pub output_value: f64,
    /// Total neighbors aggregated across all hops.
    pub neighbors_aggregated: usize,
    /// Neighbors per hop (deterministic — useful for debugging).
    pub neighbors_per_hop: Vec<usize>,
    pub hops_used: usize,
}

/// QPL Convolution Layer — classical neighborhood aggregation with hop-aware kernels.
pub struct QPLConvLayer {
    pub config: QPLConfig,
}

impl QPLConvLayer {
    pub fn new(config: QPLConfig) -> Self {
        Self { config }
    }
    pub fn default_layer() -> Self {
        Self::new(QPLConfig::default())
    }

    /// BFS from `node`, returning neighbors grouped by hop distance.
    /// `result[i]` = nodes at distance `i+1` from `node`.
    /// The node itself is NOT included.
    /// Order within each hop level is deterministic (sorted by NodeId).
    fn neighbors_by_hop<H, E>(&self, graph: &Hypergraph<H, E>, node: NodeId) -> Vec<Vec<NodeId>> {
        let mut result: Vec<Vec<NodeId>> = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        visited.insert(node);
        let mut frontier: VecDeque<NodeId> = VecDeque::new();
        frontier.push_back(node);

        for _ in 0..self.config.hops {
            let mut next_frontier: Vec<NodeId> = Vec::new();
            while let Some(n) = frontier.pop_front() {
                for neighbor in graph.neighbors(n) {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        next_frontier.push(neighbor);
                    }
                }
            }
            // Sort for deterministic ordering
            next_frontier.sort();
            result.push(next_frontier.clone());
            frontier = next_frontier.into_iter().collect();
        }

        result
    }

    /// Run convolution on a single node.
    pub fn forward_single<H, E>(
        &self,
        graph: &Hypergraph<H, E>,
        node_id: NodeId,
        values: &HashMap<NodeId, f64>,
    ) -> QPLResult {
        let input_val = values.get(&node_id).copied().unwrap_or(0.0);
        let hops = self.neighbors_by_hop(graph, node_id);

        let mut aggregated = 0.0;
        let mut total_count = 0;
        let mut counts_per_hop: Vec<usize> = Vec::new();

        for (hop_idx, hop_neighbors) in hops.iter().enumerate() {
            let kernel_weight = self.config.kernels.get(hop_idx).copied().unwrap_or(0.0);
            let mut hop_sum = 0.0;
            for &neighbor in hop_neighbors {
                let neighbor_val = values.get(&neighbor).copied().unwrap_or(0.0);
                hop_sum += kernel_weight * neighbor_val;
            }

            if self.config.normalize && !hop_neighbors.is_empty() {
                hop_sum /= hop_neighbors.len() as f64;
            }

            aggregated += hop_sum;
            total_count += hop_neighbors.len();
            counts_per_hop.push(hop_neighbors.len());
        }

        let output = if let Some(ref act) = self.config.activation {
            arkhe_core::calibration::activate(aggregated, act)
        } else {
            aggregated
        };

        QPLResult {
            node_id,
            input_value: input_val,
            output_value: output,
            neighbors_aggregated: total_count,
            neighbors_per_hop: counts_per_hop,
            hops_used: self.config.hops,
        }
    }

    /// Run convolution on all nodes with given values.
    pub fn forward<H, E>(
        &self,
        graph: &Hypergraph<H, E>,
        values: &HashMap<NodeId, f64>,
    ) -> Vec<QPLResult> {
        let mut results = Vec::new();
        // Sort node IDs for deterministic output order
        let mut node_ids: Vec<NodeId> = values.keys().copied().collect();
        node_ids.sort();

        for node_id in node_ids {
            results.push(self.forward_single(graph, node_id, values));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// Build a chain: 0 — 1 — 2 — 3 — 4
    fn make_chain(n: usize) -> Hypergraph<&'static str, &'static str> {
        let mut g = Hypergraph::new();
        let mut prev = None;
        for _ in 0..n {
            let node = g.add_node("node");
            if let Some(p) = prev {
                g.add_edge(HashSet::from([p]), HashSet::from([node]), 1.0, None);
            }
            prev = Some(node);
        }
        g
    }

    #[test]
    fn test_neighbors_by_hop_1hop_chain() {
        let g = make_chain(5);
        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let hops = layer.neighbors_by_hop(&g, 2);
        // 1-hop from node 2: {1, 3}
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0], vec![1, 3]); // sorted, deterministic
    }

    #[test]
    fn test_neighbors_by_hop_2hop_chain() {
        let g = make_chain(5);
        let layer = QPLConvLayer::new(QPLConfig::default());
        let hops = layer.neighbors_by_hop(&g, 2);
        assert_eq!(hops.len(), 2);
        // 1-hop from 2: {1, 3}
        assert_eq!(hops[0], vec![1, 3]);
        // 2-hop from 2: {0, 4}
        assert_eq!(hops[1], vec![0, 4]);
    }

    #[test]
    fn test_neighbors_by_hop_boundary() {
        let g = make_chain(5);
        let layer = QPLConvLayer::new(QPLConfig::default());
        // Node 0: 1-hop={1}, 2-hop={2}
        let hops = layer.neighbors_by_hop(&g, 0);
        assert_eq!(hops[0], vec![1]);
        assert_eq!(hops[1], vec![2]);
    }

    #[test]
    fn test_neighbors_by_hop_isolated() {
        let mut g: Hypergraph<&str, &str> = Hypergraph::new();
        let isolated = g.add_node("iso");
        let layer = QPLConvLayer::new(QPLConfig::default());
        let hops = layer.neighbors_by_hop(&g, isolated);
        assert_eq!(hops.len(), 2);
        assert!(hops[0].is_empty());
        assert!(hops[1].is_empty());
    }

    #[test]
    fn test_qpl_single_node_chain() {
        let g = make_chain(5);
        let values: HashMap<NodeId, f64> =
            [(0, 0.0), (1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0)].into();
        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let r = layer.forward_single(&g, 2, &values);
        // 1-hop: {1, 3}, kernel=[1.0], normalize=true
        // = (1.0*1 + 1.0*3) / 2 = 2.0, ReLU(2.0) = 2.0
        assert!((r.output_value - 2.0).abs() < 1e-9);
        assert_eq!(r.neighbors_aggregated, 2);
        assert_eq!(r.neighbors_per_hop, vec![2]);
    }

    #[test]
    fn test_qpl_boundary_node() {
        let g = make_chain(5);
        let values: HashMap<NodeId, f64> =
            [(0, 0.0), (1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0)].into();
        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let r = layer.forward_single(&g, 0, &values);
        // 1-hop: {1}, kernel=[1.0], normalize=true
        // = 1.0 / 1 = 1.0, ReLU(1.0) = 1.0
        assert!((r.output_value - 1.0).abs() < 1e-9);
        assert_eq!(r.neighbors_per_hop, vec![1]);
    }

    #[test]
    fn test_qpl_2hop_chain() {
        // THIS IS THE CRITICAL TEST THAT WAS FLAKY IN v20.0.
        // Now it's deterministic because neighbors_by_hop uses sorted Vec.
        let g = make_chain(5);
        let values: HashMap<NodeId, f64> =
            [(0, 0.0), (1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0)].into();
        let layer = QPLConvLayer::new(QPLConfig::default());
        let r = layer.forward_single(&g, 2, &values);
        // 1-hop: {1, 3}, kernel[0]=1.0, normalized: (1+3)/2 = 2.0
        // 2-hop: {0, 4}, kernel[1]=0.5, normalized: (0.5*0 + 0.5*4)/2 = 1.0
        // total = 2.0 + 1.0 = 3.0, ReLU(3.0) = 3.0
        assert!((r.output_value - 3.0).abs() < 1e-9);
        assert_eq!(r.neighbors_per_hop, vec![2, 2]);
        assert_eq!(r.neighbors_aggregated, 4);
    }

    #[test]
    fn test_qpl_no_activation() {
        let g = make_chain(3);
        let values: HashMap<NodeId, f64> = [(0, -1.0), (1, 2.0), (2, -3.0)].into();
        let config = QPLConfig {
            activation: None,
            ..QPLConfig::single_hop()
        };
        let layer = QPLConvLayer::new(config);
        let r = layer.forward_single(&g, 1, &values);
        // 1-hop: {0, 2}, kernel=[1.0], normalized: (-1 + -3) / 2 = -2.0
        assert!((r.output_value - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_qpl_no_neighbors() {
        let mut g: Hypergraph<&str, &str> = Hypergraph::new();
        let isolated = g.add_node("isolated");
        let values: HashMap<NodeId, f64> = [(isolated, 5.0)].into();
        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let r = layer.forward_single(&g, isolated, &values);
        assert_eq!(r.neighbors_aggregated, 0);
        assert_eq!(r.neighbors_per_hop, vec![0]);
        // No neighbors → aggregated = 0.0 (0 hops contribute)
        assert!(r.output_value.abs() < 1e-9);
    }

    #[test]
    fn test_qpl_forward_all_deterministic_order() {
        let g = make_chain(3);
        let values: HashMap<NodeId, f64> = [(0, 1.0), (1, 2.0), (2, 3.0)].into();
        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let results = layer.forward(&g, &values);
        assert_eq!(results.len(), 3);
        // Results should be in sorted NodeId order
        assert_eq!(results[0].node_id, 0);
        assert_eq!(results[1].node_id, 1);
        assert_eq!(results[2].node_id, 2);
    }

    #[test]
    fn test_qpl_star_graph() {
        // Center node 0 connected to 1, 2, 3, 4
        let mut g: Hypergraph<&str, &str> = Hypergraph::new();
        let center = g.add_node("center");
        let mut leaves = Vec::new();
        for _ in 0..4 {
            let leaf = g.add_node("leaf");
            g.add_edge(HashSet::from([center]), HashSet::from([leaf]), 1.0, None);
            leaves.push(leaf);
        }
        let mut values = HashMap::new();
        values.insert(center, 0.0);
        for (i, &leaf) in leaves.iter().enumerate() {
            values.insert(leaf, (i + 1) as f64);
        }

        let layer = QPLConvLayer::new(QPLConfig::single_hop());
        let r = layer.forward_single(&g, center, &values);
        // 1-hop: {1,2,3,4}, kernel=[1.0], normalized: (1+2+3+4)/4 = 2.5
        assert!((r.output_value - 2.5).abs() < 1e-9);
        assert_eq!(r.neighbors_per_hop, vec![4]);

        // 2-hop: all leaves' neighbors = just center, but center is visited
        let layer2 = QPLConvLayer::new(QPLConfig::default());
        let r2 = layer2.forward_single(&g, center, &values);
        // 1-hop contributes 2.5, 2-hop is empty → total = 2.5
        assert!((r2.output_value - 2.5).abs() < 1e-9);
        assert_eq!(r2.neighbors_per_hop, vec![4, 0]);
    }
}
