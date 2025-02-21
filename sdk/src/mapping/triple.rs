use std::collections::HashMap;

use futures::{Stream, TryStreamExt};
use neo4rs::{BoltMap, BoltType};
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata, pb};

use super::{
    query_utils::{PropFilter, Query, QueryPart, QueryStream, VersionFilter},
    Value,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Triple {
    pub entity: String,

    #[serde(alias = "id")]
    pub attribute: String,

    #[serde(flatten)]
    pub value: Value,
}

impl Triple {
    pub fn new(
        entity: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            entity: entity.into(),
            attribute: attribute.into(),
            value: value.into(),
        }
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version.into(), self)
    }
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    attribute_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j,
        block,
        attribute_id.into(),
        entity_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

pub fn delete_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    attribute_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneQuery {
    FindOneQuery::new(
        neo4j,
        attribute_id.into(),
        entity_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

impl TryFrom<pb::ipfs::Triple> for Triple {
    type Error = String;

    fn try_from(triple: pb::ipfs::Triple) -> Result<Self, Self::Error> {
        if let Some(value) = triple.value {
            Ok(Triple {
                entity: triple.entity,
                attribute: triple.attribute,
                value: value.try_into()?,
            })
        } else {
            Err("Triple value is required".to_string())
        }
    }
}

impl From<Triple> for BoltType {
    fn from(triple: Triple) -> Self {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "entity".into(),
            },
            triple.entity.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "attribute".into(),
            },
            triple.attribute.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            triple.value.into(),
        );

        BoltType::Map(neo4rs::BoltMap {
            value: triple_bolt_map,
        })
    }
}

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triple: Triple,
}

impl InsertOneQuery {
    pub(crate) fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
        triple: Triple,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triple,
        }
    }
}

impl Query<()> for InsertOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e:Entity {{id: $triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            CALL (e) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: $triple.attribute}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e) {{
                MERGE (e) -[r:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: $triple.attribute}})
                SET m += $triple.value
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triple", self.triple)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct InsertManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triples: Vec<Triple>,
}

impl InsertManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triples: vec![],
        }
    }

    pub fn triple(mut self, triple: Triple) -> Self {
        self.triples.push(triple);
        self
    }

    pub fn triple_mut(&mut self, triple: Triple) {
        self.triples.push(triple);
    }

    pub fn triples(mut self, triples: impl IntoIterator<Item = Triple>) -> Self {
        self.triples.extend(triples);
        self
    }

    pub fn triples_mut(&mut self, triples: impl IntoIterator<Item = Triple>) {
        self.triples.extend(triples);
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $triples as triple
            MERGE (e:Entity {{id: triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e, triple
            CALL (e, triple) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: triple.attribute}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, triple) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: triple.attribute}})
                SET m += triple.value
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triples", self.triples)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: VersionFilter,
}

impl FindOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        space_version: Option<String>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id,
            entity_id,
            space_id,
            space_version: VersionFilter::new(space_version),
        }
    }
}

impl Query<Option<Triple>> for FindOneQuery {
    async fn send(self) -> Result<Option<Triple>, DatabaseError> {
        let query_part = QueryPart::default()
            .match_clause("(e:Entity {id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (n:Attribute {id: $attribute_id})")
            .merge(self.space_version.into_query_part("r"))
            .return_clause("n{.*, entity: e.id} AS triple")
            .params("attribute_id", self.attribute_id)
            .params("entity_id", self.entity_id)
            .params("space_id", self.space_id);

        if cfg!(debug_assertions) {
            tracing::info!("triple::FindOneQuery:\n{}", query_part.query());
        }
        let query = query_part.build();

        self.neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                // NOTE: When returning a projection, you can deserialize directly to
                // the struct without an intermediate "RowResult" struct since neo4j
                // will not return the data as a "Node" but instead as raw JSON.
                // let row = row.to::<RowResult>()?;
                // Result::<_, DatabaseError>::Ok(row.triple)
                row.to::<Triple>().map_err(DatabaseError::from)
            })
            .transpose()
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    attribute_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    entity_id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    space_version: VersionFilter,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id: None,
            value: None,
            value_type: None,
            entity_id: None,
            space_id: None,
            space_version: VersionFilter::default(),
        }
    }

    pub fn attribute_id(mut self, filter: PropFilter<String>) -> Self {
        self.attribute_id = Some(filter);
        self
    }

    pub fn value(mut self, filter: PropFilter<String>) -> Self {
        self.value = Some(filter);
        self
    }

    pub fn value_type(mut self, filter: PropFilter<String>) -> Self {
        self.value_type = Some(filter);
        self
    }

    pub fn entity_id(mut self, filter: PropFilter<String>) -> Self {
        self.entity_id = Some(filter);
        self
    }

    pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
        self.space_id = Some(filter);
        self
    }

    pub fn space_version(mut self, space_version: impl Into<String>) -> Self {
        self.space_version.version_mut(space_version.into());
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part =
            QueryPart::default().match_clause("(e:Entity) -[r:ATTRIBUTE]-> (n:Attribute)");

        if let Some(attribute_id) = self.attribute_id {
            query_part = query_part.merge(attribute_id.into_query_part("n", "id"));
        }

        if let Some(value) = self.value {
            query_part = query_part.merge(value.into_query_part("n", "value"));
        }

        if let Some(value_type) = self.value_type {
            query_part = query_part.merge(value_type.into_query_part("n", "value_type"));
        }

        if let Some(entity_id) = self.entity_id {
            query_part = query_part.merge(entity_id.into_query_part("e", "id"));
        }

        if let Some(space_id) = self.space_id {
            query_part = query_part.merge(space_id.into_query_part("r", "space_id"));
        }

        query_part
            .merge(self.space_version.into_query_part("r"))
            .return_clause("n{.*, entity: e.id}")
    }
}

