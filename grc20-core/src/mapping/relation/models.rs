use neo4rs::BoltType;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{pb, system_ids};

/// Error type for converting from pb::grc20::Relation to Relation
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid UUID for {0}: {1}")]
    InvalidUuid(String, String),
    #[error("Invalid UTF-8 for position: {0}")]
    InvalidPositionUtf8(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Relation {
    pub id: Uuid,
    pub r#type: Uuid,
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub entity: Uuid,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CreateRelation {
    pub id: Uuid,
    pub r#type: Uuid,
    pub from_entity: Uuid,
    pub from_space: Option<Uuid>,
    pub from_version: Option<Uuid>,
    pub to_entity: Uuid,
    pub to_space: Option<Uuid>,
    pub to_version: Option<Uuid>,
    pub entity: Uuid,
    pub position: Option<String>,
    pub verified: Option<bool>,
}

impl CreateRelation {
    pub fn new(id: Uuid, r#type: Uuid, from_entity: Uuid, to_entity: Uuid, entity: Uuid) -> Self {
        Self {
            id,
            r#type,
            from_entity,
            from_space: None,
            from_version: None,
            to_entity,
            to_space: None,
            to_version: None,
            entity,
            position: None,
            verified: None,
        }
    }

    pub fn from_space(mut self, space: Uuid) -> Self {
        self.from_space = Some(space);
        self
    }

    pub fn from_version(mut self, version: Uuid) -> Self {
        self.from_version = Some(version);
        self
    }

    pub fn to_space(mut self, space: Uuid) -> Self {
        self.to_space = Some(space);
        self
    }

    pub fn to_version(mut self, version: Uuid) -> Self {
        self.to_version = Some(version);
        self
    }

    pub fn position(mut self, position: impl Into<String>) -> Self {
        self.position = Some(position.into());
        self
    }

    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = Some(verified);
        self
    }
}

impl From<CreateRelation> for BoltType {
    fn from(relation: CreateRelation) -> Self {
        let mut map = HashMap::new();
        map.insert(
            neo4rs::BoltString { value: "id".into() },
            relation.id.to_string().into(),
        );
        map.insert(
            neo4rs::BoltString {
                value: "type".into(),
            },
            relation.r#type.to_string().into(),
        );
        if relation.r#type == system_ids::TYPES_ATTRIBUTE {
            map.insert(
                neo4rs::BoltString {
                    value: "labels".into(),
                },
                relation.to_entity.to_string().into(),
            );
        } else {
            map.insert(
                neo4rs::BoltString {
                    value: "labels".into(),
                },
                Vec::<String>::new().into(),
            );
        }
        map.insert(
            neo4rs::BoltString {
                value: "from_entity".into(),
            },
            relation.from_entity.to_string().into(),
        );
        if let Some(from_space) = relation.from_space {
            map.insert(
                neo4rs::BoltString {
                    value: "from_space".into(),
                },
                from_space.to_string().into(),
            );
        }
        if let Some(from_version) = relation.from_version {
            map.insert(
                neo4rs::BoltString {
                    value: "from_version".into(),
                },
                from_version.to_string().into(),
            );
        }
        map.insert(
            neo4rs::BoltString {
                value: "to_entity".into(),
            },
            relation.to_entity.to_string().into(),
        );
        if let Some(to_space) = relation.to_space {
            map.insert(
                neo4rs::BoltString {
                    value: "to_space".into(),
                },
                to_space.to_string().into(),
            );
        }
        if let Some(to_version) = relation.to_version {
            map.insert(
                neo4rs::BoltString {
                    value: "to_version".into(),
                },
                to_version.to_string().into(),
            );
        }
        map.insert(
            neo4rs::BoltString {
                value: "entity".into(),
            },
            relation.entity.to_string().into(),
        );
        if let Some(position) = relation.position {
            map.insert(
                neo4rs::BoltString {
                    value: "position".into(),
                },
                position.into(),
            );
        }
        if let Some(verified) = relation.verified {
            map.insert(
                neo4rs::BoltString {
                    value: "verified".into(),
                },
                verified.into(),
            );
        }
        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl TryFrom<pb::grc20::Relation> for CreateRelation {
    type Error = ConversionError;

    fn try_from(pb_relation: pb::grc20::Relation) -> Result<Self, Self::Error> {
        let id = Uuid::from_bytes(
            pb_relation
                .id
                .try_into()
                .map_err(|e| ConversionError::InvalidUuid("id".to_string(), format!("{e:?}")))?,
        );
        let r#type = Uuid::from_bytes(
            pb_relation
                .r#type
                .try_into()
                .map_err(|e| ConversionError::InvalidUuid("type".to_string(), format!("{e:?}")))?,
        );
        let from_entity = Uuid::from_bytes(pb_relation.from_entity.try_into().map_err(|e| {
            ConversionError::InvalidUuid("from_entity".to_string(), format!("{e:?}"))
        })?);
        let from_space = match pb_relation.from_space {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("from_space".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };
        let from_version = match pb_relation.from_version {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("from_version".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };
        let to_entity = Uuid::from_bytes(pb_relation.to_entity.try_into().map_err(|e| {
            ConversionError::InvalidUuid("to_entity".to_string(), format!("{e:?}"))
        })?);
        let to_space = match pb_relation.to_space {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("to_space".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };
        let to_version = match pb_relation.to_version {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("to_version".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };
        let entity =
            Uuid::from_bytes(pb_relation.entity.try_into().map_err(|e| {
                ConversionError::InvalidUuid("entity".to_string(), format!("{e:?}"))
            })?);
        let position = match pb_relation.position {
            Some(pos) => Some(pos),
            None => None,
        };
        let verified = pb_relation.verified;

        Ok(Self {
            id,
            r#type,
            from_entity,
            from_space,
            from_version,
            to_entity,
            to_space,
            to_version,
            entity,
            position,
            verified,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UpdateRelation {
    pub id: Uuid,
    pub from_space: Option<Uuid>,
    pub from_version: Option<Uuid>,
    pub to_space: Option<Uuid>,
    pub to_version: Option<Uuid>,
    pub position: Option<String>,
    pub verified: Option<bool>,
}

impl TryFrom<pb::grc20::RelationUpdate> for UpdateRelation {
    type Error = ConversionError;

    fn try_from(pb_update: pb::grc20::RelationUpdate) -> Result<Self, Self::Error> {
        let id = Uuid::from_bytes(
            pb_update
                .id
                .try_into()
                .map_err(|e| ConversionError::InvalidUuid("id".to_string(), format!("{e:?}")))?,
        );

        let from_space = match pb_update.from_space {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("from_space".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };

        let from_version = match pb_update.from_version {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("from_version".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };

        let to_space = match pb_update.to_space {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("to_space".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };

        let to_version = match pb_update.to_version {
            Some(bytes) => Some(Uuid::from_bytes(bytes.try_into().map_err(|e| {
                ConversionError::InvalidUuid("to_version".to_string(), format!("{e:?}"))
            })?)),
            None => None,
        };

        Ok(Self {
            id,
            from_space,
            from_version,
            to_space,
            to_version,
            position: pb_update.position,
            verified: pb_update.verified,
        })
    }
}

impl From<UpdateRelation> for BoltType {
    fn from(update: UpdateRelation) -> Self {
        let mut map = HashMap::new();

        if let Some(from_space) = update.from_space {
            map.insert(
                neo4rs::BoltString {
                    value: "from_space".into(),
                },
                from_space.to_string().into(),
            );
        }

        if let Some(from_version) = update.from_version {
            map.insert(
                neo4rs::BoltString {
                    value: "from_version".into(),
                },
                from_version.to_string().into(),
            );
        }

        if let Some(to_space) = update.to_space {
            map.insert(
                neo4rs::BoltString {
                    value: "to_space".into(),
                },
                to_space.to_string().into(),
            );
        }

        if let Some(to_version) = update.to_version {
            map.insert(
                neo4rs::BoltString {
                    value: "to_version".into(),
                },
                to_version.to_string().into(),
            );
        }

        if let Some(position) = update.position {
            map.insert(
                neo4rs::BoltString {
                    value: "position".into(),
                },
                position.into(),
            );
        }

        if let Some(verified) = update.verified {
            map.insert(
                neo4rs::BoltString {
                    value: "verified".into(),
                },
                verified.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UnsetRelationFields {
    pub id: Uuid,
    pub from_space: Option<bool>,
    pub from_version: Option<bool>,
    pub to_space: Option<bool>,
    pub to_version: Option<bool>,
    pub position: Option<bool>,
    pub verified: Option<bool>,
}

impl TryFrom<pb::grc20::UnsetRelationFields> for UnsetRelationFields {
    type Error = ConversionError;

    fn try_from(pb_unset: pb::grc20::UnsetRelationFields) -> Result<Self, Self::Error> {
        let id = Uuid::from_bytes(
            pb_unset
                .id
                .try_into()
                .map_err(|e| ConversionError::InvalidUuid("id".to_string(), format!("{e:?}")))?,
        );

        Ok(Self {
            id,
            from_space: pb_unset.from_space,
            from_version: pb_unset.from_version,
            to_space: pb_unset.to_space,
            to_version: pb_unset.to_version,
            position: pb_unset.position,
            verified: pb_unset.verified,
        })
    }
}

impl From<UnsetRelationFields> for BoltType {
    fn from(unset: UnsetRelationFields) -> Self {
        let mut map = HashMap::new();

        if unset.from_space == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "from_space".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }
        if unset.from_version == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "from_version".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }
        if unset.to_space == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "to_space".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }
        if unset.to_version == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "to_version".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }
        if unset.position == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "position".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }
        if unset.verified == Some(true) {
            map.insert(
                neo4rs::BoltString {
                    value: "verified".into(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}
