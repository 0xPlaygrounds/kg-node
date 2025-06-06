use neo4rs::Path;

use crate::{
    entity::EntityFilter,
    error::DatabaseError,
    mapping::{
        order_by::FieldOrderBy,
        query_utils::{
            query_builder::{MatchQuery, QueryBuilder, Subquery},
            VersionFilter,
        },
        AttributeFilter, FromAttributes, PropFilter, Query,
    },
};

pub struct Relation {
    pub nodes_ids: Vec<String>,
    pub relations_ids: Vec<String>,
}

pub struct FindPathQuery<T> {
    neo4j: neo4rs::Graph,
    id1: String,
    id2: String,
    filter: EntityFilter,
    order_by: Option<FieldOrderBy>,
    limit: usize,
    skip: Option<usize>,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindPathQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph, id1: String, id2: String) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id1,
            id2,
            filter: EntityFilter::default(),
            order_by: None,
            limit: 100,
            skip: None,
            space_id: None,
            version: VersionFilter::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.filter.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.filter.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.filter.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.filter.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.filter.attributes.extend(attributes);
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

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, space_version: String) -> Self {
        self.version.version_mut(space_version);
        self
    }

    pub fn version_opt(mut self, space_version: Option<String>) -> Self {
        self.version.version_opt(space_version);
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(MatchQuery::new(
                "p = allShortestPaths((e1:Entity {id: $id1}) -[:RELATION*1..5]-(e2:Entity {id: $id2}))",
            ))
            .limit(self.limit)
            .params("id1", self.id1.clone())
            .params("id2", self.id2.clone())
    }
}

impl<T: FromAttributes> Query<Vec<Relation>> for FindPathQuery<T> {
    async fn send(self) -> Result<Vec<Relation>, DatabaseError> {
        let query = self.subquery().r#return("p");

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity::FindPathQuery::<T>:\n{}\nparams:{:?}",
                query.compile(),
                [self.id1, self.id2]
            );
        }

        let mut result = self.neo4j.execute(query.build()).await?;
        let mut all_relationship_data = Vec::new();

        // Process each row
        while let Some(row) = result.next().await? {
            let path: Path = row.get("p")?;
            tracing::info!("This is the info for Path: {:?}", path);

            let relationship_data: Relation = Relation {
                nodes_ids: (path
                    .nodes()
                    .iter()
                    .filter_map(|rel| rel.get("id").ok())
                    .collect()),
                relations_ids: (path
                    .rels()
                    .iter()
                    .filter_map(|rel| rel.get("relation_type").ok())
                    .collect()),
            };

            all_relationship_data.push(relationship_data);
        }

        Ok(all_relationship_data)
    }
}
