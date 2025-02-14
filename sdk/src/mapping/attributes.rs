use std::collections::{hash_map, HashMap};

use neo4rs::{BoltList, BoltMap, BoltType};
use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata};

use super::{
    query_utils::{Query, QueryPart, VersionFilter},
    AttributeNode, Triple, TriplesConversionError, Value, ValueType,
};

/// Group of attributes belonging to the same entity.
/// Read and written in bulk
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attributes(pub Vec<AttributeNode>);

impl Attributes {
    pub fn attribute(mut self, attribute: impl Into<AttributeNode>) -> Self {
        self.0.push(attribute.into());
        self
    }

    pub fn attribute_mut(&mut self, attribute: impl Into<AttributeNode>) {
        self.0.push(attribute.into());
    }

    // pub fn iter(&self) -> Iter {
    //     Iter {
    //         items: self.triples.iter(),
    //     }
    // }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        entity_id: String,
        space_id: String,
        space_version: i64,
    ) -> InsertOneQuery<Attributes> {
        InsertOneQuery::new(neo4j, block, entity_id, space_id, space_version, self)
    }
}

impl Into<BoltType> for Attributes {
    fn into(self) -> BoltType {
        BoltType::List(BoltList {
            value: self.0.into_iter().map(|attr| attr.into()).collect(),
        })
    }
}

impl From<Vec<Triple>> for Attributes {
    fn from(value: Vec<Triple>) -> Self {
        Attributes(
            value
                .into_iter()
                .map(|triple| AttributeNode {
                    id: triple.attribute,
                    value: triple.value,
                })
                .collect(),
        )
    }
}

impl From<Vec<AttributeNode>> for Attributes {
    fn from(value: Vec<AttributeNode>) -> Self {
        Attributes(value)
    }
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: String,
    space_version: i64,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id, space_version)
}

pub fn insert_one<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: i64,
    attributes: T,
) -> InsertOneQuery<T> {
    InsertOneQuery::new(
        neo4j,
        block,
        entity_id.into(),
        space_id.into(),
        space_version,
        attributes,
    )
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<i64>,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, entity_id.into(), space_id.into(), space_version)
}

/// Aggregate triples by entity as triple sets
// pub fn aggregate(triples: Vec<Triple>) -> Vec<Attributes> {
//     let mut map = HashMap::new();

//     for triple in triples {
//         let entity = triple.entity.clone();

//         map.entry(entity)
//             .or_insert_with(Vec::new)
//             .push(triple.into());
//     }

//     map.into_iter()
//         .map(|(entity, triples)| Attributes { attributes: triples })
//         .collect()
// }

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    entity_id: String,
    space_id: String,
    space_version: i64,
    attributes: T,
}

impl<T> InsertOneQuery<T> {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        entity_id: String,
        space_id: String,
        space_version: i64,
        attributes: T,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            entity_id,
            space_id,
            space_version,
            attributes,
        }
    }
}

