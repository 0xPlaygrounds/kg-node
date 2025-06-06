use crate::mapping::{
    query_utils::{
        query_builder::{MatchQuery, QueryBuilder, Subquery},
        VersionFilter,
    },
    EntityFilter, PropFilter,
};

#[derive(Clone, Debug, Default)]
pub struct RelationFilter {
    pub(crate) id: Option<PropFilter<String>>,
    pub(crate) relation_type: Option<EntityFilter>,
    pub(crate) from_: Option<EntityFilter>,
    pub(crate) to_: Option<EntityFilter>,
}

impl RelationFilter {
    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn relation_type(mut self, relation_type: EntityFilter) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn from_(mut self, from_: EntityFilter) -> Self {
        self.from_ = Some(from_);
        self
    }

    pub fn to_(mut self, to_: EntityFilter) -> Self {
        self.to_ = Some(to_);
        self
    }

    pub fn subquery(&self, edge_var: &str, from: &str, to: &str) -> QueryBuilder {
        QueryBuilder::default()
            .subquery_opt(
                self.from_
                    .as_ref()
                    .map(|from_filter| from_filter.subquery(from)),
            )
            .subquery_opt(self.to_.as_ref().map(|to_filter| to_filter.subquery(to)))
            .subquery(MatchQuery::new(format!(
                "(rt:Entity {{id: {edge_var}.relation_type}})"
            )))
            .subquery_opt(self.relation_type.as_ref().map(|rt| rt.subquery("rt")))
    }
}

pub struct MatchOneRelation<'a> {
    id: String,
    space_id: String,
    space_version: &'a VersionFilter,
}

impl<'a> MatchOneRelation<'a> {
    pub fn new(id: String, space_id: impl Into<String>, space_version: &'a VersionFilter) -> Self {
        Self {
            id,
            space_id: space_id.into(),
            space_version,
        }
    }

    pub fn chain(
        self,
        from_node_var: impl Into<String>,
        to_node_var: impl Into<String>,
        edge_var: impl Into<String>,
        next: impl Subquery,
    ) -> QueryBuilder {
        let from_node_var = from_node_var.into();
        let to_node_var = to_node_var.into();
        let edge_var = edge_var.into();

        QueryBuilder::default()
            .subquery(
                MatchQuery::new(format!("({from_node_var}:Entity)-[{edge_var}:RELATION {{id: $id, space_id: $space_id}}]->({to_node_var}:Entity)"))
                    .r#where(self.space_version.subquery(&edge_var))
            )
            .limit(1)
            .subquery(format!("ORDER BY {edge_var}.index"))
            .params("id", self.id)
            .params("space_id", self.space_id)
            .with(vec![from_node_var, edge_var.to_string(), to_node_var], next)
    }
}
