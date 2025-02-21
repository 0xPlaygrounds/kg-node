use std::collections::{hash_map, HashMap};

use futures::{Stream, StreamExt, TryStreamExt};
use neo4rs::{BoltList, BoltMap, BoltType};
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata};

use super::{
    query_utils::{query_part, Query, QueryPart, QueryStream, VersionFilter},
    AttributeFilter, AttributeNode, PropFilter, Triple, TriplesConversionError, Value,
};

/// Group of attributes belonging to the same entity.
/// Read and written in bulk
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attributes(pub HashMap<String, AttributeNode>);

impl Attributes {
    pub fn attribute(mut self, attribute: impl Into<AttributeNode>) -> Self {
        let attr = attribute.into();
        self.0.insert(attr.id.clone(), attr);
        self
    }

    pub fn attribute_mut(&mut self, attribute: impl Into<AttributeNode>) {
        let attr = attribute.into();
        self.0.insert(attr.id.clone(), attr);
    }

    pub fn pop<T>(&mut self, attribute_id: &str) -> Result<T, TriplesConversionError>
    where
        T: TryFrom<Value, Error = String>,
    {
        self.0
            .remove(attribute_id)
            .ok_or_else(|| TriplesConversionError::MissingAttribute(attribute_id.to_string()))?
            .value
            .try_into()
            .map_err(TriplesConversionError::InvalidValue)
    }

    pub fn pop_opt<T>(&mut self, attribute_id: &str) -> Result<Option<T>, TriplesConversionError>
    where
        T: TryFrom<Value, Error = String>,
    {
        self.0
            .remove(attribute_id)
            .map(|attr| {
                attr.value
                    .try_into()
                    .map_err(TriplesConversionError::InvalidValue)
            })
            .transpose()
    }

    pub fn get<T>(&self, attribute_id: &str) -> Result<T, TriplesConversionError>
    where
        T: TryFrom<Value, Error = String>,
    {
        self.0
            .get(attribute_id)
            .ok_or_else(|| TriplesConversionError::MissingAttribute(attribute_id.to_string()))?
            .value
            .clone()
            .try_into()
            .map_err(TriplesConversionError::InvalidValue)
    }

    pub fn get_opt<T>(&self, attribute_id: &str) -> Result<Option<T>, TriplesConversionError>
    where
        T: TryFrom<Value, Error = String>,
    {
        self.0
            .get(attribute_id)
            .map(|attr| {
                attr.value
                    .clone()
                    .try_into()
                    .map_err(TriplesConversionError::InvalidValue)
            })
            .transpose()
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
        entity_id: impl Into<String>,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery<Attributes> {
        InsertOneQuery::new(
            neo4j,
            block,
            entity_id.into(),
            space_id.into(),
            space_version.into(),
            self,
        )
    }
}

impl From<Attributes> for BoltType {
    fn from(attributes: Attributes) -> Self {
        BoltType::List(BoltList {
            value: attributes.0.into_values().map(|attr| attr.into()).collect(),
        })
    }
}

impl From<Vec<Triple>> for Attributes {
    fn from(value: Vec<Triple>) -> Self {
        Attributes(
            value
                .into_iter()
                .map(|triple| {
                    (
                        triple.attribute.clone(),
                        AttributeNode {
                            id: triple.attribute,
                            value: triple.value,
                        },
                    )
                })
                .collect(),
        )
    }
}

impl From<Vec<AttributeNode>> for Attributes {
    fn from(value: Vec<AttributeNode>) -> Self {
        Attributes(
            value
                .into_iter()
                .map(|attr| (attr.id.clone(), attr))
                .collect(),
        )
    }
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, entity_id.into(), space_id.into(), space_version)
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn insert_one<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
    attributes: T,
) -> InsertOneQuery<T> {
    InsertOneQuery::new(
        neo4j,
        block,
        entity_id.into(),
        space_id.into(),
        space_version.into(),
        attributes,
    )
}

// /// Aggregate triples by entity as triple sets
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
    space_version: String,
    attributes: T,
}

impl<T> InsertOneQuery<T> {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        entity_id: String,
        space_id: String,
        space_version: String,
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
            MERGE (e:Entity {{id: $entity_id}})
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
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: attribute.id}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: attribute.id}})
                SET m += attribute
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
    space_version: String,
    attributes: Vec<(String, Attributes)>,
}

