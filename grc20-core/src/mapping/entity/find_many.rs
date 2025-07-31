use futures::{Stream, StreamExt, TryStreamExt};
use uuid::Uuid;

use crate::{
    entity::{find_one::EntityQueryResult, utils::EntityFilter, Entity},
    error::DatabaseError,
    mapping::query_utils::{
        FieldOrderBy, MatchQuery, PropertyFilter, QueryBuilder, Subquery, ValueFilter,
    },
};

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    filter: EntityFilter,
    order_by: Option<FieldOrderBy>,
    limit: usize,
    skip: Option<usize>,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            filter: EntityFilter::default(),
            order_by: None,
            limit: 100,
            skip: None,
        }
    }

    pub fn id(mut self, id: impl Into<ValueFilter<Uuid>>) -> Self {
        self.filter.id = Some(id.into());
        self
    }

    pub fn property(mut self, property: PropertyFilter) -> Self {
        self.filter.properties.push(property);
        self
    }

    pub fn property_mut(&mut self, property: PropertyFilter) {
        self.filter.properties.push(property);
    }

    pub fn properties(mut self, properties: impl IntoIterator<Item = PropertyFilter>) -> Self {
        self.filter.properties.extend(properties);
        self
    }

    pub fn properties_mut(&mut self, properties: impl IntoIterator<Item = PropertyFilter>) {
        self.filter.properties.extend(properties);
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Overwrite the current filter with a new one
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn order_by(mut self, order_by: FieldOrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn order_by_mut(&mut self, order_by: FieldOrderBy) {
        self.order_by = Some(order_by);
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(MatchQuery::new("(e:Entity)"))
            .subquery(self.filter.subquery("e"))
            .subquery_opt(self.order_by.as_ref().map(|o| o.subquery("e")))
            .limit(self.limit)
            .skip_opt(self.skip)
            .with(vec!["DISTINCT e".to_string()],
                QueryBuilder::default()
                    .subqueries(vec![
                        "OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)",
                        "WHERE props IS NOT NULL",
                    ])
                    .r#return("e.id AS entity_id, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS properties, labels(e) AS types"))
    }

    pub async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity, DatabaseError>>, DatabaseError> {
        let query = self.subquery();

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity_node::FindManyQuery:\n{}\nparams:{:?}",
                query.compile(),
                query.params
            );
        };

        let stream = self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<EntityQueryResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| row_result.map(|row| row.into_entity()));

        Ok(stream)
    }
}
