use rand::distributions::DistString;

use crate::{
    mapping::{
        query_utils::{
            query_builder::{MatchQuery, NamePair, QueryBuilder, Rename, Subquery},
            RelationDirection, VersionFilter,
        },
        AttributeFilter, PropFilter,
    },
    system_ids,
};

/// Filter used to find entities in the knowledge graph.
#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    pub(crate) id: Option<PropFilter<String>>,
    pub(crate) attributes: Vec<AttributeFilter>,
    pub(crate) relations: Option<EntityRelationFilter>,
    /// traverse relation now in entity directly but eventually for modularity will be standalone to be chained
    pub(crate) traverse_relation: Option<TraverseRelation>,
    /// Used to check if the entity exists in the space (i.e.: the entity
    /// has at least one attribute in the space).
    pub(crate) space_id: Option<PropFilter<String>>,
}

impl EntityFilter {
    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.attributes.extend(attributes);
    }

    pub fn relations(mut self, relations: impl Into<EntityRelationFilter>) -> Self {
        self.relations = Some(relations.into());
        self
    }

    pub fn traverse_relation(mut self, traverse_relation: impl Into<TraverseRelation>) -> Self {
        self.traverse_relation = Some(traverse_relation.into());
        self
    }

    /// Used to check if the entity exists in the space.
    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id.clone());
        self
    }

    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> QueryBuilder {
        let node_var = node_var.into();

        QueryBuilder::default()
            // Apply attribute filters
            .subqueries(
                self.attributes
                    .iter()
                    .map(|attribute| attribute.subquery(&node_var))
                    .collect(),
            )
            // Apply the space_id filter
            .subquery_opt(self.space_id.as_ref().map(|space_id| {
                MatchQuery::new(format!("({node_var}) -[a:ATTRIBUTE]- (:Attribute)"))
                    .r#where(space_id.subquery("a", "space_id", None))
            }))
            // Apply the relations filter
            .subquery_opt(
                self.relations
                    .as_ref()
                    .map(|relations| relations.subquery(&node_var)),
            )
            // Apply relation traversal
            .subquery_opt(
                self.traverse_relation
                    .as_ref()
                    .map(|traverse| traverse.subquery(&node_var)),
            )
    }
}

/// Filter used to:
/// - Filter the relations outgoing from the entity
/// - Filter an entity by its outgoing relations
#[derive(Clone, Debug, Default)]
pub struct EntityRelationFilter {
    relation_type: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl EntityRelationFilter {
    pub fn relation_type(mut self, relation_type: impl Into<PropFilter<String>>) -> Self {
        self.relation_type = Some(relation_type.into());
        self
    }

    pub fn to_id(mut self, to_id: impl Into<PropFilter<String>>) -> Self {
        self.to_id = Some(to_id.into());
        self
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type.is_none() && self.to_id.is_none()
    }

    // /// Applies a global space_id to all sub-filters (i.e.: relation_type and to_id filters).
    // /// If a space_id is already set in a sub-filter, it will be overwritten.
    // pub fn with_space_id(mut self, space_id: PropFilter<String>) -> Self {
    //     self.relation_type = self
    //         .relation_type
    //         .map(|filter| filter.space_id(space_id.clone()));

    //     self.to_id = self.to_id.map(|filter| filter.space_id(space_id));

    //     self
    // }

    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> MatchQuery {
        let node_var = node_var.into();
        let random_suffix: String =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 4);
        let rel_edge_var = format!("r_{node_var}_{random_suffix}");
        let to_node_var = format!("r_{node_var}_to");

        MatchQuery::new(format!(
            "({node_var}) -[{rel_edge_var}:RELATION]-> ({to_node_var})"
        ))
        // Apply the version filter to the relation
        .r#where(self.version.subquery(&rel_edge_var))
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
        // Apply the space_id filter to the relation (if any)
        .where_opt(
            self.space_id
                .as_ref()
                .map(|space_id| space_id.subquery(&rel_edge_var, "space_id", None)),
        )
    }
}

