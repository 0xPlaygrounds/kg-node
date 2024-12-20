use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SystemProperties {
    pub space_id: String,
    #[serde(rename = "82nP7aFmHJLbaPFszj2nbx")] // CREATED_AT_TIMESTAMP
    pub created_at: DateTime<Utc>,
    #[serde(rename = "59HTYnd2e4gBx2aA98JfNx")] // CREATED_AT_BLOCK
    pub created_at_block: String,
    #[serde(rename = "5Ms1pYq8v8G1RXC3wWb9ix")] // UPDATED_AT_TIMESTAMP
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "7pXCVQDV9C7ozrXkpVg8RJ")] // UPDATED_AT_BLOCK
    pub updated_at_block: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Attributes<T> {
    pub id: String,

    // System properties
    #[serde(flatten)]
    pub system_properties: SystemProperties,

    // Actual node data
    #[serde(flatten)]
    pub attributes: T,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        mapping::triple::{Options, Triple, Triples, ValueType},
        models::BlockMetadata,
    };
    use serde_with::with_prefix;

    #[test]
    fn test_attributes_struct() {
        with_prefix!(foo_prefix "foo");
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            #[serde(flatten, with = "foo_prefix")]
            foo: Triple,
        }

        let block = BlockMetadata::default();

        let attributes = Attributes {
            id: "id".to_string(),
            system_properties: SystemProperties {
                space_id: "space_id".to_string(),
                created_at: block.timestamp,
                created_at_block: block.block_number.to_string(),
                updated_at: block.timestamp,
                updated_at_block: block.block_number.to_string(),
            },
            attributes: Foo {
                foo: Triple {
                    value: "Hello, World!".to_string(),
                    value_type: ValueType::Text,
                    options: Options {
                        format: Some("text".to_string()),
                        unit: Some("unit".to_string()),
                        ..Default::default()
                    },
                },
            },
        };

        let serialized = serde_json::to_value(&attributes).unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "id": "id",
                "space_id": "space_id",
                "created_at": "1970-01-01T00:00:00+00:00",
                "created_at_block": "0",
                "updated_at": "1970-01-01T00:00:00+00:00",
                "updated_at_block": "0",
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "foo.options.unit": "unit",
            })
        );

        let deserialized: Attributes<Foo> = serde_json::from_value(serialized).unwrap();

        assert_eq!(attributes, deserialized);
    }

    #[test]
    fn test_attributes_multiple_fields() {
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

        let block = BlockMetadata::default();

        let attributes = Attributes {
            id: "id".to_string(),
            system_properties: SystemProperties {
                space_id: "space_id".to_string(),
                created_at: block.timestamp,
                created_at_block: block.block_number.to_string(),
                updated_at: block.timestamp,
                updated_at_block: block.block_number.to_string(),
            },
            attributes: Foo {
                foo: Triple {
                    value: "Hello, World!".to_string(),
                    value_type: ValueType::Text,
                    options: Options {
                        format: Some("text".to_string()),
                        ..Default::default()
                    },
                },
                bar: Triple {
                    value: "123".to_string(),
                    value_type: ValueType::Number,
                    options: Options {
                        unit: Some("int".to_string()),
                        ..Default::default()
                    },
                },
                other_field: "other".to_string(),
            },
        };

        let serialized = serde_json::to_value(&attributes).unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "id": "id",
                "space_id": "space_id",
                "space_id": "space_id",
                "created_at": "1970-01-01T00:00:00+00:00",
                "created_at_block": "0",
                "updated_at": "1970-01-01T00:00:00+00:00",
                "updated_at_block": "0",
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "bar": "123",
                "bar.type": "NUMBER",
                "bar.options.unit": "int",
                "other_field": "other",
            })
        );

        let deserialized: Attributes<Foo> = serde_json::from_value(serialized).unwrap();

        assert_eq!(attributes, deserialized);
    }

    #[test]
    fn test_attribtes_triples() {
        let block = BlockMetadata::default();

        let attributes = Attributes {
            id: "id".to_string(),
            system_properties: SystemProperties {
                space_id: "space_id".to_string(),
                created_at: block.timestamp,
                created_at_block: block.block_number.to_string(),
                updated_at: block.timestamp,
                updated_at_block: block.block_number.to_string(),
            },
            attributes: Triples(HashMap::from([
                (
                    "foo".to_string(),
                    Triple {
                        value: "Hello, World!".to_string(),
                        value_type: ValueType::Text,
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
                        value_type: ValueType::Number,
                        options: Options {
                            unit: Some("int".to_string()),
                            ..Default::default()
                        },
                    },
                ),
            ])),
        };

        let serialized = serde_json::to_value(&attributes).expect("Failed to serialize Value");

        assert_eq!(
            serialized,
            serde_json::json!({
                "id": "id",
                "space_id": "space_id",
                "space_id": "space_id",
                "created_at": "1970-01-01T00:00:00+00:00",
                "created_at_block": "0",
                "updated_at": "1970-01-01T00:00:00+00:00",
                "updated_at_block": "0",
                "foo": "Hello, World!",
                "foo.type": "TEXT",
                "foo.options.format": "text",
                "bar": "123",
                "bar.type": "NUMBER",
                "bar.options.unit": "int",
            })
        );

        let deserialized: Attributes<Triples> =
            serde_json::from_value(serialized).expect("Failed to deserialize Value");

        assert_eq!(deserialized, attributes);
    }
}
