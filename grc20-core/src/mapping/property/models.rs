use std::{collections::HashMap, fmt::Display};

use neo4rs::BoltType;
use serde::Deserialize;
use uuid::Uuid;

use crate::pb;

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Invalid data type: {0}")]
    InvalidDataType(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    pub id: Uuid,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DataType {
    Text,
    Number,
    Checkbox,
    Time,
    Point,
    Relation,
}

impl TryFrom<pb::grc20::Property> for Property {
    type Error = ConversionError;

    fn try_from(value: pb::grc20::Property) -> Result<Self, Self::Error> {
        let id = Uuid::from_bytes(
            value
                .id
                .try_into()
                .map_err(|e| ConversionError::InvalidUuid(format!("{e:?}")))?,
        );
        let data_type = pb::grc20::DataType::try_from(value.data_type)
            .map_err(|e| ConversionError::InvalidDataType(e.to_string()))?
            .into();

        Ok(Property { id, data_type })
    }
}

impl From<pb::grc20::DataType> for DataType {
    fn from(value: pb::grc20::DataType) -> Self {
        match value {
            pb::grc20::DataType::Text => DataType::Text,
            pb::grc20::DataType::Number => DataType::Number,
            pb::grc20::DataType::Checkbox => DataType::Checkbox,
            pb::grc20::DataType::Time => DataType::Time,
            pb::grc20::DataType::Point => DataType::Point,
            pb::grc20::DataType::Relation => DataType::Relation,
        }
    }
}

impl From<Property> for BoltType {
    fn from(property: Property) -> Self {
        let mut map = HashMap::new();
        map.insert(
            neo4rs::BoltString { value: "id".into() },
            property.id.to_string().into(),
        );
        map.insert(
            neo4rs::BoltString {
                value: "data_type".into(),
            },
            property.data_type.to_string().into(),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl From<DataType> for BoltType {
    fn from(data_type: DataType) -> Self {
        data_type.to_string().into()
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Text => write!(f, "TEXT"),
            DataType::Number => write!(f, "NUMBER"),
            DataType::Checkbox => write!(f, "CHECKBOX"),
            DataType::Time => write!(f, "TIME"),
            DataType::Point => write!(f, "POINT"),
            DataType::Relation => write!(f, "RELATION"),
        }
    }
}

impl Property {
    pub fn text(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Text,
        }
    }

    pub fn number(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Number,
        }
    }

    pub fn checkbox(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Checkbox,
        }
    }

    pub fn time(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Time,
        }
    }

    pub fn point(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Point,
        }
    }

    pub fn relation(id: Uuid) -> Self {
        Property {
            id,
            data_type: DataType::Relation,
        }
    }
}
