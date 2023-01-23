mod utils;

use std::{cmp, collections::HashMap};

use bimap::BiMap;
use rs_graph::{
    Builder, VecGraph, EdgeVec,
    traits::{Directed, GraphIterator},
    vecgraph::{self, VecGraphBuilder},
    maxflow::dinic,
    mcf::{NetworkSimplex, MinCostFlow, SolutionState},
};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


type ID = u32;
type Graph = VecGraph<ID>;
type GraphNode = vecgraph::Node<ID>;
type GraphEdge = vecgraph::Edge<ID>;

const COST_MULTIPLIER: f64 = 1000.0;

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Path {
    flow: f64,
    nodes: Vec<String>,
}

#[wasm_bindgen]
impl Path {
    pub fn flow(&self) -> f64 { self.flow }
    pub fn nodes(&mut self) -> Vec<JsValue> { self.nodes.iter().map(|v| v.clone().into()).collect() }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct McmfSolution {
    max_flow: f64,
    total_cost: f64,
    paths: Vec<Path>,
}

#[wasm_bindgen]
impl McmfSolution {
    pub fn max_flow(&self) -> f64 { self.max_flow }
    pub fn total_cost(&self) -> f64 { self.total_cost }
    pub fn paths(&mut self) -> Vec<JsValue> { self.paths.iter().map(|v| v.clone().into()).collect() }
}

#[wasm_bindgen]
pub struct GraphBuilder {
    node_names: BiMap<String, GraphNode>,
    graph_builder: VecGraphBuilder<ID>,
    capacities: HashMap<GraphEdge, i64>,
    costs: HashMap<GraphEdge, f64>,
}

#[wasm_bindgen]
impl GraphBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        GraphBuilder {
            node_names: BiMap::new(),
            graph_builder: VecGraphBuilder::new(),
            capacities: HashMap::new(),
            costs: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: String, to: String, capacity: f64, cost: f64) {
        let capacity = capacity as i64;
        assert!(capacity > 0);
        let from = self.get_or_insert_vertex(from);
        let to = self.get_or_insert_vertex(to);
        let edge = self.graph_builder.add_edge(from, to);
        self.capacities.insert(edge, capacity);
        self.costs.insert(edge, cost);
    }

    pub fn solve_mcmf(self, source: String, sink: String) -> JsValue {
        return self.solve_mcmf_impl(source, sink).into()
    }

    fn solve_mcmf_impl(self, source: String, sink: String) -> McmfSolution {
        let source = self.get_vertex(source);
        let sink = self.get_vertex(sink);
        let graph = self.graph_builder.into_graph();
        let capacities = |v| self.capacities[&v];
        let costs = |v| (self.costs[&v] * COST_MULTIPLIER) as i64;
        let max_flow = dinic(&graph, source, sink, capacities).0 as i64;

        let mut spx = NetworkSimplex::new(&graph);
        spx.set_uppers(capacities);
        spx.set_costs(costs);
        spx.set_balance(source, max_flow);
        spx.set_balance(sink, -max_flow);
        assert_eq!(spx.solve(), SolutionState::Optimal);
        let paths = reconstruct_paths(&spx, &self.node_names, source, sink);
        McmfSolution {
            max_flow: max_flow as f64,
            total_cost: (spx.value() as f64) / COST_MULTIPLIER,
            paths,
        }
    }

    fn get_vertex(&self, v: String) -> GraphNode {
        *self.node_names.get_by_left(&v).unwrap()
    }
    fn get_or_insert_vertex(&mut self, v: String) -> GraphNode {
        if let Some(&id) = self.node_names.get_by_left(&v) {
            id
        } else {
            let id = self.graph_builder.add_node();
            self.node_names.insert_no_overwrite(v, id).unwrap();
            id
        }
    }
}

fn fill_paths<'g>(
    graph: &Graph,
    node_names: &BiMap<String, GraphNode>,
    to: GraphNode,
    path_flow: i64,
    path_prefix: &mut Vec<GraphNode>,
    path_prefix_edges: &mut Vec<GraphEdge>,
    remaining_flows: &mut EdgeVec<'g, &'g Graph, i64>,
    paths: &mut Vec<Path>
) {
    let from = *path_prefix.last().unwrap();
    for (e, v) in graph.out_iter(from).iter(&graph) {
        if remaining_flows[e] > 0 {
            let path_flow = cmp::min(path_flow, remaining_flows[e]);
            path_prefix.push(v);
            path_prefix_edges.push(e);
            if v == to {
                let path_nodes = path_prefix.iter().map(
                    |n| node_names.get_by_right(n).unwrap().clone()
                ).collect();
                paths.push(Path {
                    flow: path_flow as f64,
                    nodes: path_nodes,
                });
                for &e in path_prefix_edges.iter() {
                    remaining_flows[e] -= path_flow;
                }
            } else {
                fill_paths(
                    graph, node_names, to, path_flow,
                    path_prefix, path_prefix_edges, remaining_flows, paths
                );
            }
            path_prefix.pop();
            path_prefix_edges.pop();
        }
    }
}

fn reconstruct_paths(
    spx: &NetworkSimplex<Graph, i64>,
    node_names: &BiMap<String, GraphNode>,
    source: GraphNode,
    sink: GraphNode
) -> Vec<Path> {
    let graph = spx.as_graph();
    let mut path_prefix = vec![source];
    let mut path_prefix_edges = vec![];
    let mut paths = vec![];
    let mut remaining_flows = spx.flow_vec();
    let path_flow = i64::MAX;
    fill_paths(
        &graph, node_names, sink, path_flow,
        &mut path_prefix, &mut path_prefix_edges, &mut remaining_flows, &mut paths
    );
    assert_eq!(path_prefix.len(), 1);
    assert!(path_prefix_edges.is_empty());
    assert!(remaining_flows.iter().all(|(_, &flow)| flow == 0));
    paths
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_graph() {
        let mut builder = GraphBuilder::new();
        builder.add_edge("a".to_owned(), "b".to_owned(), 10., 200.);
        builder.add_edge("b".to_owned(), "c".to_owned(), 20., 0.);
        builder.add_edge("c".to_owned(), "e".to_owned(), 15., 0.);
        builder.add_edge("a".to_owned(), "d".to_owned(), 2., 100.);
        builder.add_edge("d".to_owned(), "e".to_owned(), 3., 0.);
        let solution = builder.solve_mcmf_impl("a".to_owned(), "e".to_owned());
        assert_eq!(solution.max_flow(), 12.0);
        assert_eq!(solution.total_cost(), 2200.0);
    }
}
