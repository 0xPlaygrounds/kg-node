use uuid::Uuid;

use super::{query_builder::MatchQuery, value_filter::ValueFilter};

/// Struct representing a property filter subquery for an entity's properties.
///
/// IMPORTANT: This filter subquery is designed to be used to filter an entity by its
/// properties (and not filter a list of properties!)
///
/// The struct follows the builder pattern to set the filter parameters.
/// ```rust
/// let filter = PropertyFilter::new(system_ids::NAME_PROPERTY)
///     .value(["Bob", "Alice"])
///     .value_type("TEXT")
///     .space_id("25omwWh6HYgeRQKCaSpVpa");
///
/// let subquery = filter.subquery("e");
/// ```
#[derive(Clone, Debug)]
pub struct PropertyFilter {
    property: Uuid,
    value: Option<ValueFilter<String>>,
}

impl PropertyFilter {
    /// Create a new filter subquery for the provided `property`. By default, if no other
    /// parameters are set, this filter subquery will filter entities for which the `property`
    /// exists in the current version of the knowledge graph.
    pub fn new(property: Uuid) -> Self {
        Self {
            property,
            value: None,
        }
    }

    pub fn value(mut self, value: impl Into<ValueFilter<String>>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Compiles the property filter into a Neo4j subquery that will filter the nodes
    /// identified by `node_var` according to the provided parameters.
    ///
    /// The subquery will have the following form:
    /// ```cypher
    /// MATCH ({node_var}) -[r_{node_var}_property:PROPERTY]-> ({node_var}_property:Property {id: $property})
    /// WHERE {VALUE_FILTER}
    /// ```
    ///
    /// For example, if:
    /// - the property to filter on is `LuBWqZAu6pz54eiJS5mLv8`
    /// - the nodes to filter are bound to the variable `e`
    /// - the value filter is set to `["foo", "bar"]`
    ///
    /// the subquery will be:
    /// ```cypher
    /// MATCH (e) -[r_e_property:PROPERTY]-> (e_property:Property {id: $property})
    /// WHERE e_property.value IN ["foo", "bar"]
    /// ```
    /// Note: the `$property` query parameter will contain the value `LuBWqZAu6pz54eiJS5mLv8`
    pub fn subquery(&self, node_var: &str) -> MatchQuery {
        let attr_rel_var = format!("r_{node_var}_{}", self.property.as_simple());
        let attr_node_var = format!("{node_var}_{}", self.property.as_simple());
        let attr_id_var = format!("a_{node_var}_{}", self.property.as_simple());

        let value_filter = if let Some(value_filter) = &self.value {
            value_filter.clone()
        } else {
            // If no value filter is set, we assume the property exists
            ValueFilter::default().exists(true)
        };

        MatchQuery::new(format!(
            "({node_var}) -[{attr_rel_var}:PROPERTIES]-> ({attr_node_var}:Properties)"
        ))
        .r#where(value_filter.subquery(
            &attr_node_var,
            "value",
            Some(&format!("{attr_node_var}[${attr_id_var}]")),
        ))
        .params(attr_id_var, self.property.to_string())
    }
}
