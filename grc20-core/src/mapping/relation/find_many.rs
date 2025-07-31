use crate::{
    error::DatabaseError,
    mapping::query_utils::{MatchQuery, ValueFilter, QueryBuilder, Subquery},
    relation::models::Relation,
};
use futures::{Stream, TryStreamExt};
use uuid::Uuid;

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

#[derive(Clone)]
pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    pub from: Option<ValueFilter<Uuid>>,
    pub to: Option<ValueFilter<Uuid>>,
    pub r#type: Option<Uuid>,
    pub limit: usize,
    pub skip: usize,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            from: None,
            to: None,
            r#type: None,
            limit: 100,
            skip: 0,
        }
    }

    pub fn from(mut self, filter: impl Into<ValueFilter<Uuid>>) -> Self {
        self.from = Some(filter.into());
        self
    }

    pub fn to(mut self, filter: impl Into<ValueFilter<Uuid>>) -> Self {
        self.to = Some(filter.into());
        self
    }

    pub fn r#type(mut self, relation_type: Uuid) -> Self {
        self.r#type = Some(relation_type);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    pub async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation, DatabaseError>>, DatabaseError> {
        let query = self.subquery().build();

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<Relation>()
            .map_err(DatabaseError::from))
    }

    fn subquery(&self) -> impl Subquery {
        let mut builder = QueryBuilder::default().subquery(
            MatchQuery::new("(from_entity:Entity)-[r:RELATION]->(to_entity:Entity)")
                .where_opt(
                    self.from
                        .clone()
                        .map(|s| s.as_string_filter().subquery("from_entity", "id", None)),
                )
                .where_opt(
                    self.to
                        .clone()
                        .map(|s| s.as_string_filter().subquery("to_entity", "id", None)),
                )
                .where_opt(self.r#type.clone().map(|t| {
                    ValueFilter::<Uuid>::from(t)
                        .as_string_filter()
                        .subquery("r", "type", None)
                })),
        );

        if let Some(relation_type) = self.r#type {
            builder = builder.params("relation_type", relation_type.to_string());
        }

        builder
            .skip(self.skip)
            .limit(self.limit)
            .r#return("r{.*, from_entity: from_entity.id, to_entity: to_entity.id} as r")
    }
}
