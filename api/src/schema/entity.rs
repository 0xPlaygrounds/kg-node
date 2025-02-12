use chrono::{DateTime, Utc};
use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{mapping, system_ids};

use crate::{
    context::KnowledgeGraph,
    schema::{Relation, Triple},
};

use super::{AttributeFilter, EntityRelationFilter, Options};

#[derive(Debug)]
pub struct Entity {
    pub(crate) id: String,
    pub(crate) _types: Vec<String>,
    pub(crate) space_id: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) created_at_block: String,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) updated_at_block: String,
    pub(crate) attributes: Vec<Triple>,
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Entity object
impl Entity {
    /// Entity ID
    fn id(&self) -> &str {
        &self.id
    }

    /// Entity name (if available)
    fn name(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::NAME_ATTRIBUTE)
            .map(|triple| triple.value.as_str())
    }

    /// Entity description (if available)
    fn description(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::DESCRIPTION_ATTRIBUTE)
            .map(|triple| triple.value.as_str())
    }

    /// Entity name (if available)
    fn cover(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::COVER_ATTRIBUTE)
            .map(|triple| triple.value.as_str())
    }

    /// Entity name (if available)
    async fn blocks<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        mapping::Entity::<mapping::Triples>::find_blocks(
            &executor.context().0,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        &self.space_id
    }

    fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    fn created_at_block(&self) -> &str {
        &self.created_at_block
    }

    fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    fn updated_at_block(&self) -> &str {
        &self.updated_at_block
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        mapping::Entity::<mapping::Triples>::find_types(
            &executor.context().0,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }

    /// Attributes of the entity
    fn attributes(&self, filter: Option<AttributeFilter>) -> Vec<&Triple> {
        match filter {
            Some(AttributeFilter {
                value_type: Some(value_type),
            }) => self
                .attributes
                .iter()
                .filter(|triple| triple.value_type == value_type)
                .collect::<Vec<_>>(),
            _ => self.attributes.iter().collect::<Vec<_>>(),
        }
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        r#where: Option<EntityRelationFilter>,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        let mut base_query = mapping::relation_queries::FindMany::new("r")
            .from(|from_query| from_query.id(self.id()))
            .space_id(&self.space_id);

        if let Some(filter) = r#where {
            if let Some(id) = filter.id {
                base_query = base_query.id(&id);
            }

            if let Some(to_id) = filter.to_id {
                base_query = base_query.to(|to_query| to_query.id(&to_id));
            }

            if let Some(relation_type) = filter.relation_type {
                base_query = base_query.relation_type(&relation_type);
            }
        }

        mapping::Relation::<mapping::Triples>::find_many(&executor.context().0, Some(base_query))
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
    }
}

impl From<mapping::Entity<mapping::Triples>> for Entity {
    fn from(entity: mapping::Entity<mapping::Triples>) -> Self {
        Self {
            id: entity.attributes.id,
            _types: entity.types,
            space_id: entity.attributes.system_properties.space_id.clone(),
            created_at: entity.attributes.system_properties.created_at,
            created_at_block: entity.attributes.system_properties.created_at_block,
            updated_at: entity.attributes.system_properties.updated_at,
            updated_at_block: entity.attributes.system_properties.updated_at_block,
            attributes: entity
                .attributes
                .attributes
                .into_iter()
                .map(|(key, triple)| Triple {
                    space_id: entity.attributes.system_properties.space_id.clone(),
                    attribute: key,
                    value: triple.value,
                    value_type: triple.value_type.into(),
                    options: Options {
                        format: triple.options.format,
                        unit: triple.options.unit,
                        language: triple.options.language,
                    },
                })
                .collect(),
        }
    }
}
