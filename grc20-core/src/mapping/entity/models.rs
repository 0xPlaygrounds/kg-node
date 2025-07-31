use neo4rs::BoltType;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::entity::FindManyQuery;
use crate::mapping::query_utils::value_filter;
use crate::mapping::value::models::{Value, ValueConversionError};
use crate::{pb, system_ids};

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Value conversion error: {0}")]
    ValueConversionError(#[from] ValueConversionError),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Entity {
    pub id: Uuid,
    pub types: Vec<Uuid>,
    pub values: HashMap<Uuid, HashMap<Uuid, Value>>,
}

impl Entity {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            types: Vec::new(),
            values: HashMap::new(),
        }
    }

    pub fn with_types(mut self, types: Vec<Uuid>) -> Self {
        self.types = types;
        self
    }

    pub fn value(mut self, space_id: impl Into<Uuid>, value: impl Into<Value>) -> Self {
        let val = value.into();

        self.values
            .entry(space_id.into())
            .or_default()
            .insert(val.property, val);
        self
    }

    /// Returns the possible names of the entity
    pub fn names(&self) -> Vec<String> {
        self.values
            .iter()
            .filter_map(|(_, props)| props.get(&system_ids::NAME_PROPERTY))
            .map(|value| value.value.clone())
            .collect()
    }

    /// Returns the possible descriptions of the entity
    pub fn descriptions(&self) -> Vec<String> {
        self.values
            .iter()
            .filter_map(|(_, props)| props.get(&system_ids::DESCRIPTION_PROPERTY))
            .map(|value| value.value.clone())
            .collect()
    }

    /// Returns a query to fetch the entities of the types of the entity
    pub fn get_types(&self, neo4j: &neo4rs::Graph) -> FindManyQuery {
        FindManyQuery::new(neo4j).id(value_filter::value_in(self.types.clone()))
    }

    /// Returns a query to fetch the entities of the properties of the entity
    pub fn get_properties(&self, neo4j: &neo4rs::Graph) -> FindManyQuery {
        FindManyQuery::new(neo4j).id(value_filter::value_in(
            self.values
                .values()
                .flat_map(|props| props.values().map(|v| v.property))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
        ))
    }

    /// Returns a flattened map of all properties across all spaces
    pub fn flattened_properties(&self) -> HashMap<Uuid, Vec<Value>> {
        let mut flattened = HashMap::new();
        for (_, props) in &self.values {
            for value in props.values() {
                flattened
                    .entry(value.property)
                    .or_insert_with(Vec::new)
                    .push(value.clone());
            }
        }
        flattened
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateEntity {
    pub id: Uuid,
    pub values: HashMap<Uuid, Value>,
    pub embedding: Option<Vec<f32>>,
}

impl UpdateEntity {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: HashMap::new(),
            embedding: None,
        }
    }

    pub fn value(mut self, value: impl Into<Value>) -> Self {
        let val = value.into();
        self.values.insert(val.property, val);
        self
    }

    pub fn values(mut self, values: Vec<Value>) -> Self {
        for value in values {
            self.values.insert(value.property, value);
        }
        self
    }

    pub fn embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

impl TryFrom<pb::grc20::Entity> for UpdateEntity {
    type Error = ConversionError;

    fn try_from(entity: pb::grc20::Entity) -> Result<Self, Self::Error> {
        let id = Uuid::from_slice(&entity.id).map_err(|_| {
            ConversionError::InvalidUuid(format!("Invalid entity ID: {:?}", entity.id))
        })?;

        let values: Result<Vec<Value>, ValueConversionError> =
            entity.values.into_iter().map(Value::try_from).collect();

        Ok(Self {
            id,
            values: values?.into_iter().map(|v| (v.property, v)).collect(),
            embedding: None,
        })
    }
}

impl From<UpdateEntity> for BoltType {
    fn from(entity: UpdateEntity) -> Self {
        let mut map = HashMap::new();

        map.insert(
            neo4rs::BoltString { value: "id".into() },
            entity.id.to_string().into(),
        );

        let values: Vec<BoltType> = entity
            .values
            .into_iter()
            .map(|(_, val)| BoltType::from(val))
            .collect();

        map.insert(
            neo4rs::BoltString {
                value: "values".into(),
            },
            values.into(),
        );

        map.insert(
            neo4rs::BoltString {
                value: "embedding".into(),
            },
            entity.embedding.into(),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnsetEntityValues {
    pub id: Uuid,
    pub properties: Vec<Uuid>,
}

impl TryFrom<pb::grc20::UnsetEntityValues> for UnsetEntityValues {
    type Error = ConversionError;

    fn try_from(unset: pb::grc20::UnsetEntityValues) -> Result<Self, Self::Error> {
        let id = Uuid::from_slice(&unset.id).map_err(|_| {
            ConversionError::InvalidUuid(format!("Invalid entity ID: {:?}", unset.id))
        })?;

        let properties: Result<Vec<Uuid>, ConversionError> = unset
            .properties
            .into_iter()
            .map(|prop| {
                Uuid::from_slice(&prop).map_err(|_| {
                    ConversionError::InvalidUuid(format!("Invalid property ID: {:?}", prop))
                })
            })
            .collect();

        Ok(Self {
            id,
            properties: properties?,
        })
    }
}

impl From<UnsetEntityValues> for BoltType {
    fn from(unset: UnsetEntityValues) -> Self {
        let mut map = HashMap::new();

        map.insert(
            neo4rs::BoltString { value: "id".into() },
            unset.id.to_string().into(),
        );

        let mut properties_map = HashMap::new();
        for property_id in unset.properties {
            properties_map.insert(
                neo4rs::BoltString {
                    value: property_id.to_string(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }

        map.insert(
            neo4rs::BoltString {
                value: "properties".into(),
            },
            BoltType::Map(neo4rs::BoltMap {
                value: properties_map,
            }),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl From<Entity> for BoltType {
    fn from(entity: Entity) -> Self {
        let mut map = HashMap::new();
        map.insert(
            neo4rs::BoltString { value: "id".into() },
            entity.id.to_string().into(),
        );

        // Build the "spaces" array
        let spaces: Vec<BoltType> = entity
            .values
            .into_iter()
            .map(|(space_id, values)| {
                let mut space_map = HashMap::new();
                space_map.insert(
                    neo4rs::BoltString {
                        value: "space_id".into(),
                    },
                    space_id.to_string().into(),
                );
                // Convert Vec<Value> to Vec<BoltType>
                let values: Vec<BoltType> = values
                    .into_iter()
                    .map(|(_, val)| BoltType::from(val))
                    .collect();
                space_map.insert(
                    neo4rs::BoltString {
                        value: "values".into(),
                    },
                    values.into(),
                );
                BoltType::Map(neo4rs::BoltMap { value: space_map })
            })
            .collect();

        map.insert(
            neo4rs::BoltString {
                value: "spaces".into(),
            },
            spaces.into(),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}