impl InsertManyQuery {
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
            MERGE (e:Entity {{id: attributes.entity}})
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
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: attribute.id}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: attribute.id}})
                SET m += attribute
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
        space_version: Option<String>,
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
            .match_clause(
                "(:Entity {id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (n:Attribute)",
            )
            .merge(self.space_version.into_query_part("r"))
            .with_clause("collect(n{.*}) AS attrs", query_part::return_query("attrs"))
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
        let query = if cfg!(debug_assertions) {
            let query_part = self.into_query_part();
            tracing::info!("attributes::FindOneQuery:\n{}", query_part);
            query_part.build()
        } else {
            self.into_query_part().build()
        };

        #[derive(Debug, Deserialize)]
        struct RowResult {
            attrs: Vec<AttributeNode>,
        }

        let result = neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.attrs)
            })
            .transpose()?;

        Ok(result
            .map(|attrs| T::from_attributes(attrs.into()))
            .transpose()?)
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    attributes: Vec<AttributeFilter>,

    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl FindManyQuery {
    fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            attributes: vec![],
            space_id: None,
            version: VersionFilter::default(),
        }
    }

    pub fn id(mut self, id: impl Into<PropFilter<String>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.attributes.extend(attributes);
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub(crate) fn into_query_part(self) -> QueryPart {
        let mut query = QueryPart::default()
            .match_clause("(e:Entity) -[r:ATTRIBUTE]-> (n:Attribute)")
            .merge(self.version.into_query_part("r"))
            .with_clause(
                "e, collect(n{.*}) AS attrs",
                query_part::return_query("e{.id, attributes: attrs}"),
            );

        if let Some(id) = self.id {
            query = query.merge(id.into_query_part("e", "id"));
        }

        if let Some(space_id) = self.space_id {
            query = query.merge(space_id.into_query_part("r", "space_id"));
        }

        query
    }
}

impl<T> QueryStream<T> for FindManyQuery
where
    T: FromAttributes,
{
    async fn send(self) -> Result<impl Stream<Item = Result<T, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            #[serde(rename = "id")]
            _id: String,
            attributes: Vec<AttributeNode>,
        }

        let stream = neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|attrs| {
                attrs.and_then(|attrs| {
                    T::from_attributes(attrs.attributes.into()).map_err(DatabaseError::from)
                })
            });

        Ok(stream)
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

// impl<T> IntoAttributes for T
// where
//     T: Serialize,
// {
//     fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
//         if let serde_json::Value::Object(map) = serde_json::to_value(self)? {
//             map.into_iter()
//                 .try_fold(Attributes::default(), |acc, (key, value)| match value {
//                     serde_json::Value::Bool(value) => Ok(acc.attribute((key, value))),
//                     serde_json::Value::Number(value) => {
//                         Ok(acc.attribute((key, Value::number(value.to_string()))))
//                     }
//                     serde_json::Value::String(value) => Ok(acc.attribute((key, value))),
//                     serde_json::Value::Array(_) => {
//                         Err(TriplesConversionError::InvalidValue("Array".into()))
//                     }
//                     serde_json::Value::Object(_) => {
//                         Err(TriplesConversionError::InvalidValue("Object".into()))
//                     }
//                     serde_json::Value::Null => {
//                         Err(TriplesConversionError::InvalidValue("null".into()))
//                     }
//                 })
//         } else {
//             Err(TriplesConversionError::InvalidValue(
//                 "must serialize to serde_json::Map of (String, Scalar) values".into(),
//             ))
//         }
//     }
// }

// impl<T> FromAttributes for T
// where
//     T: for<'a> Deserialize<'a>,
// {
//     fn from_attributes(attributes: Attributes) -> Result<Self, TriplesConversionError> {
//         let obj = attributes
//             .0
//             .into_iter()
//             .map(|(_, attr)| -> (_, serde_json::Value) {
//                 match attr.value {
//                     Value {
//                         value,
//                         value_type: ValueType::Checkbox,
//                         ..
//                     } => (
//                         attr.id,
//                         serde_json::Value::Bool(value.parse().expect("bool should parse")),
//                     ),
//                     Value {
//                         value,
//                         value_type: ValueType::Number,
//                         ..
//                     } => (
//                         attr.id,
//                         serde_json::Value::Number(value.parse().expect("number should parse")),
//                     ),
//                     Value {
//                         value,
//                         value_type: ValueType::Point,
//                         ..
//                     } => (attr.id, serde_json::Value::String(value)),
//                     Value {
//                         value,
//                         value_type: ValueType::Text,
//                         ..
//                     } => (attr.id, serde_json::Value::String(value)),
//                     Value {
//                         value,
//                         value_type: ValueType::Time,
//                         ..
//                     } => (attr.id, serde_json::Value::String(value)),
//                     Value {
//                         value,
//                         value_type: ValueType::Url,
//                         ..
//                     } => (attr.id, serde_json::Value::String(value)),
//                 }
//             })
//             .collect();

