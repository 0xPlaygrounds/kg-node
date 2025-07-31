use rand::distributions::DistString;
use uuid::Uuid;

use crate::{
    mapping::query_utils::{
        query_builder::{MatchQuery, QueryBuilder},
        PropertyFilter, ValueFilter,
    },
    relation::RelationDirection,
};

/// Filter used to find entities in the knowledge graph.
#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    pub(crate) id: Option<ValueFilter<Uuid>>,
    pub(crate) properties: Vec<PropertyFilter>,
    pub(crate) types: Option<TypesFilter>,
    pub(crate) relations: Option<EntityRelationFilter>,
}

impl EntityFilter {
    pub fn id(mut self, id: ValueFilter<Uuid>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn property(mut self, propertie: PropertyFilter) -> Self {
        self.properties.push(propertie);
        self
    }

    pub fn property_mut(&mut self, propertie: PropertyFilter) {
        self.properties.push(propertie);
    }

    pub fn properties(mut self, properties: impl IntoIterator<Item = PropertyFilter>) -> Self {
        self.properties.extend(properties);
        self
    }

    pub fn properties_mut(&mut self, properties: impl IntoIterator<Item = PropertyFilter>) {
        self.properties.extend(properties);
    }

    pub fn types(mut self, types: TypesFilter) -> Self {
        self.types = Some(types);
        self
    }

    pub fn types_mut(&mut self, types: TypesFilter) {
        self.types = Some(types);
    }

    pub fn relations(mut self, relations: impl Into<EntityRelationFilter>) -> Self {
        self.relations = Some(relations.into());
        self
    }

    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> QueryBuilder {
        let node_var = node_var.into();

        QueryBuilder::default()
            .subquery(
                MatchQuery::new(format!("({node_var}:$($types))"))
                    .where_opt(
                        self.id
                            .clone()
                            .map(|q| q.as_string_filter().subquery(&node_var, "id", None)),
                    )
                    .params(
                        "types",
                        self.types
                            .clone()
                            .map(|t| t.types.into_iter().map(|t| t.to_string()).collect())
                            .unwrap_or(vec!["Entity".to_string()]),
                    ),
            )
            // Apply attribute filters
            .subqueries(
                self.properties
                    .iter()
                    .map(|attribute| attribute.subquery(&node_var))
                    .collect(),
            )
            // Apply the relations filter
            .subquery_opt(
                self.relations
                    .as_ref()
                    .map(|relations| relations.subquery(&node_var)),
            )
    }
}

/// Filter used to:
/// - Filter the relations outgoing from the entity
/// - Filter an entity by its outgoing relations
#[derive(Clone, Debug, Default)]
pub struct EntityRelationFilter {
    relation_type: Option<ValueFilter<String>>,
    to_id: Option<ValueFilter<String>>,
}

impl EntityRelationFilter {
    pub fn relation_type(mut self, relation_type: impl Into<ValueFilter<String>>) -> Self {
        self.relation_type = Some(relation_type.into());
        self
    }

    pub fn to_id(mut self, to_id: impl Into<ValueFilter<String>>) -> Self {
        self.to_id = Some(to_id.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type.is_none() && self.to_id.is_none()
    }

    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> MatchQuery {
        let node_var = node_var.into();
        let random_suffix: String =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 4);
        let rel_edge_var = format!("r_{node_var}_{random_suffix}");
        let to_node_var = format!("r_{node_var}_to");

        MatchQuery::new(format!(
            "({node_var}) -[{rel_edge_var}:RELATION]-> ({to_node_var})"
        ))
        // Apply the relation_type filter to the relation (if any)
        .where_opt(
            self.relation_type
                .as_ref()
                .map(|relation_type| relation_type.subquery(&rel_edge_var, "relation_type", None)),
        )
        // Apply the to_id filter to the relation (if any)
        .where_opt(
            self.to_id
                .as_ref()
                .map(|to_id| to_id.subquery(&to_node_var, "id", None)),
        )
    }
}

/// Filter used to:
/// - Traverse to inbound or outbound relation
#[derive(Clone, Debug, Default)]
pub struct RelationTraversal {
    pub relation_type_id: Option<ValueFilter<Uuid>>,
    pub destination_id: Option<ValueFilter<Uuid>>,
    pub direction: Option<RelationDirection>,
}

impl RelationTraversal {
    pub fn relation_type_id(mut self, relation_type_id: impl Into<ValueFilter<Uuid>>) -> Self {
        self.relation_type_id = Some(relation_type_id.into());
        self
    }

    pub fn destination_id(mut self, destination_id: impl Into<ValueFilter<Uuid>>) -> Self {
        self.destination_id = Some(destination_id.into());
        self
    }

    pub fn direction(mut self, direction: RelationDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type_id.is_none() && self.destination_id.is_none()
    }
}

#[derive(Clone, Debug, Default)]
pub struct TypesFilter {
    types: Vec<Uuid>,
}

impl TypesFilter {
    pub fn r#type(mut self, r#type: impl Into<Uuid>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn types(mut self, mut types: Vec<Uuid>) -> Self {
        self.types.append(&mut types);
        self
    }
}

impl From<TypesFilter> for EntityFilter {
    fn from(types_filter: TypesFilter) -> Self {
        EntityFilter::default().types(types_filter)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        entity::utils::{EntityFilter, TypesFilter},
        mapping::query_utils::{PropertyFilter, Subquery, ValueFilter},
    };

    #[test]
    fn test_entity_filter() {
        let filter = EntityFilter::default()
            .id(ValueFilter::default().value(Uuid::new_v4()))
            .property(PropertyFilter::new(Uuid::new_v4()).value("value".to_string()))
            .types(TypesFilter::default().r#type(Uuid::new_v4()));

        println!("{}", filter.subquery("e").compile())
    }
}
