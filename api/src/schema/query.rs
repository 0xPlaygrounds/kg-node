use futures::TryStreamExt;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use grc20_core::{
    indexer_ids,
    mapping::{
        self, entity_node, prop_filter,
        query_utils::{Query, QueryStream},
        relation_node,
    },
};
use grc20_sdk::models::{account, property, space};

use crate::{
    context::KnowledgeGraph,
    schema::{Account, AccountFilter, Entity, Relation, RelationFilter, Space, SpaceFilter},
};

use super::{entity_order_by::OrderDirection, EntityFilter, Triple};

#[derive(Clone)]
pub struct RootQuery;

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl RootQuery {
    /// Returns a single space by ID
    async fn space<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        version: Option<String>,
    ) -> FieldResult<Option<Space>> {
        Ok(Space::load(&executor.context().0, id, version).await?)
    }

    /// Returns multiple spaces according to the provided filter
    #[allow(clippy::too_many_arguments)]
    async fn spaces<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        r#where: Option<SpaceFilter>,
        version: Option<String>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<Space>> {
        let mut query = space::find_many(&executor.context().0, indexer_ids::INDEXER_SPACE_ID);

        // Apply filters if provided
        if let Some(where_) = &r#where {
            // Network filter
            if let Some(network_filter) = where_.network_filter() {
                query = query.network(network_filter);
            }

            // Governance type filter
            if let Some(governance_type_filter) = where_.governance_type_filter() {
                query = query.governance_type(governance_type_filter);
            }

            // DAO contract address filter
            if let Some(dao_contract_address_filter) = where_.dao_contract_address_filter() {
                query = query.dao_contract_address(dao_contract_address_filter);
            }

            // Space plugin address filter
            if let Some(space_plugin_address_filter) = where_.space_plugin_address_filter() {
                query = query.space_plugin_address(space_plugin_address_filter);
            }

            // Voting plugin address filter
            if let Some(voting_plugin_address_filter) = where_.voting_plugin_address_filter() {
                query = query.voting_plugin_address(voting_plugin_address_filter);
            }

            // Member access plugin filter
            if let Some(member_access_plugin_filter) = where_.member_access_plugin_filter() {
                query = query.member_access_plugin(member_access_plugin_filter);
            }

            // Personal space admin plugin filter
            if let Some(personal_space_admin_plugin_filter) =
                where_.personal_space_admin_plugin_filter()
            {
                query = query.personal_space_admin_plugin(personal_space_admin_plugin_filter);
            }
        }

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(|entity| Space::new(entity, version.clone()))
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Returns a single account by ID
    async fn account<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
    ) -> FieldResult<Option<Account>> {
        Account::load(&executor.context().0, id).await
    }

    /// Returns a single account by address
    async fn account_by_address<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        address: String,
    ) -> FieldResult<Option<Account>> {
        let query = account::find_one(
            &executor.context().0,
            &address,
            indexer_ids::INDEXER_SPACE_ID,
        );
        Ok(query.send().await?.map(Account::new))
    }

    /// Returns multiple accounts according to the provided filter
    #[allow(clippy::too_many_arguments)]
    async fn accounts<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        where_: Option<AccountFilter>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<Account>> {
        let mut query = account::find_many(&executor.context().0, indexer_ids::INDEXER_SPACE_ID);

        // Apply filters if provided
        if let Some(where_) = &where_ {
            // Address filter
            if let Some(address_filter) = where_.address_filter() {
                query = query.address(address_filter);
            }
        }

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(Account::new)
            .try_collect::<Vec<_>>()
            .await?)
    }
    /// Returns a single entity identified by its ID and space ID
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        version_id: Option<String>,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Option<Entity>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Entity::load(&executor.context().0, id, space_id, version_index, strict).await
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
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Vec<Entity>> {
        let mut query = entity_node::find_many(&executor.context().0);

        let entity_filter = if let Some(r#where) = r#where {
            mapping::EntityFilter::from(r#where).space_id(prop_filter::value(&space_id))
        } else {
            mapping::EntityFilter::default().space_id(prop_filter::value(&space_id))
        };
        query = query.with_filter(entity_filter);

        match (order_by, order_direction) {
            (Some(order_by), Some(OrderDirection::Asc) | None) => {
                query.order_by_mut(mapping::order_by::asc(order_by));
            }
            (Some(order_by), Some(OrderDirection::Desc)) => {
                query.order_by_mut(mapping::order_by::desc(order_by));
            }
            _ => {}
        }

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(|entity| Entity::new(entity, space_id.clone(), None, strict))
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
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Option<Relation>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Relation::load(&executor.context().0, id, space_id, version_index, strict).await
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
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Vec<Relation>> {
        let mut query = relation_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            query = r#where.apply_filter(query);
        }

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(|relation| Relation::new(relation, space_id.clone(), None, strict))
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Returns a single triple identified by its entity ID, attribute ID, space ID and
    /// optional version ID
    async fn triple<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        entity_id: String,
        attribute_id: String,
        space_id: String,
        version_id: Option<String>,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Option<Triple>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Ok(property::get_triple(
            &executor.context().0,
            &attribute_id,
            &entity_id,
            &space_id,
            version_index.clone(),
            strict,
        )
        .await?
        .map(|triple| Triple::new(triple, space_id, version_index)))
    }
}
