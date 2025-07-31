use uuid::Uuid;

use super::query_builder::{MatchQuery, QueryBuilder, Subquery};

#[derive(Clone, Debug)]
pub struct FieldOrderBy {
    pub(crate) property: Uuid,
    pub(crate) order_direction: OrderDirection,
}

pub fn asc(property: impl Into<Uuid>) -> FieldOrderBy {
    FieldOrderBy {
        property: property.into(),
        order_direction: OrderDirection::Asc,
    }
}

pub fn desc(property: impl Into<Uuid>) -> FieldOrderBy {
    FieldOrderBy {
        property: property.into(),
        order_direction: OrderDirection::Desc,
    }
}

impl FieldOrderBy {
    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> impl Subquery {
        let node_var = node_var.into();
        let mut query = QueryBuilder::default()
            .subquery(MatchQuery::new(format!(
                r#"({node_var}) -[:PROPERTIES]-> ({node_var}_order_by:Properties)"#
            )))
            .params("property", self.property.to_string());

        match self.order_direction {
            OrderDirection::Asc => {
                query = query.subquery(format!("ORDER BY {node_var}_order_by[$property]"));
            }
            OrderDirection::Desc => {
                query = query.subquery(format!("ORDER BY {node_var}_order_by[$property] DESC"));
            }
        }

        query
    }
}

#[derive(Clone, Debug, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}
