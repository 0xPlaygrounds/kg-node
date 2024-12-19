use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Attributes<T> {
    pub id: String,
    pub space_id: String,
    #[serde(flatten)]
    pub attributes: T,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::mapping::triple::{Options, Triple, Triples, ValueType};
    use serde_with::with_prefix;

    #[test]
    fn test_attributes_struct() {
        with_prefix!(foo_prefix "foo");
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            #[serde(flatten, with = "foo_prefix")]
            foo: Triple,
        }

        let attributes = Attributes {
            id: "id".to_string(),
            space_id: "space_id".to_string(),
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

        let attributes = Attributes {
            id: "id".to_string(),
            space_id: "space_id".to_string(),
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
        let attributes = Attributes {
            id: "id".to_string(),
            space_id: "space_id".to_string(),
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