impl<T: IntoAttributes> Query<()> for InsertOneQuery<T> {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e {{id: $entity_id}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            UNWIND $attributes AS attribute
            CALL (e, attribute) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> ({{id: attribute.id}})
                WHERE r.max_version IS null
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m)
                SET m = attribute
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("entity_id", self.entity_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("attributes", self.attributes.into_attributes()?)
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
    space_version: i64,
    attributes: Vec<(String, Attributes)>,
}

impl InsertManyQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            attributes: vec![],
        }
    }

    pub fn attributes(mut self, entity_id: String, attributes: Attributes) -> Self {
        self.attributes.push((entity_id, attributes));
        self
    }

    pub fn attributes_mut(&mut self, entity_id: String, attributes: Attributes) {
        self.attributes.push((entity_id, attributes));
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $attributes AS attributes
            MERGE (e {{id: attributes.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            UNWIND attributes.attributes AS attribute
            CALL (e, attribute) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> ({{id: attribute.id}})
                WHERE r.max_version IS null
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m)
                SET m = attribute
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
            .param(
                "attributes",
                self.attributes
                    .into_iter()
                    .map(|(entity, attrs)| {
                        BoltType::Map(BoltMap {
                            value: HashMap::from([
                                (
                                    neo4rs::BoltString {
                                        value: "entity".into(),
                                    },
                                    entity.into(),
                                ),
                                (
                                    neo4rs::BoltString {
                                        value: "attributes".into(),
                                    },
                                    attrs.into(),
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

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    entity_id: String,
    space_id: String,
    space_version: VersionFilter,
}

impl FindOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        entity_id: String,
        space_id: String,
        space_version: Option<i64>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            entity_id,
            space_id,
            space_version: VersionFilter::new(space_version),
        }
    }

    fn into_query_part(self) -> QueryPart {
        QueryPart::default()
            .match_clause("(e {id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (n {attribute: $attribute_id})")
            .merge(self.space_version.into_query_part("r"))
            .with_clause("e, collect(n{.*}) AS triples")
            .return_clause("RETURN e{entity: .id, triples: triples")
            .params("entity_id", self.entity_id)
            .params("space_id", self.space_id)
    }
}

impl<T> Query<Option<T>> for FindOneQuery
where
    T: FromAttributes,
{
    async fn send(self) -> Result<Option<T>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: Vec<AttributeNode>,
        }

        let result = neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.n)
            })
            .transpose()?;

        Ok(result
            .map(|attrs| T::from_attributes(attrs.into()))
            .transpose()?)
    }
}

pub trait FromAttributes: Sized {
    fn from_attributes(attributes: Attributes) -> Result<Self, TriplesConversionError>;
}

impl FromAttributes for Attributes {
    fn from_attributes(attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(attributes)
    }
}

/// Trait to convert a type into Triples
pub trait IntoAttributes {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError>;
}

impl IntoAttributes for Attributes {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        Ok(self)
    }
}

impl<T> IntoAttributes for T
where
    T: Serialize,
{
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        if let serde_json::Value::Object(map) = serde_json::to_value(self)? {
            map.into_iter()
                .try_fold(Attributes::default(), |acc, (key, value)| match value {
                    serde_json::Value::Bool(value) => Ok(acc.attribute((key, value))),
                    serde_json::Value::Number(value) => {
                        Ok(acc.attribute((key, Value::number(value.to_string()))))
                    }
                    serde_json::Value::String(value) => Ok(acc.attribute((key, value))),
                    serde_json::Value::Array(_) => {
                        Err(TriplesConversionError::InvalidValue("Array".into()))
                    }
                    serde_json::Value::Object(_) => {
                        Err(TriplesConversionError::InvalidValue("Object".into()))
                    }
                    serde_json::Value::Null => {
                        Err(TriplesConversionError::InvalidValue("null".into()))
                    }
                })
        } else {
            Err(TriplesConversionError::InvalidValue(
                "must serialize to serde_json::Map of (String, Scalar) values".into(),
            ))
        }
    }
}

impl<T> FromAttributes for T
where
    T: for<'a> Deserialize<'a>,
{
    fn from_attributes(attributes: Attributes) -> Result<Self, TriplesConversionError> {
        let obj = attributes
            .0
            .into_iter()
            .map(|attr| -> (_, serde_json::Value) {
                match attr.value {
                    Value {
                        value,
                        value_type: ValueType::Checkbox,
                        ..
                    } => (attr.id, value.parse().expect("bool should parse")),
                    Value {
                        value,
                        value_type: ValueType::Number,
                        ..
                    } => (attr.id, value.parse().expect("number should parse")),
                    Value {
                        value,
                        value_type: ValueType::Point,
                        ..
                    } => (attr.id, value.parse().expect("point should parse")),
                    Value {
                        value,
                        value_type: ValueType::Text,
                        ..
                    } => (attr.id, value.parse().expect("text should parse")),
                    Value {
                        value,
                        value_type: ValueType::Time,
                        ..
                    } => (attr.id, value.parse().expect("time should parse")),
                    Value {
                        value,
                        value_type: ValueType::Url,
                        ..
                    } => (attr.id, value.parse().expect("url should parse")),
                }
            })
            .collect();

        Ok(serde_json::from_value(obj)?)
    }
}

pub struct Iter<'a> {
    items: hash_map::Iter<'a, String, Triple>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Triple);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
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
    async fn test_attributes_insert_find_one() {
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

        let attributes = Attributes(vec![
            AttributeNode {
                id: "foo".to_string(),
                value: "hello".into(),
            },
            AttributeNode {
                id: "bar".to_string(),
                value: 123u64.into(),
            },
        ]);

        attributes
            .clone()
            .insert(
                &neo4j,
                &BlockMetadata::default(),
                "abc".to_string(),
                "space_id".to_string(),
                0,
            )
            .send()
            .await
            .expect("Failed to insert triple set");

        let result = find_one(&neo4j, "abc".to_string(), "space_id".to_string(), None)
            .send()
            .await
            .expect("Failed to find triple set")
            .expect("Triple set not found");

        assert_eq!(attributes, result);
    }

    #[tokio::test]
    async fn test_attributes_insert_find_one_parse() {
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
            bar: u64,
        }

        let foo = Foo {
            foo: "abc".into(),
            bar: 123,
        };

        insert_one(
            &neo4j,
            &BlockMetadata::default(),
            "abc".to_string(),
            "space_id".to_string(),
            0,
            foo.clone(),
        )
        .send()
        .await
        .expect("Insert failed");

        let result = find_one(&neo4j, "abc".to_string(), "space_id".to_string(), None)
            .send()
            .await
            .expect("Failed to find triple set")
            .expect("Triple set not found");

        assert_eq!(foo, result);
    }
}
