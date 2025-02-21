use futures::TryStreamExt;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use sdk::mapping::{self, entity_node, query_utils::QueryStream, relation_node};

use crate::{
    context::KnowledgeGraph,
    schema::{Entity, Relation, RelationFilter},
};

use super::{entity_order_by::OrderDirection, EntityFilter};

#[derive(Clone)]
pub struct Query;

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Query {
    /// Returns a single entity identified by its ID and space ID
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        version_id: Option<String>,
    ) -> FieldResult<Option<Entity>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Entity::load(&executor.context().0, id, space_id, version_index).await
    }

    #[allow(clippy::too_many_arguments)]
    /// Returns multiple entities according to the provided space ID and filter
    async fn entities<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        order_by: Option<String>,
        order_direction: Option<OrderDirection>,
        r#where: Option<EntityFilter>,
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<Entity>> {
        let mut query = entity_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            let filter = entity_node::EntityFilter::from(r#where).with_space_id(&space_id);

            query = query.with_filter(filter);
        }

        match (order_by, order_direction) {
            (Some(order_by), Some(OrderDirection::Asc) | None) => {
                query.order_by_mut(mapping::order_by::asc(order_by));
            }
            (Some(order_by), Some(OrderDirection::Desc)) => {
                query.order_by_mut(mapping::order_by::desc(order_by));
            }
            _ => {}
        }

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 entities at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(query
            .send()
            .await?
            .map_ok(|entity| Entity::new(entity, space_id.clone(), None))
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Returns a single relation identified by its ID and space ID
    async fn relation<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        version_id: Option<String>,
    ) -> FieldResult<Option<Relation>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Relation::load(&executor.context().0, id, space_id, version_index).await
    }

    // TODO: Add order_by and order_direction
    #[allow(clippy::too_many_arguments)]
    /// Returns multiple relations according to the provided space ID and filter
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        _order_by: Option<String>,
        _order_direction: Option<OrderDirection>,
        r#where: Option<RelationFilter>,
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<Relation>> {
        let mut query = relation_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            query = r#where.apply_filter(query);
        }

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 relations at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(query
            .send()
            .await?
            .map_ok(|relation| Relation::new(relation, space_id.clone(), None))
            .try_collect::<Vec<_>>()
            .await?)
    }
}
