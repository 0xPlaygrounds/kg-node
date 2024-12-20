use std::collections::HashMap;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    error::DatabaseError, models::BlockMetadata, neo4j_utils::serde_value_to_bolt, system_ids,
};

use super::{
    attributes::{Attributes, SystemProperties},
    query::Query,
    Entity, Triples,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Relation<T> {
    pub id: String,
    pub types: Vec<String>,
    pub from: String,
    pub to: String,
    #[serde(flatten)]
    pub attributes: Attributes<T>,
}

impl<T> Relation<T> {
    pub fn new(
        id: &str,
        space_id: &str,
        from: &str,
        to: &str,
        block: &BlockMetadata,
        data: T,
    ) -> Self {
        Self {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            types: vec![system_ids::RELATION_TYPE.to_string()],
            attributes: Attributes {
                id: id.to_string(),
                system_properties: SystemProperties {
                    space_id: space_id.to_string(),
                    created_at: block.timestamp,
                    created_at_block: block.block_number.to_string(),
                    updated_at: block.timestamp,
                    updated_at_block: block.block_number.to_string(),
                },
                attributes: data,
            },
        }
    }

    pub fn from_entity(entity: Entity<T>, from: &str, to: &str) -> Self {
        Self {
            id: entity.id().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            types: entity.types,
            attributes: entity.attributes,
        }
    }

    pub fn id(&self) -> &str {
        &self.attributes.id
    }

    pub fn space_id(&self) -> &str {
        &self.attributes.system_properties.space_id
    }

    pub fn attributes(&self) -> &T {
        &self.attributes.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut T {
        &mut self.attributes.attributes
    }

    pub fn with_type(mut self, type_id: &str) -> Self {
        self.types.push(type_id.to_string());
        self
    }

    /// Returns a query to delete the current relation
    pub fn delete_query(id: &str) -> Query<()> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (r {{id: $id}})
            DETACH DELETE r
            "#,
        );

        Query::new(QUERY).param("id", id)
    }

    pub async fn types(
        &self,
        neo4j: &neo4rs::Graph,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        Self::find_types(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_types(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (r {{id: $id, space_id: $space_id}})
            UNWIND labels(r) as l
            MATCH (t {{id: l, space_id: $space_id}})
            RETURN t
            "#,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            t: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.t.try_into()?) })
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn to<E>(&self, neo4j: &neo4rs::Graph) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        Self::find_to(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_to<E>(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH ({{id: $id, space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (to)
            RETURN to
            "#,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            to: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                row.to.try_into()
            })
            .transpose()?)
    }

    pub async fn from<E>(&self, neo4j: &neo4rs::Graph) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        Self::find_from(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_from<E>(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH ({{id: $id, space_id: $space_id}}) -[:`{FROM_ENTITY}`]-> (from)
            RETURN from
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                row.from.try_into()
            })
            .transpose()?)
    }
}

impl<T> Relation<T>
where
    T: Serialize,
{
    /// Upsert the current relation
    pub async fn upsert(&self, neo4j: &neo4rs::Graph) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from {{id: $from_id}})
            MATCH (to {{id: $to_id}})
            MERGE (from)<-[:`{FROM_ENTITY}`]-(r {{id: $id, space_id: $space_id}})-[:`{TO_ENTITY}`]->(to)
            ON CREATE SET r += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET r:$($labels)
            SET r += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET r += $data
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let bolt_data = match serde_value_to_bolt(serde_json::to_value(self.attributes())?) {
            neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
            _ => neo4rs::BoltType::Map(Default::default()),
        };

        let query = neo4rs::query(QUERY)
            .param("id", self.id())
            .param("space_id", self.space_id())
            .param("from_id", self.from.clone())
            .param("to_id", self.to.clone())
            .param("space_id", self.space_id())
            .param(
                "created_at",
                self.attributes.system_properties.created_at.to_rfc3339(),
            )
            .param(
                "created_at_block",
                self.attributes
                    .system_properties
                    .created_at_block
                    .to_string(),
            )
            .param(
                "updated_at",
                self.attributes.system_properties.updated_at.to_rfc3339(),
            )
            .param(
                "updated_at_block",
                self.attributes
                    .system_properties
                    .updated_at_block
                    .to_string(),
            )
            .param("labels", self.types.clone())
            .param("data", bolt_data);

        Ok(neo4j.run(query).await?)
    }
}

impl<T> Relation<T>
where
    T: for<'a> Deserialize<'a>,
{
    /// Returns the entity with the given ID, if it exists
    pub async fn find_by_id(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r:`{RELATION_TYPE}` {{ id: $id, space_id: $space_id }}) -[:`{TO_ENTITY}`]-> (to)
            RETURN from, r, to
            "#,
            RELATION_TYPE = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;

                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id()))
            })
            .transpose()
    }

    /// Returns the entities from the given list of IDs
    pub async fn find_by_ids(
        neo4j: &neo4rs::Graph,
        ids: &[String],
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $ids AS id
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r:`{RELATION_TYPE}` {{ id: $id, space_id: $space_id }}) -[:`{TO_ENTITY}`]-> (to)
            RETURN from, r, to
            "#,
            RELATION_TYPE = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY).param("ids", ids);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id()))
            })
            .try_collect::<Vec<_>>()
            .await
    }

    /// Returns the entities with the given types
    pub async fn find_by_types(
        neo4j: &neo4rs::Graph,
        types: &[String],
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from {{space_id: $space_id}}) <-[:`{FROM_ENTITY}`]- (r:`{RELATION_TYPE}`:$($types) {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (to {{space_id: $space_id}})
            RETURN from, r, to
            "#,
            RELATION_TYPE = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("types", types)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id()))
            })
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn find_all(
        neo4j: &neo4rs::Graph,
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from {{space_id: $space_id}}) <-[:`{FROM_ENTITY}`]- (r:`{RELATION_TYPE}` {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (to {{space_id: $space_id}})
            RETURN from, r, to
            LIMIT 100
            "#,
            RELATION_TYPE = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY).param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id()))
            })
            .try_collect::<Vec<_>>()
            .await
    }
}

impl Relation<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, key: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut().insert(key, value.into());
        self
    }
}
