#![allow(unused_imports)]
use std::collections::HashSet;

pub type NodeId = usize;

pub struct Hypergraph<H, E> {
    nodes: HashSet<NodeId>,
    edges: Vec<(HashSet<NodeId>, HashSet<NodeId>, f64, Option<E>)>,
    next_id: NodeId,
    _h: std::marker::PhantomData<H>,
}

impl<H, E> Default for Hypergraph<H, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H, E> Hypergraph<H, E> {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
            next_id: 0,
            _h: std::marker::PhantomData,
        }
    }

    pub fn add_node(&mut self, _data: H) -> NodeId {
        let id = self.next_id;
        self.nodes.insert(id);
        self.next_id += 1;
        id
    }

    pub fn add_edge(
        &mut self,
        sources: HashSet<NodeId>,
        targets: HashSet<NodeId>,
        weight: f64,
        data: Option<E>,
    ) {
        self.edges.push((sources, targets, weight, data));
    }

    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        let mut n = HashSet::new();
        for (src, tgt, _, _) in &self.edges {
            if src.contains(&node) {
                n.extend(tgt.iter().copied());
            }
            if tgt.contains(&node) {
                n.extend(src.iter().copied());
            }
        }
        n.remove(&node);
        n.into_iter().collect()
    }
}