/// Filter used to:
/// - Traverse to inbound or outbound relation
#[derive(Clone, Debug, Default)]
pub struct TraverseRelation {
    relation_type_id: Option<PropFilter<String>>,
    destination_id: Option<PropFilter<String>>,
    direction: RelationDirection,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl TraverseRelation {
    pub fn relation_type_id(mut self, relation_type_id: impl Into<PropFilter<String>>) -> Self {
        self.relation_type_id = Some(relation_type_id.into());
        self
    }

    pub fn destination_id(mut self, destination_id: impl Into<PropFilter<String>>) -> Self {
        self.destination_id = Some(destination_id.into());
        self
    }

    pub fn direction(mut self, direction: RelationDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type_id.is_none() && self.destination_id.is_none()
    }

    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> MatchQuery {
        let node_var_curr = node_var.into();
        let random_suffix: String =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 4);
        let rel_edge_var = format!("r_{node_var_curr}_{random_suffix}");
        let node_var_dest = format!("r_{node_var_curr}_{random_suffix}_to");

        MatchQuery::new(match self.direction {
            RelationDirection::From => {
                format!("({node_var_curr}) -[{rel_edge_var}:RELATION]-> ({node_var_dest})")
            }
            RelationDirection::To => {
                format!("({node_var_dest}) -[{rel_edge_var}:RELATION]-> ({node_var_curr})")
            }
            RelationDirection::Both => {
                format!("({node_var_curr}) -[{rel_edge_var}:RELATION]- ({node_var_dest})")
            }
        })
        // rename to change direction of relation
        .rename(Rename::new(NamePair::new(
            node_var_curr.clone(),
            node_var_dest.clone(),
        )))
        // Apply the version filter to the relation
        .r#where(self.version.subquery(&rel_edge_var))
        // Apply the relation_type filter to the relation (if any)
        .where_opt(
            self.relation_type_id
                .as_ref()
                .map(|relation_type| relation_type.subquery(&rel_edge_var, "relation_type", None)),
        )
        // Apply the from_id filter to the relation (if any)
        .where_opt(
            self.destination_id
                .as_ref()
                .map(|dest_id| dest_id.subquery(&node_var_curr, "id", None)),
        )
        // Apply the space_id filter to the relation (if any)
        .where_opt(
            self.space_id
                .as_ref()
                .map(|space_id| space_id.subquery(&rel_edge_var, "space_id", None)),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct TypesFilter {
    types_contains: Vec<String>,
}

impl TypesFilter {
    pub fn r#type(mut self, r#type: impl Into<String>) -> Self {
        self.types_contains.push(r#type.into());
        self
    }

    pub fn types(mut self, mut types: Vec<String>) -> Self {
        self.types_contains.append(&mut types);
        self
    }
}

impl From<TypesFilter> for EntityRelationFilter {
    fn from(types_filter: TypesFilter) -> Self {
        let mut filter = EntityRelationFilter::default();

        if !types_filter.types_contains.is_empty() {
            filter = filter.relation_type(system_ids::TYPES_ATTRIBUTE);

            if let [r#type] = &types_filter.types_contains[..] {
                filter = filter.to_id(r#type.to_string());
            } else {
                filter = filter.to_id(types_filter.types_contains);
            }
        }

        filter
    }
}

#[derive(Clone, Debug)]
pub struct MatchEntityAttributes<'a> {
    space_id: &'a Option<PropFilter<String>>,
    version: &'a VersionFilter,
}

impl<'a> MatchEntityAttributes<'a> {
    pub(crate) fn new(
        space_id: &'a Option<PropFilter<String>>,
        version: &'a VersionFilter,
    ) -> Self {
        Self { space_id, version }
    }

    pub fn subquery(
        self,
        node_var: impl Into<String>,
        attributes_node_var: impl Into<String>,
    ) -> MatchQuery {
        let node_var = node_var.into();
        let attrs_node_var = attributes_node_var.into();

        MatchQuery::new_optional(format!(
            "({node_var}) -[attribute:ATTRIBUTE]- ({attrs_node_var}:Attribute)"
        ))
        .r#where(self.version.subquery("attribute"))
        .where_opt(
            self.space_id
                .as_ref()
                .map(|space_id| space_id.subquery("attribute", "space_id", None)),
        )
    }

    // /// Returns a query part that selects the attributes of an entity `node_var`.
    // /// The attributes can be referenced by the `attributes_node_var` variable.
    // pub fn chain(
    //     self,
    //     node_var: impl Into<String>,
    //     attributes_node_var: impl Into<String>,
    //     next: QueryPart,
    // ) -> QueryPart {
    //     let node_var = node_var.into();
    //     let attrs_node_var = attributes_node_var.into();

    //     self.compile(&node_var, &attrs_node_var).with_clause(
    //         format!("{node_var}, {attrs_node_var}{{.*}} AS {attrs_node_var}"),
    //         next,
    //     )
    // }
}

#[derive(Clone, Debug)]
pub struct MatchEntityTypes<'a> {
    space_id: &'a Option<PropFilter<String>>,
    version: &'a VersionFilter,
}

impl<'a> MatchEntityTypes<'a> {
    pub(crate) fn new(
        space_id: &'a Option<PropFilter<String>>,
        version: &'a VersionFilter,
    ) -> Self {
        Self { space_id, version }
    }

    /// Returns a query part that selects the types of an entity `node_var`.
    /// The types can be referenced by the `types_node_var` variable.
    pub fn subquery(
        self,
        node_var: impl Into<String>,
        types_node_var: impl Into<String>,
    ) -> MatchQuery {
        let node_var = node_var.into();
        let types_rel_var = format!("r_{node_var}_types");
        let types_node_var = types_node_var.into();

        MatchQuery::new_optional(format!(r#"({node_var}) -[{types_rel_var}:RELATION {{relation_type: "{}"}}]-> ({types_node_var}:Entity)"#, system_ids::TYPES_ATTRIBUTE))
            .r#where(self.version.subquery(&types_rel_var))
            .where_opt(self.space_id.as_ref()
                .map(|space_id| space_id.subquery(&types_rel_var, "space_id", None)))
    }

    // /// Returns a query part that selects the types of an entity `node_var`.
    // /// The types can be referenced by the `types_node_var` variable.
    // pub fn chain(
    //     self,
    //     node_var: impl Into<String>,
    //     types_node_var: impl Into<String>,
    //     next: QueryPart,
    // ) -> QueryPart {
    //     let node_var = node_var.into();
    //     let types_node_var = types_node_var.into();

    //     self.subquery(&node_var, &types_node_var).with_clause(
    //         format!("{node_var}, {types_node_var}{{.*}} AS {types_node_var}"),
    //         next,
    //     )
    // }
}

#[derive(Clone, Debug)]
pub struct MatchEntity<'a> {
    match_attributes: MatchEntityAttributes<'a>,
    match_types: MatchEntityTypes<'a>,
}

impl<'a> MatchEntity<'a> {
    pub fn new(space_id: &'a Option<PropFilter<String>>, version: &'a VersionFilter) -> Self {
        Self {
            match_attributes: MatchEntityAttributes::new(space_id, version),
            match_types: MatchEntityTypes::new(space_id, version),
        }
    }

    /// Returns a query part that selects the entity `node_var` with its
    /// attributes and types.
    /// The query part will end with a `WITH` clause that contains the entity data and
    /// the attributes and types.
    /// The data can be referenced in subsequent queries with the `entity_node_var` variable.
    pub fn chain(
        self,
        node_var: impl Into<String>,
        attributes_node_var: impl Into<String>,
        types_node_var: impl Into<String>,
        extra_vars: Option<Vec<String>>,
        next: impl Subquery,
    ) -> QueryBuilder {
        let node_var = node_var.into();
        let attributes_node_var = attributes_node_var.into();
        let types_node_var = types_node_var.into();
        // let entity_node_var = entity_node_var.into();

        // let attributes_node_var = format!("{entity_node_var}_attributes");
        // let types_node_var = format!("{entity_node_var}_types");

        let with_vars = vec![
            node_var.clone(),
            format!("COLLECT(DISTINCT {attributes_node_var}{{.*}}) AS {attributes_node_var}"),
            format!("COLLECT(DISTINCT {types_node_var}{{.*}}) AS {types_node_var}"),
        ];
        let with_vars = if let Some(extra_vars) = extra_vars {
            with_vars.into_iter().chain(extra_vars).collect()
        } else {
            with_vars
        };

        QueryBuilder::default()
            .subquery(
                self.match_attributes
                    .subquery(&node_var, &attributes_node_var),
            )
            .subquery(self.match_types.subquery(&node_var, &types_node_var))
            .with(with_vars, next)
    }
}
