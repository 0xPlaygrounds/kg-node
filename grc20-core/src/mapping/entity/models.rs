use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{
    block::BlockMetadata,
    mapping::{
        attributes, entity_version, prop_filter, triple, AttributeNode, EntityFilter, Triple,
    },
    relation::{self, utils::RelationFilter},
};

use super::{insert_one, DeleteOneQuery, InsertOneQuery};

/// Neo4j model of an Entity
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct EntityNode {
    pub id: String,

    /// System properties
    #[serde(flatten)]
    pub system_properties: SystemProperties,
}

impl EntityNode {
    pub fn delete(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> DeleteOneQuery {
        DeleteOneQuery::new(neo4j, block, self.id, space_id.into(), space_version.into())
    }

    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.into(), space_version)
    }

    pub fn get_outbound_relations<T>(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation::FindManyQuery<T> {
        relation::find_many(neo4j)
            .filter(
                RelationFilter::default()
                    .from_(EntityFilter::default().id(prop_filter::value(self.id.clone()))),
            )
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn get_inbound_relations<T>(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation::FindManyQuery<T> {
        relation::find_many(neo4j)
            .filter(
                RelationFilter::default()
                    .to_(EntityFilter::default().id(prop_filter::value(self.id.clone()))),
            )
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.into(),
            space_version.into(),
            Triple::new(self.id.clone(), attribute.id, attribute.value),
        )
    }

    pub fn set_attributes<T>(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attributes: T,
    ) -> attributes::InsertOneQuery<T> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.into(),
            space_version.into(),
            attributes,
        )
    }

    /// Get all the versions that have been applied to this entity
    pub fn versions(&self, neo4j: &neo4rs::Graph) -> entity_version::FindManyQuery {
        entity_version::FindManyQuery::new(neo4j.clone(), self.id.clone())
    }
}

/// Reference to an entity node
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
#[serde(transparent)]
pub struct EntityNodeRef(pub String);

impl From<EntityNodeRef> for String {
    fn from(node_ref: EntityNodeRef) -> Self {
        node_ref.0
    }
}

impl From<&EntityNodeRef> for String {
    fn from(node_ref: &EntityNodeRef) -> Self {
        node_ref.0.clone()
    }
}

impl From<EntityNode> for EntityNodeRef {
    fn from(node: EntityNode) -> Self {
        Self(node.id)
    }
}

impl From<String> for EntityNodeRef {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&String> for EntityNodeRef {
    fn from(id: &String) -> Self {
        Self(id.clone())
    }
}

impl Display for EntityNodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// High level model encapsulating an entity with its attributes and types.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity<T> {
    pub(crate) node: EntityNode,
    pub attributes: T,
    pub types: Vec<String>,
}

impl<T> Entity<T> {
    pub fn new(id: impl Into<String>, attributes: T) -> Self {
        Entity {
            node: EntityNode {
                id: id.into(),
                system_properties: SystemProperties::default(),
            },
            attributes,
            types: vec![],
        }
    }

    pub fn id(&self) -> &str {
        &self.node.id
    }

    pub fn system_properties(&self) -> &SystemProperties {
        &self.node.system_properties
    }

    pub fn with_type(mut self, r#type: impl Into<String>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn with_types(mut self, types: impl IntoIterator<Item = String>) -> Self {
        self.types.extend(types);
        self
    }

    pub fn get_outbound_relations<U>(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation::FindManyQuery<U> {
        relation::find_many(neo4j)
            .filter(
                RelationFilter::default()
                    .from_(EntityFilter::default().id(prop_filter::value(&self.node.id))),
            )
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn get_inbound_relations<U>(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation::FindManyQuery<U> {
        relation::find_many(neo4j)
            .filter(
                RelationFilter::default()
                    .to_(EntityFilter::default().id(prop_filter::value(&self.node.id))),
            )
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery<Self> {
        insert_one::<Self>(neo4j, block, self, space_id.into(), space_version.into())
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct SystemProperties {
    #[serde(rename = "82nP7aFmHJLbaPFszj2nbx")] // CREATED_AT_TIMESTAMP
    pub created_at: DateTime<Utc>,
    #[serde(rename = "59HTYnd2e4gBx2aA98JfNx")] // CREATED_AT_BLOCK
    pub created_at_block: String,
    #[serde(rename = "5Ms1pYq8v8G1RXC3wWb9ix")] // UPDATED_AT_TIMESTAMP
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "7pXCVQDV9C7ozrXkpVg8RJ")] // UPDATED_AT_BLOCK
    pub updated_at_block: String,
    #[serde(rename = "pagerank")]
    pub page_rank: Option<f64>,
}

impl From<BlockMetadata> for SystemProperties {
    fn from(block: BlockMetadata) -> Self {
        Self {
            created_at: block.timestamp,
            created_at_block: block.block_number.to_string(),
            updated_at: block.timestamp,
            updated_at_block: block.block_number.to_string(),
            page_rank: None,
        }
    }
}

impl Default for SystemProperties {
    fn default() -> Self {
        Self {
            created_at: Default::default(),
            created_at_block: "0".to_string(),
            updated_at: Default::default(),
            updated_at_block: "0".to_string(),
            page_rank: None,
        }
    }
}