impl QueryStream<Triple> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Triple, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = if cfg!(debug_assertions) {
            let query_part = self.into_query_part();
            tracing::info!("triple::FindManyQuery:\n{}", query_part.query());
            query_part.build()
        } else {
            self.into_query_part().build()
        };

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<Triple>()
            .map_err(DatabaseError::from))
    }
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: String,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            attribute_id,
            entity_id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e:Entity {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: $attribute_id}})
            WHERE r.max_version IS null
            SET r.max_version = $space_version
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("attribute_id", self.attribute_id)
            .param("entity_id", self.entity_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct DeleteManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triples: Vec<(String, String)>,
}

impl DeleteManyQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triples: vec![],
        }
    }

    pub fn triple(mut self, entity_id: impl Into<String>, attribute_id: impl Into<String>) -> Self {
        self.triples.push((entity_id.into(), attribute_id.into()));
        self
    }

    pub fn triple_mut(&mut self, entity_id: impl Into<String>, attribute_id: impl Into<String>) {
        self.triples.push((entity_id.into(), attribute_id.into()));
    }

    pub fn triples(mut self, triples: impl IntoIterator<Item = (String, String)>) -> Self {
        self.triples.extend(triples);
        self
    }

    pub fn triples_mut(&mut self, triples: impl IntoIterator<Item = (String, String)>) {
        self.triples.extend(triples);
    }
}

impl Query<()> for DeleteManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $triples as triple
            MATCH (e:Entity {{id: triple.entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: triple.attribute_id}})
            WHERE r.max_version IS null
            SET r.max_version = $space_version
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param(
                "triples",
                self.triples
                    .into_iter()
                    .map(|(entity_id, attribute_id)| {
                        BoltType::Map(BoltMap {
                            value: HashMap::from([
                                (
                                    neo4rs::BoltString {
                                        value: "entity_id".into(),
                                    },
                                    entity_id.into(),
                                ),
                                (
                                    neo4rs::BoltString {
                                        value: "attribute_id".into(),
                                    },
                                    attribute_id.into(),
                                ),
                            ]),
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
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
    async fn test_find_one() {
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

        neo4j.run(
            neo4rs::query(r#"CREATE (:Entity {id: "abc"}) -[:ATTRIBUTE {space_id: "ROOT", min_version: 0}]-> (:Attribute {id: "name", value: "Alice", value_type: "TEXT"})"#)
        )
        .await
        .expect("Failed to create test data");

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        let found_triple = find_one(&neo4j, "name", "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find triple")
            .expect("Triple not found");

        assert_eq!(triple, found_triple);
    }

    #[tokio::test]
    async fn test_insert_find_one() {
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

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let found_triple = find_one(&neo4j, "name", "abc", "ROOT", Some("0".into()))
            .send()
            .await
            .expect("Failed to find triple")
            .expect("Triple not found");

        assert_eq!(triple, found_triple);
    }

    #[tokio::test]
    pub async fn test_insert_many() {
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

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        let other_triple = Triple {
            entity: "def".to_string(),
            attribute: "name".to_string(),
            value: "Bob".into(),
        };

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![triple.clone(), other_triple])
            .send()
            .await
            .expect("Failed to insert triples");

        let found_triples = FindManyQuery::new(&neo4j)
            .attribute_id(PropFilter::default().value("name"))
            .value(PropFilter::default().value("Alice"))
            .value_type(PropFilter::default().value("TEXT"))
            .entity_id(PropFilter::default().value("abc"))
            .space_id(PropFilter::default().value("ROOT"))
            .space_version("0")
            .send()
            .await
            .expect("Failed to find triples")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(vec![triple], found_triples);
    }

    #[tokio::test]
    pub async fn test_insert_find_many() {
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

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let other_triple = Triple {
            entity: "def".to_string(),
            attribute: "name".to_string(),
            value: "Bob".into(),
        };

        other_triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let found_triples = FindManyQuery::new(&neo4j)
            .attribute_id(PropFilter::default().value("name"))
            .value(PropFilter::default().value("Alice"))
            .value_type(PropFilter::default().value("TEXT"))
            .entity_id(PropFilter::default().value("abc"))
            .space_id(PropFilter::default().value("ROOT"))
            .space_version("0")
            .send()
            .await
            .expect("Failed to find triples")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(vec![triple], found_triples);
    }

    #[tokio::test]
    async fn test_versioning() {
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

        let triple_v1 = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple_v1
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_v2 = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "NotAlice".into(),
        };

        triple_v2
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "1")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_latest = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, triple_latest);
        assert_eq!(triple_latest.value.value, "NotAlice".to_string());

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v1, found_triple_v1);
        assert_eq!(found_triple_v1.value.value, "Alice".to_string());
    }

    #[tokio::test]
    async fn test_update_no_versioning() {
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

        let triple_v1 = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple_v1
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_v2 = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "NotAlice".into(),
        };

        triple_v2
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_latest = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, triple_latest);
        assert_eq!(triple_latest.value.value, "NotAlice".to_string());

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, found_triple_v1);
        assert_eq!(found_triple_v1.value.value, "NotAlice".to_string());
    }

    #[tokio::test]
    async fn test_delete() {
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

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        delete_one(
            &neo4j,
            &BlockMetadata::default(),
            "name",
            "abc",
            "ROOT",
            "1",
        )
        .send()
        .await
        .expect("Failed to delete triple");

        let found_triple = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple");

        assert_eq!(None, found_triple);

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple, found_triple_v1);
    }
}
