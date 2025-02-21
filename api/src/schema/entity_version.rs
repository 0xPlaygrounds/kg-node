use futures::TryStreamExt;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};
use sdk::mapping::{
    query_utils::{prop_filter, QueryStream},
    triple,
};

use crate::context::KnowledgeGraph;

use super::{AttributeFilter, Triple};

pub struct EntityVersion {
    pub id: String,
    entity_id: String,
    index: String,
    space_id: String,
}

impl EntityVersion {
    pub fn new(id: String, entity_id: String, index: String, space_id: String) -> Self {
        Self {
            entity_id,
            id,
            index,
            space_id,
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl EntityVersion {
    fn id(&self) -> &str {
        &self.id
    }

    // fn index(&self) -> &str {
    //     &self.index
    // }

    // TODO: Add entity attributes filtering
    /// Attributes of the entity
    async fn attributes<S: ScalarValue>(
        &self,
        _filter: Option<AttributeFilter>,
        executor: &'_ Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Triple>> {
        let query = triple::find_many(&executor.context().0)
            .entity_id(prop_filter::value(&self.entity_id))
            .space_version(&self.index);

        Ok(query
            .send()
            .await?
            .map_ok(|triple| Triple::new(triple, self.space_id.clone(), Some(self.index.clone())))
            .try_collect::<Vec<_>>()
            .await?)
    }
}
