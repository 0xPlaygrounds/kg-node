use super::models::Entity;
use crate::entity::utils::{EntityFilter, RelationTraversal};
use crate::mapping::query_utils::Subquery;
use crate::mapping::value::models::Value;
use crate::{
    error::DatabaseError,
    mapping::{query_utils::QueryBuilder, EFFECTIVE_SEARCH_RATIO},
};
use futures::{Stream, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Query struct for performing semantic search on entities using vector embeddings
#[derive(Clone)]
pub struct SemanticSearchQuery {
    neo4j: neo4rs::Graph,
    search_vector: Vec<f64>,
    limit: usize,
    skip: Option<usize>,
    threshold: Option<f64>,
    filter: EntityFilter,
    traversals: Vec<RelationTraversal>,
}

impl SemanticSearchQuery {
    /// Create a new SemanticSearchQuery instance
    pub fn new(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            search_vector,
            limit: 100,
            skip: None,
            threshold: None,
            filter: EntityFilter::default(),
            traversals: Vec::new(),
        }
    }

    /// Set the limit for search results (builder pattern)
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the skip offset for search results (builder pattern)
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Set the threshold for minimum similarity score (builder pattern)
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Set a filter to apply to the search results (builder pattern)
    pub fn filter(mut self, filter: impl Into<EntityFilter>) -> Self {
        self.filter = filter.into();
        self
    }

    /// Add a single traversal relation to the search query
    pub fn traversal(mut self, relation: RelationTraversal) -> Self {
        self.traversals.push(relation);
        self
    }

    pub fn traversals(mut self, traversals: impl IntoIterator<Item = RelationTraversal>) -> Self {
        self.traversals.extend(traversals);
        self
    }

    pub fn subquery(&self) -> impl Subquery {
        let mut builder = QueryBuilder::default()
            .subqueries(vec![
                "CALL db.index.vector.queryNodes('vector_index', $limit * $effective_search_ratio, $vector)",
                "YIELD node, score",
                "WHERE score >= $threshold",
                "ORDER BY score DESC",
                "MATCH (e:Entity)-[r:PROPERTIES]->(node)",
                "WITH DISTINCT e, score",
            ])
            .subquery(self.filter.subquery("e"))
            .params("vector", self.search_vector.clone())
            .params("limit", self.limit as i64)
            .params("effective_search_ratio", EFFECTIVE_SEARCH_RATIO)
            .params("threshold", self.threshold.unwrap_or(0.0));

        builder = self
            .traversals
            .iter()
            .enumerate()
            .fold(builder, |query, (i, traversal)| {
                query
                    .subquery(format!("MATCH (e) -[r{i}:RELATION]- (dest)"))
                    .subquery_opt(traversal.relation_type_id.as_ref().map(|q| {
                        q.as_string_filter()
                            .subquery(&format!("r{i}"), "type", None)
                    }))
                    .subquery("WITH DISTINCT dest AS e, score")
            });

        builder
            .subqueries(vec![
                "OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)",
                "WHERE props IS NOT NULL",
                "WITH e, score, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS props",
            ])
            .skip_opt(self.skip)
            .limit(self.limit)
            .r#return("{entity_id: e.id, spaces: spaces, properties: props, score: score, types: labels(e)}")
    }

    /// Execute the semantic search query
    pub async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<SemanticSearchResult, DatabaseError>>, DatabaseError>
    {
        let query = self.subquery();

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity_node::FindManyQuery:\n{}\nparams:{:?}",
                query.compile(),
                query.params()
            );
        };

        Ok(self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<SemanticSearchResult>()
            .map_err(DatabaseError::from))
    }
}

/// Result struct representing a semantic search match with full entity data
#[derive(Clone, Debug, Deserialize)]
pub struct SemanticSearchResult {
    pub entity_id: Uuid,
    pub spaces: Vec<Uuid>,
    pub types: Vec<String>,
    pub properties: Vec<HashMap<Uuid, String>>,
    pub score: f64,
}

impl SemanticSearchResult {
    /// Convert the search result into a full Entity with score
    pub fn into_entity_with_score(self) -> (Entity, f64) {
        let entity = Entity {
            id: self.entity_id,
            types: self
                .types
                .into_iter()
                .filter_map(|id| Uuid::parse_str(&id).ok())
                .collect::<Vec<_>>(),
            values: self
                .spaces
                .into_iter()
                .zip(self.properties)
                .map(|(space_id, props)| {
                    (
                        space_id,
                        props
                            .into_iter()
                            .map(|(key, value)| {
                                (
                                    key,
                                    Value {
                                        property: key,
                                        value,
                                        options: None,
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
        };
        (entity, self.score)
    }

    /// Get just the entity without the score
    pub fn into_entity(self) -> Entity {
        self.into_entity_with_score().0
    }
}

/// Create a new SemanticSearchQuery instance
pub fn search(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> SemanticSearchQuery {
    SemanticSearchQuery::new(neo4j, search_vector)
}
