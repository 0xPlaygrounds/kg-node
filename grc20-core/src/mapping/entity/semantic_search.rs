use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    entity::utils::MatchEntity,
    error::DatabaseError,
    mapping::{
        query_utils::VersionFilter, AttributeNode, FromAttributes, PropFilter, QueryBuilder,
        QueryStream, Subquery, EFFECTIVE_SEARCH_RATIO,
    },
};

use super::{Entity, EntityFilter, EntityNode};

pub struct SemanticSearchQuery<T> {
    neo4j: neo4rs::Graph,
    vector: Vec<f64>,
    filter: EntityFilter,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
    limit: usize,
    skip: Option<usize>,

    _marker: std::marker::PhantomData<T>,
}

impl<T> SemanticSearchQuery<T> {
    pub fn new(neo4j: &neo4rs::Graph, vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            vector,
            filter: EntityFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
        self.space_id = Some(filter);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn limit_opt(mut self, limit: Option<usize>) -> Self {
        if let Some(limit) = limit {
            self.limit = limit;
        }
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn skip_opt(mut self, skip: Option<usize>) -> Self {
        self.skip = skip;
        self
    }

    fn subquery(&self) -> QueryBuilder {
        const QUERY: &str = r#"
            CALL db.index.vector.queryNodes('vector_index', $effective_search_ratio, $vector)
            YIELD node AS n, score AS score
            MATCH (e:Entity) -[r:ATTRIBUTE]-> (n)
        "#;

        // Exact neighbor search using vector index (very expensive but allows prefiltering)
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH (e:Entity) -[r:ATTRIBUTE]-> (a:Attribute:Indexed)
        //     WHERE r.max_version IS null
        //     AND a.embedding IS NOT NULL
        //     WITH e, a, r, vector.similarity.cosine(a.embedding, $vector) AS score
        //     ORDER BY score DESC
        //     WHERE score IS NOT null
        //     LIMIT $limit
        //     RETURN a{{.*, entity: e.id, space_version: r.min_version, space_id: r.space_id, score: score}}
        //     "#,
        // );

        QueryBuilder::default()
            .subquery(QUERY)
            .subquery(self.filter.subquery("e"))
            .limit(self.limit)
            .skip_opt(self.skip)
            .params("vector", self.vector.clone())
            .params("effective_search_ratio", EFFECTIVE_SEARCH_RATIO)
            .params("limit", self.limit as i64)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchResult<T> {
    pub entity: T,
    pub score: f64,
}
impl QueryStream<SemanticSearchResult<EntityNode>> for SemanticSearchQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<
        impl Stream<Item = Result<SemanticSearchResult<EntityNode>, DatabaseError>>,
        DatabaseError,
    > {
        let query = self.subquery().r#return("DISTINCT e, score");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::FindManyQuery::<EntityNode>:\n{}\nparams:{:?}",
                query.compile(),
                query.params()
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            e: EntityNode,
            score: f64,
        }

        Ok(self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                Ok(SemanticSearchResult {
                    entity: row.e,
                    score: row.score,
                })
            }))
    }
}

impl<T: FromAttributes> QueryStream<SemanticSearchResult<Entity<T>>>
    for SemanticSearchQuery<Entity<T>>
{
    async fn send(
        self,
    ) -> Result<
        impl Stream<Item = Result<SemanticSearchResult<Entity<T>>, DatabaseError>>,
        DatabaseError,
    > {
        let match_entity = MatchEntity::new(&self.space_id, &self.version);

        let query = self.subquery().with(
            vec!["e".to_string(), "score".to_string()],
            match_entity.chain(
                "e",
                "attrs",
                "types",
                Some(vec!["score".to_string()]),
                "RETURN e{.*, attrs: attrs, types: types, score: score}",
            ),
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::FindManyQuery::<Entity<T>>:\n{}\nparams:{:?}",
                query.compile(),
                query.params
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: EntityNode,
            attrs: Vec<AttributeNode>,
            types: Vec<EntityNode>,
            score: f64,
        }

        let stream = self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| {
                row_result.and_then(|row| {
                    T::from_attributes(row.attrs.into())
                        .map(|data| SemanticSearchResult {
                            entity: Entity {
                                node: row.node,
                                attributes: data,
                                types: row.types.into_iter().map(|t| t.id).collect(),
                            },
                            score: row.score,
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}
