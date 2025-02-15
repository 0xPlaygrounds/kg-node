use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use neo4rs::BoltType;
use serde::Deserialize;

use crate::pb;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Value {
    pub value: String,
    pub value_type: ValueType,

    #[serde(flatten)]
    pub options: Options,
}

impl Value {
    pub fn text(value: String) -> Self {
        Self {
            value,
            value_type: ValueType::Text,
            options: Options::default(),
        }
    }

    pub fn number(value: impl ToString) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Number,
            options: Options::default(),
        }
    }

    pub fn checkbox(value: bool) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Checkbox,
            options: Options::default(),
        }
    }

    pub fn url(value: String) -> Self {
        Self {
            value,
            value_type: ValueType::Url,
            options: Options::default(),
        }
    }

    pub fn time(value: DateTime<Utc>) -> Self {
        Self {
            value: value.to_rfc3339(),
            value_type: ValueType::Time,
            options: Options::default(),
        }
    }
}

impl Into<BoltType> for Value {
    fn into(self) -> BoltType {
        let mut value_bolt_map = HashMap::new();
        value_bolt_map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            self.value.into(),
        );
        value_bolt_map.insert(
            neo4rs::BoltString {
                value: "value_type".into(),
            },
            self.value_type.to_string().into(),
        );
        if let Some(format) = self.options.format {
            value_bolt_map.insert(
                neo4rs::BoltString {
                    value: "format".into(),
                },
                format.into(),
            );
        }
        if let Some(unit) = self.options.unit {
            value_bolt_map.insert(
                neo4rs::BoltString {
                    value: "unit".into(),
                },
                unit.into(),
            );
        }
        if let Some(language) = self.options.language {
            value_bolt_map.insert(
                neo4rs::BoltString {
                    value: "language".into(),
                },
                language.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap {
            value: value_bolt_map,
        })
    }
}

impl TryFrom<pb::ipfs::Value> for Value {
    type Error = String;

    fn try_from(value: pb::ipfs::Value) -> Result<Self, Self::Error> {
        Ok(Self {
            value_type: value.r#type().try_into()?,
            value: value.value,
            options: Default::default(),
        })
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            value,
            value_type: ValueType::Text,
            options: Options::default(),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Text,
            options: Options::default(),
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Number,
            options: Options::default(),
        }
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Number,
            options: Options::default(),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Number,
            options: Options::default(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            value: value.to_string(),
            value_type: ValueType::Checkbox,
            options: Options::default(),
        }
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(value: DateTime<Utc>) -> Self {
        Self {
            value: value.to_rfc3339(),
            value_type: ValueType::Time,
            options: Options::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Options {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValueType {
    #[default]
    Text,
    Number,
    Checkbox,
    Url,
    Time,
    Point,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Text => write!(f, "TEXT"),
            ValueType::Number => write!(f, "NUMBER"),
            ValueType::Checkbox => write!(f, "CHECKBOX"),
            ValueType::Url => write!(f, "URL"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::Point => write!(f, "POINT"),
        }
    }
}

impl TryFrom<pb::ipfs::ValueType> for ValueType {
    type Error = String;

    fn try_from(value: pb::ipfs::ValueType) -> Result<Self, Self::Error> {
        match value {
            pb::ipfs::ValueType::Text => Ok(ValueType::Text),
            pb::ipfs::ValueType::Number => Ok(ValueType::Number),
            pb::ipfs::ValueType::Checkbox => Ok(ValueType::Checkbox),
            pb::ipfs::ValueType::Url => Ok(ValueType::Url),
            pb::ipfs::ValueType::Time => Ok(ValueType::Time),
            pb::ipfs::ValueType::Point => Ok(ValueType::Point),
            pb::ipfs::ValueType::Unknown => Err("Unknown ValueType".to_string()),
        }
    }
}
