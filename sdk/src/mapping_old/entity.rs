use futures::stream::{self, StreamExt, TryStreamExt};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::DatabaseError,
    // graph_uri::{self, GraphUri},
    ids::create_id_from_unique_string,
    indexer_ids,
    mapping::{self, query_utils::query_part::IntoQueryPart},
    models::BlockMetadata,
    neo4j_utils::serde_value_to_bolt,
    pb,
    system_ids,
};

use super::{
    attributes::{self, SystemProperties}, entity_queries, relation_queries, triple::{FromTriples, IntoTriples, TryFromTriples, TryIntoTriples}, Relation, Triples
};

/// GRC20 Node
#[derive(Debug, PartialEq)]
pub struct Entity<T = ()> {
    // pub space_id: String,
    // pub space_version_id: String,
    // pub version: i64,

    pub types: Vec<String>,
    pub attributes: T,
    pub system_properties: SystemProperties,
}

impl<T> Entity<T> {
    /// Creates a new entity with the given ID, space ID, and data
    pub fn new(
        id: &str,
        space_id: &str,
        // space_version_index: i64,
        block: &BlockMetadata,
        data: T,
    ) -> Self {
        Self {
            types: Vec::new(),
            // version: 0,
            system_properties: SystemProperties {
                id: id.to_string(),
                // space_id: space_id.to_string(),
                created_at: block.timestamp,
                created_at_block: block.block_number.to_string(),
                updated_at: block.timestamp,
                updated_at_block: block.block_number.to_string(),
            },
            attributes: data,
        }
    }

    /// Returns the ID of the entity
    pub fn id(&self) -> &str {
        &self.system_properties.id
    }

    /// Returns the space ID of the entity
    pub fn space_id(&self) -> &str {
        // &self.system_properties.space_id
        ""
    }

    /// Returns the attributes of the entity
    pub fn attributes(&self) -> &T {
        &self.attributes
    }

    /// Returns a mutable reference to the attributes of the entity
    pub fn attributes_mut(&mut self) -> &mut T {
        &mut self.attributes
    }

    /// Adds a type label to the entity
    pub fn with_type(mut self, type_id: &str) -> Self {
        self.types.push(type_id.to_string());
        self
    }

    /// Returns the outgoing relations of the entity
    pub async fn relations<R>(
        &self,
        neo4j: &neo4rs::Graph,
        filter: Option<relation_queries::FindMany>,
    ) -> Result<Vec<Relation<R>>, DatabaseError>
    where
        R: TryFromTriples,
    {
        Relation::<R>::find_many(
            neo4j,
            Some(filter.unwrap_or(
                relation_queries::FindMany::new("r").from(|from_query| from_query.id(self.id())),
            )),
        )
        .await
    }

    // pub async fn find_relations<R>(
    //     neo4j: &neo4rs::Graph,
    //     filter: Option<relation_queries::FindMany>,
    // ) -> Result<Vec<Relation<R>>, DatabaseError>
    // where
    //     R: for<'a> Deserialize<'a>,
    // {
    //     const QUERY: &str = const_format::formatcp!(
    //         r#"
    //         MATCH ({{ id: $id }}) <-[:`{FROM_ENTITY}`]- (r) -[:`{TO_ENTITY}`]-> (to)
    //         MATCH (r) -[:`{RELATION_TYPE}`]-> (rt)
    //         RETURN to, r, rt
    //         "#,
    //         FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
    //         TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
    //         RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE
    //     );

    //     let query = if let Some(filter) = filter {
    //         filter.into_query_part().build()
    //     } else {
    //         neo4rs::query(QUERY).param("id", id)
    //     };

    //     #[derive(Debug, Deserialize)]
    //     struct RowResult {
    //         r: neo4rs::Node,
    //         to: neo4rs::Node,
    //         rt: neo4rs::Node,
    //     }

    //     neo4j
    //         .execute(query)
    //         .await?
    //         .into_stream_as::<RowResult>()
    //         .map_err(DatabaseError::from)
    //         .and_then(|row| async move {
    //             let rel: Entity<R> = row.r.try_into()?;
    //             let to: Entity<()> = row.to.try_into()?;
    //             let rel_type: Entity<()> = row.rt.try_into()?;

