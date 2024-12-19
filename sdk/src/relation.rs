use std::collections::HashMap;

use crate::{
    conversion::{FromTriples, ToTriples},
    graph_uri::{GraphUri, InvalidGraphUri},
    ids::{create_geo_id, Grc20Id},
    pb::grc20,
    system_ids,
};

pub const INITIAL_COLLECTION_ITEM_INDEX_VALUE: &str = "a0";

/// A relation between two entities.
pub struct Relation {
    id: Grc20Id,
    from: Grc20Id,
    to: Grc20Id,
    relationship_types: Vec<Grc20Id>,
    index: String,
    other_attributes: HashMap<Grc20Id, (grc20::ValueType, String)>,
}

impl ToTriples for Relation {
    fn to_triples(&self) -> impl Iterator<Item = grc20::Triple> {
        let base_triples = vec![
            // Type of Collection Item
            grc20::Triple {
                entity: self.id.clone().into(),
                attribute: system_ids::TYPES.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Url as i32,
                    value: GraphUri::from_id_str(system_ids::RELATION_TYPE).to_string(),
                }),
            },
            // Entity value for the collection itself
            grc20::Triple {
                entity: self.id.clone().into(),
                attribute: system_ids::RELATION_FROM_ATTRIBUTE.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Url as i32,
                    value: GraphUri::from_id(self.from.clone()).to_string(),
                }),
            },
            // Entity value for the entity referenced by this collection item
            grc20::Triple {
                entity: self.id.clone().into(),
                attribute: system_ids::RELATION_TO_ATTRIBUTE.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Url as i32,
                    value: GraphUri::from_id(self.to.clone()).to_string(),
                }),
            },
            grc20::Triple {
                entity: self.id.clone().into(),
                attribute: system_ids::RELATION_INDEX.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Text as i32,
                    value: self.index.clone(),
                }),
            },
        ];

        base_triples
            .into_iter()
            // Add relation types
            .chain(
                self.relationship_types
                    .iter()
                    .map(|relationship_type| grc20::Triple {
                        entity: self.id.clone().into(),
                        attribute: system_ids::RELATION_TYPE_ATTRIBUTE.to_string(),
                        value: Some(grc20::Value {
                            r#type: grc20::ValueType::Url as i32,
                            value: GraphUri::from_id(relationship_type.clone()).to_string(),
                        }),
                    }),
            )
            // Add other attributes
            .chain(
                self.other_attributes
                    .iter()
                    .map(|(attribute, (r#type, value))| grc20::Triple {
                        entity: self.id.clone().into(),
                        attribute: attribute.into(),
                        value: Some(grc20::Value {
                            r#type: *r#type as i32,
                            value: value.into(),
                        }),
                    }),
            )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RelationConversionError {
    #[error("Error initializing relation: {0}")]
    MissingFieldError(#[from] RelationBuilderError),
    #[error("Type mismatch: Expected {0}, got {1}")]
    TypeMismatch(String, String),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] InvalidGraphUri),
    #[error("Invalid type: {0}")]
    InvalidType(#[from] prost::UnknownEnumValue),
}

impl FromTriples for Relation {
    type Error = RelationConversionError;

    fn from_triples(
        id: Grc20Id,
        triples: impl IntoIterator<Item = grc20::Triple>,
    ) -> Result<Self, Self::Error> {
        let relation = triples
            .into_iter()
            .try_fold::<_, _, Result<RelationBuilder, Self::Error>>(
                RelationBuilder::default(),
                |builder, triple| {
                    match triple {
                        // "Consume" the type attribute if it's a relation
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } if attribute == system_ids::TYPES
                            && r#type == grc20::ValueType::Url as i32
                            && value
                                == GraphUri::from_id_str(system_ids::RELATION_TYPE).to_string() =>
                        {
                            Ok(builder)
                        }

                        // Set the FROM_ENTITY attribute
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } if attribute == system_ids::RELATION_FROM_ATTRIBUTE
                            && r#type == grc20::ValueType::Url as i32 =>
                        {
                            Ok(GraphUri::from_uri(&value)
                                .map(|uri| builder.from_entity(uri.to_id()))?)
                        }

                        // Set the TO_ENTITY attribute
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } if attribute == system_ids::RELATION_TO_ATTRIBUTE
                            && r#type == grc20::ValueType::Url as i32 =>
                        {
                            Ok(GraphUri::from_uri(&value)
                                .map(|uri| builder.to_entity(uri.to_id()))?)
                        }

                        // Set the RELATION_TYPE attribute
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } if attribute == system_ids::RELATION_TYPE_ATTRIBUTE
                            && r#type == grc20::ValueType::Url as i32 =>
                        {
                            Ok(GraphUri::from_uri(&value)
                                .map(|uri| builder.relation_type(uri.to_id()))?)
                        }

                        // Set the RELATION_INDEX attribute
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } if attribute == system_ids::RELATION_INDEX
                            && r#type == grc20::ValueType::Text as i32 =>
                        {
                            Ok(builder.index(value))
                        }

                        // Set other attributes
                        grc20::Triple {
                            attribute,
                            value: Some(grc20::Value { r#type, value }),
                            ..
                        } => Ok(builder.other_attribute(
                            Grc20Id(attribute),
                            grc20::ValueType::try_from(r#type)?,
                            value,
                        )),

                        // Ignore triples with no value
                        grc20::Triple { value: None, .. } => Ok(builder),
                    }
                },
            )?
            .build_with_id(id)?;

        Ok(relation)
    }
}

