use std::{collections::HashMap, fmt::Display};

use neo4rs::BoltType;
use serde::Deserialize;
use uuid::Uuid;

use crate::pb;

#[derive(Debug, thiserror::Error)]
pub enum ValueConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Invalid options data: {0}")]
    InvalidOptions(String),
    #[error("Missing options value")]
    MissingOptionsValue,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Value {
    pub property: Uuid,
    pub value: String,
    pub options: Option<Options>,
}

impl Value {
    pub fn new(property: impl Into<Uuid>, value: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            value: value.into(),
            options: None,
        }
    }
}

impl TryFrom<pb::grc20::Value> for Value {
    type Error = ValueConversionError;

    fn try_from(pb_value: pb::grc20::Value) -> Result<Self, Self::Error> {
        let property = Uuid::from_bytes(
            pb_value
                .property
                .try_into()
                .map_err(|e| ValueConversionError::InvalidUuid(format!("{e:?}")))?,
        );

        let options = pb_value
            .options
            .map(|opts| Options::try_from(opts))
            .transpose()?;

        Ok(Self {
            property,
            value: pb_value.value,
            options,
        })
    }
}

impl From<Value> for BoltType {
    fn from(value: Value) -> Self {
        let mut map = HashMap::new();
        map.insert(
            neo4rs::BoltString {
                value: "property".into(),
            },
            value.property.to_string().into(),
        );
        map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            value.value.into(),
        );

        if let Some(options) = value.options {
            map.insert(
                neo4rs::BoltString {
                    value: "options".into(),
                },
                options.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Options {
    Text(TextOptions),
    Number(NumberOptions),
}

impl TryFrom<pb::grc20::Options> for Options {
    type Error = ValueConversionError;

    fn try_from(pb_options: pb::grc20::Options) -> Result<Self, Self::Error> {
        match pb_options.value {
            Some(pb::grc20::options::Value::Text(text_opts)) => {
                Ok(Options::Text(TextOptions::try_from(text_opts)?))
            }
            Some(pb::grc20::options::Value::Number(number_opts)) => {
                Ok(Options::Number(NumberOptions::try_from(number_opts)?))
            }
            None => Err(ValueConversionError::MissingOptionsValue),
        }
    }
}

impl From<Options> for BoltType {
    fn from(options: Options) -> Self {
        let mut map = HashMap::new();
        match options {
            Options::Text(text_opts) => {
                map.insert(
                    neo4rs::BoltString {
                        value: "type".into(),
                    },
                    "text".into(),
                );
                map.insert(
                    neo4rs::BoltString {
                        value: "options".into(),
                    },
                    text_opts.into(),
                );
            }
            Options::Number(number_opts) => {
                map.insert(
                    neo4rs::BoltString {
                        value: "type".into(),
                    },
                    "number".into(),
                );
                map.insert(
                    neo4rs::BoltString {
                        value: "options".into(),
                    },
                    number_opts.into(),
                );
            }
        }
        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Text(text_opts) => write!(f, "Text({})", text_opts),
            Options::Number(number_opts) => write!(f, "Number({})", number_opts),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TextOptions {
    pub language: Option<String>,
}

impl TryFrom<pb::grc20::TextOptions> for TextOptions {
    type Error = ValueConversionError;

    fn try_from(pb_text_options: pb::grc20::TextOptions) -> Result<Self, Self::Error> {
        let language = match pb_text_options.language {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(s) => Some(s),
                Err(e) => {
                    tracing::warn!("Conversion error: Invalid UTF-8 in language: {e}. Ignoring.");
                    None
                }
            },
            None => None,
        };
        Ok(Self { language })
    }
}

impl From<TextOptions> for BoltType {
    fn from(text_options: TextOptions) -> Self {
        let mut map = HashMap::new();

        if let Some(language) = text_options.language {
            map.insert(
                neo4rs::BoltString {
                    value: "language".into(),
                },
                language.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl Display for TextOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.language {
            Some(lang) => write!(f, "language: {:?}", lang),
            None => write!(f, "no language"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NumberOptions {
    pub unit: Option<String>,
}

impl TryFrom<pb::grc20::NumberOptions> for NumberOptions {
    type Error = ValueConversionError;

    fn try_from(pb_number_options: pb::grc20::NumberOptions) -> Result<Self, Self::Error> {
        let unit = match pb_number_options.unit {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(s) => Some(s),
                Err(e) => {
                    tracing::warn!("Conversion error: Invalid UTF-8 in unit: {e}. Ignoring.");
                    None
                }
            },
            None => None,
        };
        Ok(Self { unit })
    }
}

impl From<NumberOptions> for BoltType {
    fn from(number_options: NumberOptions) -> Self {
        let mut map = HashMap::new();

        if let Some(unit) = number_options.unit {
            map.insert(
                neo4rs::BoltString {
                    value: "unit".into(),
                },
                unit.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl Display for NumberOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.unit {
            Some(unit) => write!(f, "unit: {:?}", unit),
            None => write!(f, "no unit"),
        }
    }
}