//         Ok(serde_json::from_value(obj)?)
//     }
// }

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
    use crate::mapping::{self, entity, prop_filter, Entity};

    use super::*;

    use futures::pin_mut;
    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[derive(Clone, Debug, PartialEq)]
    struct Foo {
        foo: String,
        bar: u64,
    }

    impl mapping::IntoAttributes for Foo {
        fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
            Ok(mapping::Attributes::default()
                .attribute(("foo", self.foo))
                .attribute(("bar", self.bar)))
        }
    }

    impl mapping::FromAttributes for Foo {
        fn from_attributes(
            mut attributes: mapping::Attributes,
        ) -> Result<Self, mapping::TriplesConversionError> {
            Ok(Self {
                foo: attributes.pop("foo")?,
                bar: attributes.pop("bar")?,
            })
        }
    }

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

        let attributes = Attributes::from(vec![
            AttributeNode {
                id: "bar".to_string(),
                value: 123u64.into(),
            },
            AttributeNode {
                id: "foo".to_string(),
                value: "hello".into(),
            },
        ]);

        attributes
            .clone()
            .insert(
                &neo4j,
                &BlockMetadata::default(),
                "abc".to_string(),
                "space_id".to_string(),
                "0",
            )
            .send()
            .await
            .expect("Failed to insert triple set");

        let result: Attributes = find_one(&neo4j, "abc".to_string(), "space_id".to_string(), None)
            .send()
            .await
            .expect("Failed to find triple set")
            .expect("Triple set not found");

        assert_eq!(attributes, result);
    }

    #[tokio::test]
    async fn test_attributes_insert_find_many() {
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

        let attributes = Attributes::from(vec![
            AttributeNode {
                id: "bar".to_string(),
                value: 123u64.into(),
            },
            AttributeNode {
                id: "foo".to_string(),
                value: "hello".into(),
            },
        ]);

        attributes
            .clone()
            .insert(
                &neo4j,
                &BlockMetadata::default(),
                "abc".to_string(),
                "space_id".to_string(),
                "0",
            )
            .send()
            .await
            .expect("Failed to insert triple set");

        let stream = find_many(&neo4j)
            .id(prop_filter::value("abc"))
            .space_id(prop_filter::value("space_id"))
            .send()
            .await
            .expect("Failed to find triple set");

        pin_mut!(stream);

        let result = stream
            .next()
            .await
            .expect("Triple set not found")
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

        let foo = Foo {
            foo: "abc".into(),
            bar: 123,
        };

        insert_one(
            &neo4j,
            &BlockMetadata::default(),
            "abc".to_string(),
            "space_id".to_string(),
            "0",
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

        let foo = Foo {
            foo: "hello".into(),
            bar: 123,
        };

        insert_one(
            &neo4j,
            &BlockMetadata::default(),
            "abc".to_string(),
            "space_id".to_string(),
            "0",
            foo,
        )
        .send()
        .await
        .expect("Insert failed");

        Triple {
            entity: "abc".to_string(),
            attribute: "bar".to_string(),
            value: 456u64.into(),
        }
        .insert(&neo4j, &BlockMetadata::default(), "space_id", "1")
        .send()
        .await
        .expect("Failed to insert triple");

        let foo_v2 = entity::find_one(&neo4j, "abc", "space_id", None)
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(
            foo_v2,
            Entity::new(
                "abc",
                Foo {
                    foo: "hello".into(),
                    bar: 456,
                }
            )
        );

        let foo_v1 = entity::find_one(&neo4j, "abc", "space_id", Some("0".into()))
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(
            foo_v1,
            Entity::new(
                "abc",
                Foo {
                    foo: "hello".into(),
                    bar: 123,
                }
            )
        );
    }
}
