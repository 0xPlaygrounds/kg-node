use futures::TryStreamExt;
use grc20_core::{
    mapping::{aggregation::SpaceRanking, EntityNode, QueryStream, RelationEdge},
    system_ids,
};
use grc20_sdk::models::property;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use crate::context::KnowledgeGraph;

use super::{AttributeFilter, Entity, EntityRelationFilter, EntityVersion, Relation, Triple};

#[derive(Debug)]
pub struct Property {
    entity: Entity,
}

impl Property {
    pub fn new(
        node: EntityNode,
        space_id: String,
        space_version: Option<String>,
        strict: bool,
    ) -> Self {
        Self {
            entity: Entity::new(node, space_id, space_version, strict),
        }
    }

    pub fn with_hierarchy(
        node: EntityNode,
        space_id: String,
        parent_spaces: Vec<SpaceRanking>,
        subspaces: Vec<SpaceRanking>,
        space_version: Option<String>,
        strict: bool,
    ) -> Self {
        Self {
            entity: Entity::with_hierarchy(
                node,
                space_id,
                parent_spaces,
                subspaces,
                space_version,
                strict,
            ),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Property {
    /// Entity ID
    fn id(&self) -> &str {
        self.entity.id()
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        self.entity.space_id()
    }

    fn created_at(&self) -> String {
        self.entity.created_at()
    }

    fn created_at_block(&self) -> &str {
        self.entity.created_at_block()
    }

    fn updated_at(&self) -> String {
        self.entity.updated_at()
    }

    fn updated_at_block(&self) -> &str {
        self.entity.updated_at_block()
    }

    /// Entity name (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = true)] _strict: bool,
    ) -> FieldResult<Option<String>> {
        self.entity.name(executor).await
    }

    /// Entity description (if available)
    async fn description<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        self.entity.description(executor).await
    }

    /// Entity cover (if available)
    async fn cover<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        self.entity.cover(executor).await
    }

    /// Entity blocks (if available)
    async fn blocks<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        self.entity.blocks(executor).await
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        self.entity.types(executor).await
    }

    // TODO: Add entity attributes filtering
    /// Attributes of the entity
    async fn attributes<S: ScalarValue>(
        &self,
        executor: &'_ Executor<'_, '_, KnowledgeGraph, S>,
        filter: Option<AttributeFilter>,
    ) -> FieldResult<Vec<Triple>> {
        self.entity.attributes(executor, filter).await
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        r#where: Option<EntityRelationFilter>,
    ) -> FieldResult<Vec<Relation>> {
        self.entity.relations(executor, r#where).await
    }

    // TODO: Add version filtering (e.g.: time range, edit author)
    /// Versions of the entity, ordered chronologically
    async fn versions<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<EntityVersion>> {
        self.entity.versions(executor).await
    }

    /// Value type of the property
    async fn value_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<Entity>> {
        // let value_type = self
        //     .entity
        //     .node
        //     .get_outbound_relations(
        //         &executor.context().neo4j,
        //         self.space_id(),
        //         self.entity.space_version.clone(),
        //     )
        //     .relation_type(prop_filter::value(system_ids::VALUE_TYPE_ATTRIBUTE))
        //     .limit(1)
        //     .send()
        //     .await?;

        let value_type = property::get_outbound_relations::<RelationEdge<EntityNode>>(
            &executor.context().neo4j,
            system_ids::VALUE_TYPE_ATTRIBUTE,
            self.entity.id(),
            self.space_id(),
            self.entity.space_version.clone(),
            Some(1),
            None,
            self.entity.strict,
        )
        .await?
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await?;

        Ok(value_type.first().map(|value_type| {
            Entity::with_hierarchy(
                value_type.to.clone(),
                self.space_id().to_string(),
                self.entity.parent_spaces.clone(),
                self.entity.subspaces.clone(),
                self.entity.space_version.clone(),
                self.entity.strict,
            )
        }))
    }

    /// Value type of the property
    async fn relation_value_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<Entity>> {
        // let rel_value_type = self
        //     .entity
        //     .node
        //     .get_outbound_relations(
        //         &executor.context().neo4j,
        //         self.space_id(),
        //         self.entity.space_version.clone(),
        //     )
        //     .relation_type(prop_filter::value(system_ids::RELATION_VALUE_RELATIONSHIP_TYPE))
        //     .limit(1)
        //     .send()
        //     .await?;

        let rel_value_type = property::get_outbound_relations::<RelationEdge<EntityNode>>(
            &executor.context().neo4j,
            system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
            self.entity.id(),
            self.space_id(),
            self.entity.space_version.clone(),
            Some(1),
            None,
            self.entity.strict,
        )
        .await?
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await?;

        Ok(rel_value_type.first().map(|rel_value_type| {
            Entity::with_hierarchy(
                rel_value_type.to.clone(),
                self.space_id().to_string(),
                self.entity.parent_spaces.clone(),
                self.entity.subspaces.clone(),
                self.entity.space_version.clone(),
                self.entity.strict,
            )
        }))
    }
}
