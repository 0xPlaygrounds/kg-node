use futures::stream::TryStreamExt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::DatabaseError,
    graph_uri::{self, GraphUri},
    mapping,
    models::BlockMetadata,
    neo4j_utils::serde_value_to_bolt,
    pb, system_ids,
};

use super::{
    attributes::{Attributes, SystemProperties},
    Relation, Triples,
};

/// GRC20 Node
#[derive(Debug, Deserialize, PartialEq)]
pub struct Entity<T = ()> {
    #[serde(rename = "labels", default)]
    pub types: Vec<String>,
    #[serde(flatten)]
    pub attributes: Attributes<T>,
}

impl<T> Entity<T> {
    /// Creates a new entity with the given ID, space ID, and data
    pub fn new(id: &str, space_id: &str, block: &BlockMetadata, data: T) -> Self {
        Self {
            types: Vec::new(),
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

    pub async fn relations<R>(
        &self,
        neo4j: &neo4rs::Graph,
    ) -> Result<Vec<Relation<R>>, DatabaseError>
    where
        R: for<'a> Deserialize<'a>,
    {
        Self::find_relations(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_relations<R>(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Vec<Relation<R>>, DatabaseError>
    where
        R: for<'a> Deserialize<'a>,
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH ({{ id: $id, space_id: $space_id }}) <-[:`{FROM_ENTITY}`]- (r) -[:`{TO_ENTITY}`]-> (to)
            RETURN to, r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            r: neo4rs::Node,
            to: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                let rel: Entity<R> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;

                Ok(Relation::from_entity(rel, id, to.id()))
            })
            .try_collect::<Vec<_>>()
            .await
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
            MATCH ({{ id: $id, space_id: $space_id }}) <-[:`{FROM_ENTITY}`]- (:`{TYPES}` {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (t {{space_id: $space_id}})
            RETURN t
            "#,
            TYPES = system_ids::TYPES,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
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

    pub async fn set_triple(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        entity_id: &str,
        attribute_id: &str,
        value: &pb::grc20::Value,
    ) -> Result<(), DatabaseError> {
        match (attribute_id, value.r#type(), value.value.as_str()) {
            // Set the type of the entity
            (system_ids::TYPES, pb::grc20::ValueType::Url, value) => {
                const SET_TYPE_QUERY: &str = const_format::formatcp!(
                    r#"
                    MERGE (n {{ id: $id, space_id: $space_id }})
                    ON CREATE SET n += {{
                        `{CREATED_AT}`: datetime($created_at),
                        `{CREATED_AT_BLOCK}`: $created_at_block
                    }}
                    SET n += {{
                        `{UPDATED_AT}`: datetime($updated_at),
                        `{UPDATED_AT_BLOCK}`: $updated_at_block
                    }}
                    SET n:$($labels)
                    "#,
                    CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
                    CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
                    UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
                    UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
                );

                let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

                let query = neo4rs::query(SET_TYPE_QUERY)
                    .param("id", entity_id)
                    .param("space_id", space_id)
                    .param("created_at", block.timestamp.to_rfc3339())
                    .param("created_at_block", block.block_number.to_string())
                    .param("updated_at", block.timestamp.to_rfc3339())
                    .param("updated_at_block", block.block_number.to_string())
                    .param("labels", uri.id);

                Ok(neo4j.run(query).await?)
            }

            // Set the FROM_ENTITY or TO_ENTITY on a relation entity
            (
                system_ids::RELATION_FROM_ATTRIBUTE | system_ids::RELATION_TO_ATTRIBUTE,
                pb::grc20::ValueType::Url,
                value,
            ) => {
                let query = format!(
                    r#"
                    MATCH (n {{ id: $other, space_id: $space_id }})
                    MERGE (r {{ id: $id, space_id: $space_id }})
                    MERGE (r) -[:`{attribute_id}`]-> (n)
                    ON CREATE SET r += {{
                        `{CREATED_AT}`: datetime($created_at),
                        `{CREATED_AT_BLOCK}`: $created_at_block
                    }}
                    SET r += {{
                        `{UPDATED_AT}`: datetime($updated_at),
                        `{UPDATED_AT_BLOCK}`: $updated_at_block
                    }}
                    "#,
                    attribute_id = attribute_id,
                    CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
                    CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
                    UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
                    UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
                );

                let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

                let query = neo4rs::query(&query)
                    .param("id", entity_id)
                    .param("other", uri.id)
                    .param("space_id", space_id)
                    .param("created_at", block.timestamp.to_rfc3339())
                    .param("created_at_block", block.block_number.to_string())
                    .param("updated_at", block.timestamp.to_rfc3339())
                    .param("updated_at_block", block.block_number.to_string());

                Ok(neo4j.run(query).await?)
            }

            // Set the RELATION_TYPE on a relation entity
            (system_ids::RELATION_TYPE_ATTRIBUTE, pb::grc20::ValueType::Url, value) => {
                const QUERY: &str = const_format::formatcp!(
                    r#"
                    MERGE (r {{ id: $id, space_id: $space_id }})
                    ON CREATE SET r += {{
                        `{CREATED_AT}`: datetime($created_at),
                        `{CREATED_AT_BLOCK}`: $created_at_block
                    }}
                    SET r:$($label)
                    SET r += {{
                        `{UPDATED_AT}`: datetime($updated_at),
                        `{UPDATED_AT_BLOCK}`: $updated_at_block
                    }}
                    "#,
                    CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
                    CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
                    UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
                    UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
                );

                let uri = GraphUri::from_uri(value).map_err(SetTripleError::InvalidGraphUri)?;

                let query = neo4rs::query(QUERY)
                    .param("id", entity_id)
                    .param("space_id", space_id)
                    .param("created_at", block.timestamp.to_rfc3339())
                    .param("created_at_block", block.block_number.to_string())
                    .param("updated_at", block.timestamp.to_rfc3339())
                    .param("updated_at_block", block.block_number.to_string())
                    .param("label", uri.id);

                Ok(neo4j.run(query).await?)
            }

            // Set a regular triple
            (attribute_id, value_type, value) => {
                let entity = Entity::<mapping::Triples>::new(
                    entity_id,
                    space_id,
                    block,
                    mapping::Triples(HashMap::from([(
                        attribute_id.to_string(),
                        mapping::Triple {
                            value: value.to_string(),
                            value_type: mapping::ValueType::try_from(value_type)
                                .unwrap_or(mapping::ValueType::Text),
                            options: Default::default(),
                        },
                    )])),
                );

                Ok(entity.upsert(neo4j).await?)
            }
        }
    }

    pub async fn delete_triple(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        triple: pb::grc20::Triple,
    ) -> Result<(), DatabaseError> {
        let delete_triple_query = format!(
            r#"
            MATCH (n {{ id: $id, space_id: $space_id }})
            REMOVE n.`{attribute_label}`
            SET n += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            "#,
            attribute_label = triple.attribute,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(&delete_triple_query)
            .param("id", triple.entity)
            .param("space_id", space_id)
            .param("created_at", block.timestamp.to_rfc3339())
            .param("created_at_block", block.block_number.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string());

        Ok(neo4j.run(query).await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SetTripleError {
    #[error("Invalid graph URI: {0}")]
    InvalidGraphUri(#[from] graph_uri::InvalidGraphUri),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl<T> Entity<T>
where
    T: Serialize,
{
    /// Upsert the current entity
    pub async fn upsert(&self, neo4j: &neo4rs::Graph) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (n {{id: $id, space_id: $space_id}})
            ON CREATE SET n += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET n:$($labels)
            SET n += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET n += $data
            "#,
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

impl<T> Entity<T>
where
    T: for<'a> Deserialize<'a>,
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

    pub async fn find_all(
        neo4j: &neo4rs::Graph,
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str =
            const_format::formatcp!("MATCH (n {{space_id: $space_id}}) RETURN n LIMIT 100");

        let query = neo4rs::query(QUERY).param("space_id", space_id);

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
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = neo4rs::DeError;

    fn try_from(value: neo4rs::Node) -> Result<Self, Self::Error> {
        let labels = value.labels().iter().map(|l| l.to_string()).collect();
        let attributes = value.to()?;
        Ok(Self {
            types: labels,
            attributes,
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
        let container = GenericImage::new("neo4j", "latest")
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

        #[derive(Debug, Deserialize, Serialize, PartialEq)]
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

        entity.upsert(&neo4j).await.unwrap();

        let found_entity = Entity::<Foo>::find_by_id(&neo4j, "test_id", "test_space_id")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(entity, found_entity);
    }

    #[tokio::test]
    async fn test_find_by_id_with_types() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "latest")
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

        #[derive(Debug, Deserialize, Serialize, PartialEq)]
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

        entity.upsert(&neo4j).await.unwrap();

        let found_entity = Entity::<Foo>::find_by_id(&neo4j, "test_id", "test_space_id")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(entity, found_entity);
    }
}