// See https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/collections/create-relation.ts
pub fn create_relationship(
    from_id: &str,
    to_id: &str,
    relationship_type_id: &str,
    position: Option<&str>,
) -> impl Iterator<Item = grc20::Triple> {
    let new_entity_id = create_geo_id();

    vec![
        // Set the type of the new entity to RELATION_TYPE
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::TYPES.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Url as i32,
                value: GraphUri::from_id_str(system_ids::RELATION_TYPE).to_string(),
            }),
        },
        // Set the FROM_ENTITY attribute
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_FROM_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Url as i32,
                value: GraphUri::from_id_str(from_id).to_string(),
            }),
        },
        // Set the TO_ENTITY attribute
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_TO_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Url as i32,
                value: GraphUri::from_id_str(to_id).to_string(),
            }),
        },
        // Set the RELATION_INDEX attribute
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_INDEX.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Text as i32,
                value: position
                    .unwrap_or(INITIAL_COLLECTION_ITEM_INDEX_VALUE)
                    .to_string(),
            }),
        },
        // Set the RELATION_TYPE attribute
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_TYPE_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Url as i32,
                value: GraphUri::from_id_str(relationship_type_id).to_string(),
            }),
        },
    ]
    .into_iter()
}

#[derive(Debug, thiserror::Error)]
#[error("Error building relation: {0}")]
pub struct RelationBuilderError(String);

#[derive(Default)]
pub struct RelationBuilder {
    from_entity_id: Option<Grc20Id>,
    to_entity_id: Option<Grc20Id>,
    relation_types_id: Vec<Grc20Id>,
    index: Option<String>,
    other_attributes: HashMap<Grc20Id, (grc20::ValueType, String)>,
}

impl RelationBuilder {
    pub fn new(from: Grc20Id, to: Grc20Id) -> Self {
        Self {
            from_entity_id: Some(from),
            to_entity_id: Some(to),
            relation_types_id: Vec::new(),
            index: None,
            other_attributes: HashMap::new(),
        }
    }

    pub fn from_entity(mut self, from_entity_id: Grc20Id) -> Self {
        self.from_entity_id = Some(from_entity_id);
        self
    }

    pub fn to_entity(mut self, to_entity_id: Grc20Id) -> Self {
        self.to_entity_id = Some(to_entity_id);
        self
    }

    pub fn relation_type(mut self, relation_type_id: Grc20Id) -> Self {
        self.relation_types_id.push(relation_type_id);
        self
    }

    pub fn index(mut self, index: String) -> Self {
        self.index = Some(index);
        self
    }

    pub fn other_attribute(
        mut self,
        attribute_id: Grc20Id,
        r#type: grc20::ValueType,
        value: String,
    ) -> Self {
        self.other_attributes.insert(attribute_id, (r#type, value));
        self
    }

    pub fn build_with_id(self, id: Grc20Id) -> Result<Relation, RelationBuilderError> {
        Ok(Relation {
            id,
            from: self.from_entity_id.ok_or(RelationBuilderError(
                "Missing `from_entity` attribute".to_string(),
            ))?,
            to: self.to_entity_id.ok_or(RelationBuilderError(
                "Missing `to_entity` attribute".to_string(),
            ))?,
            relationship_types: self.relation_types_id,
            index: self
                .index
                .unwrap_or_else(|| INITIAL_COLLECTION_ITEM_INDEX_VALUE.to_string()),
            other_attributes: self.other_attributes,
        })
    }

    pub fn build(self) -> Result<Relation, RelationBuilderError> {
        self.build_with_id(Grc20Id::new())
    }
}
