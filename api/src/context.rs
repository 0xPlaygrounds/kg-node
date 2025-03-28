use grc20_core::neo4rs;
use std::sync::Arc;

#[derive(Clone)]
pub struct KnowledgeGraph(pub Arc<neo4rs::Graph>);

impl juniper::Context for KnowledgeGraph {}

impl KnowledgeGraph {
    pub fn new(graph: Arc<neo4rs::Graph>) -> Self {
        Self(graph)
    }
}
