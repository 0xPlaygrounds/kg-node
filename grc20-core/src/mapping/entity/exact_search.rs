use crate::entity::utils::{EntityFilter, RelationTraversal};
use crate::entity::SemanticSearchResult;
use crate::mapping::query_utils::{Subquery, ValueFilter};
use crate::property::DataType;
use crate::{
    error::DatabaseError,
    mapping::{query_utils::QueryBuilder, EFFECTIVE_SEARCH_RATIO},
};
use futures::{Stream, TryStreamExt};

/// Query struct for performing semantic search on entities using vector embeddings
#[derive(Clone)]
pub struct ExactSemanticSearchQuery {
    neo4j: neo4rs::Graph,
    search_vector: Vec<f64>,
    limit: usize,
    skip: Option<usize>,
    threshold: Option<f64>,
    filter: EntityFilter,
    traversals: Vec<RelationTraversal>,
    data_type: Option<ValueFilter<DataType>>,
}

impl ExactSemanticSearchQuery {
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
            data_type: None,
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

    /// Add a traversal relation to the search query (builder pattern)
    pub fn traversal(mut self, traversal: impl Into<RelationTraversal>) -> Self {
        self.traversals.push(traversal.into());
        self
    }

    /// Add multiple traversal relations to the search query (builder pattern)
    pub fn traversals(mut self, traversals: impl IntoIterator<Item = RelationTraversal>) -> Self {
        self.traversals.extend(traversals);
        self
    }

    /// Set the data type filter for the search query (builder pattern)
    /// Note: This is only useful if searching for an entity of type `Property`
    pub fn data_type(mut self, data_type: impl Into<ValueFilter<DataType>>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    pub fn subquery(&self) -> impl Subquery {
        let mut builder = QueryBuilder::default()
            .subquery("MATCH (e:Entity)")
            .subquery(self.filter.subquery("e"))
            .subquery("WITH DISTINCT e")
            .subquery("MATCH (e)-[:PROPERTIES]->(props:Properties)")
            .subquery("WHERE props.embedding IS NOT NULL")
            .subquery("WITH e, vector.similarity.cosine(props.embedding, $vector) AS score")
            .subquery("WHERE score IS NOT NULL")
            .subquery("ORDER BY score DESC")
            .subquery("LIMIT $limit")
            .subquery("WITH DISTINCT e, score")
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

/// Create a new SemanticSearchQuery instance
pub fn exact_search(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> ExactSemanticSearchQuery {
    ExactSemanticSearchQuery::new(neo4j, search_vector)
}
