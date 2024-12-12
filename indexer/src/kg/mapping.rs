use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Relation<T> {
    // pub id: String,
    pub relation_type: String,
    pub from: String,
    pub to: String,
    #[serde(flatten)]
    pub attributes: Attributes<T>,
}

impl<T> TryFrom<neo4rs::Relation> for Relation<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = neo4rs::DeError;

    fn try_from(value: neo4rs::Relation) -> Result<Self, Self::Error> {
        let attributes = value.to()?;
        Ok(Self {
            relation_type: value.typ().to_string(),
            attributes,
            from: value.start_node_id().to_string(),
            to: value.end_node_id().to_string(),
        })
    }
}

impl<T> Relation<T> {
    pub fn new(id: &str, space_id: &str, from: &str, to: &str, relation_type: &str, data: T) -> Self {
        Self {
            // id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            relation_type: relation_type.to_string(),
            attributes: Attributes { id: id.to_string(), space_id: space_id.to_string(), attributes: data },
        }
    }

    pub fn id(&self) -> &str {
        &self.attributes.id
    }

    pub fn space_id(&self) -> &str {
        &self.attributes.space_id
    }

    pub fn attributes(&self) -> &T {
        &self.attributes.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut T {
        &mut self.attributes.attributes
    }
}

impl Relation<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, key: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut().insert(key, value.into());
        self
    }
}

/// GRC20 Node
#[derive(Debug, Deserialize, PartialEq)]
pub struct Node<T> {
    #[serde(rename = "labels", deserialize_with = "deserialize_labels")]
    pub types: Vec<String>,
    #[serde(flatten)]
    pub attributes: Attributes<T>,
}

impl<T> TryFrom<neo4rs::Node> for Node<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = neo4rs::DeError;

    fn try_from(value: neo4rs::Node) -> Result<Self, Self::Error> {
        let labels = value.labels().iter().map(|l| l.to_string()).collect();
        let attributes = value.to()?;
        Ok(Self {
            types: labels,
            attributes,
        })
    }
}

/// Neo4j node representing a GRC20 entity of type `T`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Attributes<T> {
    pub id: String,
    pub space_id: String,
    // pub space_id: String,
    #[serde(flatten)]
    pub attributes: T,
}

fn deserialize_labels<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let labels: neo4rs::Labels = serde::Deserialize::deserialize(deserializer)?;
    Ok(labels.0)
}

impl<T> Node<T> {
    pub fn new(id: &str, space_id: &str, data: T) -> Self {
        Self {
            types: Vec::new(),
            attributes: Attributes {
                id: id.to_string(),
                space_id: space_id.to_string(),
                attributes: data,
            },
        }
    }

    pub fn id(&self) -> &str {
        &self.attributes.id
    }

    pub fn space_id(&self) -> &str {
        &self.attributes.space_id
    }

    pub fn attributes(&self) -> &T {
        &self.attributes.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut T {
        &mut self.attributes.attributes
    }

    pub fn with_type(mut self, type_id: &str) -> Self {
        self.types.push(type_id.to_string());
        self
    }
}

impl Node<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, attribute_id: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut()
            .insert(attribute_id, value.into());
        self
    }
}

impl Node<DefaultAttributes> {
    pub fn name(&self) -> Option<String> {
        self.attributes().get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

pub type DefaultAttributes = HashMap<String, serde_json::Value>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Named {
    pub name: Option<String>,
}

impl Node<Named> {
    pub fn name_or_id(&self) -> String {
        self.name().unwrap_or_else(|| self.id().to_string())
    }

    pub fn name(&self) -> Option<String> {
        self.attributes().name.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    pub fn test_node_conversion() {
        let node = neo4rs::Node::new(neo4rs::BoltNode {
            id: neo4rs::BoltInteger { value: 425 },
            labels: neo4rs::BoltList {
                value: vec![neo4rs::BoltType::String(neo4rs::BoltString {
                    value: "9u4zseS3EDXG9ZvwR9RmqU".to_string(),
                })],
            },
            properties: neo4rs::BoltMap {
                value: HashMap::from([
                    (
                        neo4rs::BoltString {
                            value: "space_id".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "NBDtpHimvrkmVu7vVBXX7b".to_string(),
                        }),
                    ),
                    (
                        neo4rs::BoltString {
                            value: "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "Person Posts Page Template".to_string(),
                        }),
                    ),
                    (
                        neo4rs::BoltString {
                            value: "id".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "98wgvodwzidmVA4ryVzGX6".to_string(),
                        }),
                    ),
                ]),
            },
        });

        let node: Node<HashMap<String, serde_json::Value>> = node.try_into()
            .expect("Failed to convert neo4rs::Node to Node<HashMap<String, neo4rs::BoltType>>");
    
        assert_eq!(
            node,
            Node {
                types: vec!["9u4zseS3EDXG9ZvwR9RmqU".to_string()],
                attributes: Attributes {
                    id: "98wgvodwzidmVA4ryVzGX6".to_string(),
                    space_id: "NBDtpHimvrkmVu7vVBXX7b".to_string(),
                    attributes: HashMap::from([
                        (
                            "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
                            serde_json::Value::String("Person Posts Page Template".to_string())
                        ),
                    ])
                }
            }
        )
    }
}