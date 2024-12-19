use std::collections::HashMap;

use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::pb;

#[derive(Clone, Debug, PartialEq)]
pub struct Triples(pub(crate) HashMap<String, Triple>);

impl Serialize for Triples {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, &value.value)?;
            map.serialize_entry(&format!("{}.type", key), &value.r#type)?;
            if let Some(ref format) = value.options.format {
                map.serialize_entry(&format!("{}.options.format", key), format)?;
            }
            if let Some(ref unit) = value.options.unit {
                map.serialize_entry(&format!("{}.options.unit", key), unit)?;
            }
            if let Some(ref language) = value.options.language {
                map.serialize_entry(&format!("{}.options.language", key), language)?;
            }
       }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Triples {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TriplesVisitor;

        impl<'de> serde::de::Visitor<'de> for TriplesVisitor {
            type Value = Triples;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map representing triples")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut triples = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.split('.').collect::<Vec<_>>()[..] {
                        [key] => {
                            let value = map.next_value::<String>()?;
                            triples.entry(key.to_string()).or_insert(Triple::default()).value = value;
                        }
                        [key, "type"] => {
                            let value = map.next_value::<ValueType>()?;
                            triples.entry(key.to_string()).or_insert(Triple::default()).r#type = value;
                        }
                        [key, "options", "format"] => {
                            let value = map.next_value::<String>()?;
                            triples.entry(key.to_string()).or_insert(Triple::default()).options.format = Some(value);
                        }
                        [key, "options", "unit"] => {
                            let value = map.next_value::<String>()?;
                            triples.entry(key.to_string()).or_insert(Triple::default()).options.unit = Some(value);
                        }
                        [key, "options", "language"] => {
                            let value = map.next_value::<String>()?;
                            triples.entry(key.to_string()).or_insert(Triple::default()).options.language = Some(value);
                        }
                        _ => return Err(serde::de::Error::custom(format!("Invalid key: {}", key))),
                    }
                }

                Ok(Triples(triples))
            }
        }

        deserializer.deserialize_map(TriplesVisitor)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triple {
    pub value: String,
    pub r#type: ValueType,
    pub options: Options,
}

impl Serialize for Triple {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("", &self.value)?;
        map.serialize_entry(".type", &self.r#type)?;
        if let Some(ref format) = self.options.format {
            map.serialize_entry(".options.format", format)?;
        }
        if let Some(ref unit) = self.options.unit {
            map.serialize_entry(".options.unit", unit)?;
        }
        if let Some(ref language) = self.options.language {
            map.serialize_entry(".options.language", language)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Triple {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TripleHelper {
            #[serde(rename = "")]
            value: String,
            #[serde(rename = ".type")]
            r#type: ValueType,
            #[serde(rename = ".options.format")]
            format: Option<String>,
            #[serde(rename = ".options.unit")]
            unit: Option<String>,
            #[serde(rename = ".options.language")]
            language: Option<String>,
        }

        let helper = TripleHelper::deserialize(deserializer)?;
        Ok(Triple {
            value: helper.value,
            r#type: helper.r#type,
            options: Options {
                format: helper.format,
                unit: helper.unit,
                language: helper.language,
            },
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Options {
    pub format: Option<String>,
    pub unit: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
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

impl From<pb::grc20::ValueType> for Option<ValueType> {
    fn from(value: pb::grc20::ValueType) -> Self {
        match value {
            pb::grc20::ValueType::Text => Some(ValueType::Text),
            pb::grc20::ValueType::Number => Some(ValueType::Number),
            pb::grc20::ValueType::Checkbox => Some(ValueType::Checkbox),
            pb::grc20::ValueType::Url => Some(ValueType::Url),
            pb::grc20::ValueType::Time => Some(ValueType::Time),
            pb::grc20::ValueType::Point => Some(ValueType::Point),
            pb::grc20::ValueType::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_with::with_prefix;
    use super::*;

    #[test]
    pub fn test_serialize_triple() {
        with_prefix!(foo_prefix "foo");
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            #[serde(flatten, with = "foo_prefix")]
            foo: Triple,
        }

        let value = Foo {
            foo: Triple {
                value: "Hello, World!".to_string(),
                r#type: ValueType::Text,
                options: Options {
                    format: Some("text".to_string()),
                    unit: Some("unit".to_string()),
                    ..Default::default()
                },
            },
        };

        let serialized = serde_json::to_value(&value).expect("Failed to serialize Value");

        assert_eq!(
            serialized,
            serde_json::json!({
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "foo.options.unit": "unit",
            })
        );

        let deserialized: Foo = serde_json::from_value(serialized).expect("Failed to deserialize Value");

        assert_eq!(deserialized, value);
    }

    #[test]
    pub fn test_serialize_triple_multiple_fields() {
        with_prefix!(foo_prefix "foo");
        with_prefix!(bar_prefix "bar");
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            #[serde(flatten, with = "foo_prefix")]
            foo: Triple,

            #[serde(flatten, with = "bar_prefix")]
            bar: Triple,

            other_field: String,
        }

        let value = Foo {
            foo: Triple {
                value: "Hello, World!".to_string(),
                r#type: ValueType::Text,
                options: Options {
                    format: Some("text".to_string()),
                    ..Default::default()
                },
            },
            bar: Triple {
                value: "123".to_string(),
                r#type: ValueType::Number,
                options: Options {
                    unit: Some("int".to_string()),
                    ..Default::default()
                },
            },
            other_field: "other".to_string(),
        };

        let serialized = serde_json::to_value(&value).expect("Failed to serialize Value");

        assert_eq!(
            serialized,
            serde_json::json!({
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "bar": "123",
                "bar.type": "NUMBER",
                "bar.options.unit": "int",
                "other_field": "other",
            })
        );

        let deserialized: Foo = serde_json::from_value(serialized).expect("Failed to deserialize Value");

        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_deserialize_triples() {
        let triples = Triples(HashMap::from([
            (
                "foo".to_string(),
                Triple {
                    value: "Hello, World!".to_string(),
                    r#type: ValueType::Text,
                    options: Options {
                        format: Some("text".to_string()),
                        ..Default::default()
                    },
                },
            ),
            (
                "bar".to_string(),
                Triple {
                    value: "123".to_string(),
                    r#type: ValueType::Number,
                    options: Options {
                        unit: Some("int".to_string()),
                        ..Default::default()
                    },
                },
            ),
        ]));

        let serialized = serde_json::to_value(&triples).expect("Failed to serialize Value");

        assert_eq!(
            serialized,
            serde_json::json!({
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "bar": "123",
                "bar.type": "NUMBER",
                "bar.options.unit": "int",
            })
        );

        let deserialized: Triples = serde_json::from_value(serialized).expect("Failed to deserialize Value");

        assert_eq!(deserialized, triples);
    }
}