    //             Ok(Relation::from_entity(rel, id, to.id(), rel_type.id()))
    //         })
    //         .try_collect::<Vec<_>>()
    //         .await
    // }

    // pub async fn types(
    //     &self,
    //     neo4j: &neo4rs::Graph,
    // ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
    //     Self::find_types(neo4j, self.id(), self.space_id()).await
    // }

    pub async fn find_types(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH ({{ id: $id, space_id: $space_id }}) <-[:`{FROM_ENTITY}`]- (r {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (t {{space_id: $space_id}})
        //     MATCH (r) -[:`{RELATION_TYPE}`]-> ({{id: "{TYPES}"}})
        //     RETURN t
        //     "#,
        //     FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
        //     TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        //     RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        //     TYPES = system_ids::TYPES_ATTRIBUTE,
        // );

        // let query = neo4rs::query(QUERY)
        //     .param("id", id)
        //     .param("space_id", space_id);

        // #[derive(Debug, Deserialize)]
        // struct RowResult {
        //     t: neo4rs::Node,
        // }

        // neo4j
        //     .execute(query)
        //     .await?
        //     .into_stream_as::<RowResult>()
        //     .map_err(DatabaseError::from)
        //     .and_then(|row| async move { Ok(row.t.try_into()?) })
        //     .try_collect::<Vec<_>>()
        //     .await
        todo!()
    }

    pub async fn blocks(
        &self,
        neo4j: &neo4rs::Graph,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        Self::find_blocks(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_blocks(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH ({{ id: $id, space_id: $space_id }}) <-[:`{FROM_ENTITY}`]- (r {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (block {{space_id: $space_id}})
        //     MATCH (r) -[:`{RELATION_TYPE}`]-> ({{id: "{BLOCKS}"}})
        //     RETURN block
        //     "#,
        //     FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
        //     TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        //     RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        //     BLOCKS = system_ids::BLOCKS,
        // );

        // let query = neo4rs::query(QUERY)
        //     .param("id", id)
        //     .param("space_id", space_id);

        // #[derive(Debug, Deserialize)]
        // struct RowResult {
        //     block: neo4rs::Node,
        // }

        // neo4j
        //     .execute(query)
        //     .await?
        //     .into_stream_as::<RowResult>()
        //     .map_err(DatabaseError::from)
        //     .and_then(|row| async move { Ok(row.block.try_into()?) })
        //     .try_collect::<Vec<_>>()
        //     .await
        todo!()
    }

    pub async fn set_triples(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        entity_id: &str,
        version: i64,
        triples: Triples,
    ) -> Result<(), DatabaseError> {
        let entity = Entity::<Triples>::new(
            entity_id,
            space_id,
            // 0,
            block,
            triples,
        );

        entity.upsert(neo4j, version).await
    }

    pub async fn set_triple(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        entity_id: &str,
        attribute_id: &str,
        value: &pb::ipfs::Value,
    ) -> Result<(), DatabaseError> {
        let entity = Entity::<mapping::Triples>::new(
            entity_id,
            space_id,
            // 0,
            block,
            mapping::Triples(HashMap::from([(
                attribute_id.to_string(),
                mapping::Triple {
                    attribute: attribute_id.to_string(),
                    value: value.value.clone(),
                    value_type: mapping::ValueType::try_from(value.r#type())
                        .unwrap_or(mapping::ValueType::Text),
                    options: Default::default(),
                },
            )])),
        );

        entity.upsert(neo4j, 0).await
        // todo!()

        // match (attribute_id, value.r#type(), value.value.as_str()) {
        // Set the type of the entity
        // (system_ids::TYPES, pb::ipfs::ValueType::Url, value) => {
        //     const SET_TYPE_QUERY: &str = const_format::formatcp!(
        //         r#"
        //         MERGE (n {{ id: $id, space_id: $space_id }})
        //         ON CREATE SET n += {{
        //             `{CREATED_AT}`: datetime($created_at),
        //             `{CREATED_AT_BLOCK}`: $created_at_block
        //         }}
        //         SET n += {{
        //             `{UPDATED_AT}`: datetime($updated_at),
        //             `{UPDATED_AT_BLOCK}`: $updated_at_block
        //         }}
        //         SET n:$($labels)
        //         "#,
        //         CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
        //         CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
        //         UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
        //         UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        //     );

        //     let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

        //     let query = neo4rs::query(SET_TYPE_QUERY)
        //         .param("id", entity_id)
        //         .param("space_id", space_id)
        //         .param("created_at", block.timestamp.to_rfc3339())
        //         .param("created_at_block", block.block_number.to_string())
        //         .param("updated_at", block.timestamp.to_rfc3339())
        //         .param("updated_at_block", block.block_number.to_string())
        //         .param("labels", uri.id);

        //     Ok(neo4j.run(query).await?)
        // }

        // Set the FROM_ENTITY, TO_ENTITY or RELATION_TYPE on a relation entity
        // (
        //     system_ids::RELATION_FROM_ATTRIBUTE | system_ids::RELATION_TO_ATTRIBUTE | system_ids::RELATION_TYPE_ATTRIBUTE,
        //     pb::ipfs::ValueType::Url,
        //     value,
        // ) => {
        //     let query = format!(
        //         r#"
        //         MATCH (n {{ id: $other, space_id: $space_id }})
        //         MERGE (r {{ id: $id, space_id: $space_id }})
        //         MERGE (r) -[:`{attribute_id}`]-> (n)
        //         ON CREATE SET r += {{
        //             `{CREATED_AT}`: datetime($created_at),
        //             `{CREATED_AT_BLOCK}`: $created_at_block
        //         }}
        //         SET r += {{
        //             `{UPDATED_AT}`: datetime($updated_at),
        //             `{UPDATED_AT_BLOCK}`: $updated_at_block
        //         }}
        //         "#,
        //         attribute_id = attribute_id,
        //         CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
        //         CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
        //         UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
        //         UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        //     );

        //     let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

        //     let query = neo4rs::query(&query)
        //         .param("id", entity_id)
        //         .param("other", uri.id)
        //         .param("space_id", space_id)
        //         .param("created_at", block.timestamp.to_rfc3339())
        //         .param("created_at_block", block.block_number.to_string())
        //         .param("updated_at", block.timestamp.to_rfc3339())
        //         .param("updated_at_block", block.block_number.to_string());

        //     Ok(neo4j.run(query).await?)
        // }

        // Set the RELATION_TYPE on a relation entity
        // (system_ids::RELATION_TYPE_ATTRIBUTE, pb::ipfs::ValueType::Url, value) => {
        //     const QUERY: &str = const_format::formatcp!(
        //         r#"
        //         MERGE (r {{ id: $id, space_id: $space_id }})
        //         ON CREATE SET r += {{
        //             `{CREATED_AT}`: datetime($created_at),
        //             `{CREATED_AT_BLOCK}`: $created_at_block
        //         }}
        //         SET r:$($label)
        //         SET r += {{
        //             `{UPDATED_AT}`: datetime($updated_at),
        //             `{UPDATED_AT_BLOCK}`: $updated_at_block
        //         }}
        //         "#,
        //         CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
        //         CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
        //         UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
        //         UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        //     );

        //     let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

        //     let query = neo4rs::query(QUERY)
        //         .param("id", entity_id)
        //         .param("space_id", space_id)
        //         .param("created_at", block.timestamp.to_rfc3339())
        //         .param("created_at_block", block.block_number.to_string())
        //         .param("updated_at", block.timestamp.to_rfc3339())
        //         .param("updated_at_block", block.block_number.to_string())
        //         .param("label", uri.id);

        //     Ok(neo4j.run(query).await?)
        // }

        // Set a regular triple
        // (attribute_id, value_type, value) => {
        //     let entity = Entity::<mapping::Triples>::new(
        //         entity_id,
        //         space_id,
        //         block,
        //         mapping::Triples(HashMap::from([(
        //             attribute_id.to_string(),
        //             mapping::Triple {
        //                 value: value.to_string(),
        //                 value_type: mapping::ValueType::try_from(value_type)
        //                     .unwrap_or(mapping::ValueType::Text),
        //                 options: Default::default(),
        //             },
        //         )])),
        //     );

        //     Ok(entity.upsert(neo4j).await?)
        // }
        // }
    }

    pub async fn delete_triples(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        entity_id: &str,
        version_index: i64,
        attributes: Vec<String>,
    ) -> Result<(), DatabaseError> {
        const QUERY: &str = r#"
        MATCH (n {id: $entity_id})
        SET n += {{
            `{UPDATED_AT}`: datetime($updated_at),
            `{UPDATED_AT_BLOCK}`: $updated_at_block
        }}
        WITH n
        UNWIND $attributes AS attribute
        MERGE (n) -[r:TRIPLE {version_index: $version_index, space_id: $space_id, attribute: attribute}]-> (m)
        SET m.value = null
        "#;

        let query = neo4rs::query(QUERY)
            .param("entity_id", entity_id)
            .param("space_id", space_id)
            .param("version_index", version_index)
            .param("attributes", attributes)
            .param(
                "updated_at",
                block.timestamp.to_rfc3339(),
            )
            .param(
                "updated_at_block",
                block.block_number.to_string(),
            );

        Ok(neo4j.run(query).await?)
    }

    pub async fn delete_triple(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        triple: pb::ipfs::Triple,
    ) -> Result<(), DatabaseError> {
        // let delete_triple_query = format!(
        //     r#"
        //     MATCH (n {{ id: $id, space_id: $space_id }})
        //     WITH n, [k IN keys(n) WHERE k CONTAINS "{attribute_label}" | k] as propertyKeys
        //     FOREACH (i IN propertyKeys | REMOVE n[i])
        //     SET n += {{
        //         `{UPDATED_AT}`: datetime($updated_at),
        //         `{UPDATED_AT_BLOCK}`: $updated_at_block
        //     }}
        //     "#,
        //     attribute_label = triple.attribute,
        //     UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
        //     UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        // );

        // let query = neo4rs::query(&delete_triple_query)
        //     .param("id", triple.entity)
        //     .param("space_id", space_id)
        //     .param("created_at", block.timestamp.to_rfc3339())
        //     .param("created_at_block", block.block_number.to_string())
        //     .param("updated_at", block.timestamp.to_rfc3339())
        //     .param("updated_at_block", block.block_number.to_string());

        // Ok(neo4j.run(query).await?)
        todo!()
    }

    pub async fn delete(
        neo4j: &neo4rs::Graph,
        _block: &BlockMetadata,
        id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH (n {{ id: $id, space_id: $space_id }})
        //     DETACH DELETE n
        //     "#,
        // );

        // let query = neo4rs::query(QUERY)
        //     .param("id", id)
        //     .param("space_id", space_id);

        // Ok(neo4j.run(query).await?)
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SetTripleError {
    // #[error("Invalid graph URI: {0}")]
    // InvalidGraphUri(#[from] graph_uri::InvalidGraphUri),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl<T> Entity<T>
where
    T: Clone + TryIntoTriples,
{
    /// Upsert the current entity
    pub async fn upsert(&self, neo4j: &neo4rs::Graph, version: i64) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (n {{id: $entity_id}})
            ON CREATE SET n += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET n += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            WITH n
            UNWIND $triples AS triple
            CALL (n, triple) {{
                MATCH (n) -[r:ATTRIBUTE {{space_id: $space_id}}]-> ({{attribute: triple.attribute}})
                WHERE r.max_version IS null
                SET r.max_version = $version
            }}
            CALL (n, triple) {{
                CREATE (n) -[:ATTRIBUTE {{space_id: $space_id, min_version: $version}}]-> (m)
                SET m = triple
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let triples = self.attributes.clone().try_into_triples()?;

        let query = neo4rs::query(QUERY)
            .param("entity_id", self.id())
            .param("space_id", self.space_id())
            .param("version", version)
            .param("triples", triples)
            .param(
                "created_at",
                self.system_properties.created_at.to_rfc3339(),
            )
            .param(
                "created_at_block",
                self.system_properties.created_at_block.to_string(),
            )
            .param(
                "updated_at",
                self.system_properties.updated_at.to_rfc3339(),
            )
            .param(
                "updated_at_block",
                self.system_properties.updated_at_block.to_string(),
            );

        Ok(neo4j.run(query).await?)

        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MERGE (n {{id: $id}})
        //     ON CREATE SET n += {{
        //         `{CREATED_AT}`: datetime($created_at),
        //         `{CREATED_AT_BLOCK}`: $created_at_block
        //     }}
        //     SET n:$($labels)
        //     SET n += {{
        //         `{UPDATED_AT}`: datetime($updated_at),
        //         `{UPDATED_AT_BLOCK}`: $updated_at_block
        //     }}
        //     UNWIND $triples AS triple
        //     MERGE (n) -[:TRIPLE {{space_id: $space_id, revision: 0}}]-> ()
        //     "#,
        //     CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
        //     CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
        //     UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
        //     UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        // );

        // let bolt_data = match serde_value_to_bolt(serde_json::to_value(self.attributes())?) {
        //     neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
        //     _ => neo4rs::BoltType::Map(Default::default()),
        // };

        // let query = neo4rs::query(QUERY)
        //     .param("id", self.id())
        //     .param("space_id", self.space_id())
        //     .param(
        //         "created_at",
        //         self.system_properties.created_at.to_rfc3339(),
        //     )
        //     .param(
        //         "created_at_block",
        //         self.attributes
        //             .created_at_block
        //             .to_string(),
        //     )
        //     .param(
        //         "updated_at",
        //         self.system_properties.updated_at.to_rfc3339(),
        //     )
        //     .param(
        //         "updated_at_block",
        //         self.attributes
        //             .updated_at_block
        //             .to_string(),
        //     )
        //     .param("labels", self.types.clone())
        //     .param("data", bolt_data);

        // neo4j.run(query).await?;

        // // Add types relations
        // stream::iter(self.types.iter())
        //     .map(Ok)
        //     .try_for_each(|r#type| async move {
        //         let relation = Relation::new(
        //             &create_id_from_unique_string(&format!(
        //                 "{}-{}-{}",
        //                 self.space_id(),
        //                 self.id(),
        //                 r#type
        //             )),
        //             self.space_id(),
        //             system_ids::TYPES_ATTRIBUTE,
        //             self.id(),
        //             r#type,
        //             &BlockMetadata {
        //                 block_number: self
        //                     .attributes
        //                     .system_properties
        //                     .created_at_block
        //                     .parse()
        //                     .expect("Failed to parse block number"),
        //                 timestamp: self.attributes.system_properties.created_at,
        //                 ..Default::default()
        //             },
        //             (),
        //         );

        //         relation.upsert(neo4j).await
        //     })
        //     .await
        // todo!()
    }
}


impl<T> Entity<T>
where
    T: TryFromTriples,
{
    /// Returns the entity with the given ID, if it exists
    pub async fn find_by_id(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        const QUERY: &str =
            const_format::formatcp!("MATCH (n {{id: $id, space_id: $space_id}}) RETURN n",);

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        Self::_find_one(neo4j, query).await
    }

    /// Returns the entities from the given list of IDs
    pub async fn find_by_ids(
        neo4j: &neo4rs::Graph,
        ids: &[String],
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $ids AS id
            MATCH (n {{id: id, space_id: $space_id}})
            RETURN n
            "#
        );

        let query = neo4rs::query(QUERY)
            .param("ids", ids)
            .param("space_id", space_id);

        Self::_find_many(neo4j, query).await
    }

    /// Returns the entities with the given types
    pub async fn find_by_types(
        neo4j: &neo4rs::Graph,
        types: &[String],
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (n:$($types) {{space_id: $space_id}})
            RETURN n
            "#,
        );

        let query = neo4rs::query(QUERY)
            .param("types", types)
            .param("space_id", space_id);

        Self::_find_many(neo4j, query).await
    }

    pub async fn find_many(
        neo4j: &neo4rs::Graph,
        query: Option<entity_queries::FindMany>,
    ) -> Result<Vec<Self>, DatabaseError> {
        let query = if let Some(query) = query {
            query.into_query_part().build()
        } else {
            entity_queries::FindMany::new("n").into_query_part().build()
        };

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.n.try_into()?) })
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn find_one(
        neo4j: &neo4rs::Graph,
        query: Option<entity_queries::FindMany>,
    ) -> Result<Option<Self>, DatabaseError> {
        let query = if let Some(query) = query {
            query.into_query_part().build()
        } else {
            entity_queries::FindMany::new("n").into_query_part().build()
        };

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    async fn _find_one(
        neo4j: &neo4rs::Graph,
        query: neo4rs::Query,
    ) -> Result<Option<Self>, DatabaseError> {
        // #[derive(Debug, Deserialize)]
        // struct RowResult {
        //     n: neo4rs::Node,
        //     r: neo4rs::Relation,
        //     t: neo4rs::Node,
        // }

        // Ok(neo4j
        //     .execute(query)
        //     .await?
        //     .next()
        //     .await?
        //     .map(|row| {
        //         let row = row.to::<RowResult>()?;
        //         row.n.try_into()
        //     })
        //     .transpose()?)
        todo!()
    }

    async fn _find_many(
        neo4j: &neo4rs::Graph,
        query: neo4rs::Query,
    ) -> Result<Vec<Self>, DatabaseError> {
        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.n.try_into()?) })
            .try_collect::<Vec<_>>()
            .await
    }
}

impl<T> TryFrom<neo4rs::Node> for Entity<T>
where
    T: TryFromTriples,
{
    // TODO: Change to better error type
    type Error = DatabaseError;

    fn try_from(value: neo4rs::Node) -> Result<Self, Self::Error> {
        #[derive(Debug, Deserialize)]
        struct RawEntity {
            #[serde(flatten)]
            system_properties: SystemProperties,

            attributes: Vec<super::Triple>
        }

        let labels = value.labels().iter().map(|l| l.to_string()).collect();

        let raw: RawEntity = value.to()?;

        Ok(Self {
            types: labels,
            attributes: T::try_from_triples(raw.attributes.into())?,
            system_properties: raw.system_properties,
            // version: 0,
        })
    }
}

impl Entity<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, attribute_id: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut().insert(attribute_id, value.into());
        self
    }
}

impl Entity<DefaultAttributes> {
    pub fn name(&self) -> Option<String> {
        self.attributes()
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn name_or_id(&self) -> String {
        self.name().unwrap_or_else(|| self.id().to_string())
    }
}

pub type DefaultAttributes = HashMap<String, serde_json::Value>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Named {
    #[serde(rename = "GG8Z4cSkjv8CywbkLqVU5M")]
    pub name: Option<String>,
}

impl Entity<Named> {
    pub fn name_or_id(&self) -> String {
        self.name().unwrap_or_else(|| self.id().to_string())
    }

    pub fn name(&self) -> Option<String> {
        self.attributes().name.clone()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[tokio::test]
    async fn test_find_by_id_no_types() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            foo: String,
        }

        let entity = Entity::new(
            "test_id",
            "test_space_id",
            &BlockMetadata::default(),
            Foo {
                foo: "bar".to_string(),
            },
        );

        entity.upsert(&neo4j, 0).await.unwrap();

        let found_entity = Entity::<Foo>::find_by_id(&neo4j, "test_id", "test_space_id")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(entity, found_entity);
    }

    #[tokio::test]
    async fn test_find_by_id_with_types() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            foo: String,
        }

        let entity = Entity::new(
            "test_id",
            "test_space_id",
            &BlockMetadata::default(),
            Foo {
                foo: "bar".to_string(),
            },
        )
        .with_type("TestType");

        entity.upsert(&neo4j, 0).await.unwrap();

        let found_entity = Entity::<Foo>::find_by_id(&neo4j, "test_id", "test_space_id")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(entity, found_entity);
    }
}
