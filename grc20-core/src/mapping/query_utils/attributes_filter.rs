use super::{prop_filter::PropFilter, query_builder::MatchQuery, version_filter::VersionFilter};

/// Struct representing an attribute filter subquery for an entity's attributes.
///
/// IMPORTANT: This filter subquery is designed to be used to filter an entity by its
/// attributes (and not filter a list of attributes!)
///
/// The struct follows the builder pattern to set the filter parameters.
/// ```rust
/// let filter = AttributeFilter::new(system_ids::NAME_ATTRIBUTE)
///     .value(["Bob", "Alice"])
///     .value_type("TEXT")
///     .space_id("25omwWh6HYgeRQKCaSpVpa");
///
/// let subquery = filter.subquery("e");
/// ```
#[derive(Clone, Debug)]
pub struct AttributeFilter {
    attribute: String,
    space_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl AttributeFilter {
    /// Create a new filter subquery for the provided `attribute`. By default, if no other
    /// parameters are set, this filter subquery will filter entities for which the `attribute`
    /// exists in the current version of the knowledge graph.  
    pub fn new(attribute: &str) -> Self {
        Self {
            attribute: attribute.to_owned(),
            space_id: None,
            value: None,
            value_type: None,
            version: VersionFilter::default(),
        }
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn space_id_mut(&mut self, space_id: impl Into<PropFilter<String>>) -> &mut Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn value(mut self, value: impl Into<PropFilter<String>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn value_type(mut self, value_type: impl Into<PropFilter<String>>) -> Self {
        self.value_type = Some(value_type.into());
        self
    }

    pub fn version(mut self, space_version: impl Into<String>) -> Self {
        self.version.version_mut(space_version.into());
        self
    }

    pub fn version_opt(mut self, space_version: Option<String>) -> Self {
        self.version.version_opt(space_version);
        self
    }

    pub fn version_mut(&mut self, space_version: impl Into<String>) -> &mut Self {
        self.version.version_mut(space_version.into());
        self
    }

    /// Compiles the attribute filter into a Neo4j subquery that will filter the nodes
    /// identified by `node_var` according to the provided parameters.
    ///
    /// The subquery will have the following form:
    /// ```cypher
    /// MATCH ({node_var}) -[r_{node_var}_attribute:ATTRIBUTE]-> ({node_var}_attribute:Attribute {id: $attribute})
    /// WHERE {VERSION_FILTER}
    /// AND {SPACE_ID_FITLER}
    /// AND {VALUE_FILTER}
    /// AND {VALUE_TYPE_FILTER}
    /// ```
    ///
    /// For example, if:
    /// - the attribute to filter on is `LuBWqZAu6pz54eiJS5mLv8`
    /// - the nodes to filter are bound to the variable `e`
    /// - the version filter is set to filter the current version
    /// - the value filter is set to `["foo", "bar"]`
    /// - the value type filter set to `TEXT`
    /// - the space id filter set to `25omwWh6HYgeRQKCaSpVpa`
    ///
    /// the subquery will be:
    /// ```cypher
    /// MATCH (e) -[r_e_attribute:ATTRIBUTE]-> (e_attribute:Attribute {id: $attribute})
    /// WHERE r_e_attribute.max_version IS NULL
    /// AND r_e_attribute.space_id = "25omwWh6HYgeRQKCaSpVpa"
    /// AND e_attribute.value IN ["foo", "bar"]
    /// AND e_attribute.value_type = "TEXT"
    /// ```
    /// Note: the `$attribute` query parameter will contain the value `LuBWqZAu6pz54eiJS5mLv8`
    pub fn subquery(&self, node_var: &str) -> MatchQuery {
        let attr_rel_var = format!("r_{node_var}_{}", self.attribute);
        let attr_node_var = format!("{node_var}_{}", self.attribute);
        let attr_id_var = format!("a_{node_var}_{}", self.attribute);

        MatchQuery::new(
            format!("({node_var}) -[{attr_rel_var}:ATTRIBUTE]-> ({attr_node_var}:Attribute {{id: ${attr_id_var}}})")
        )
            .r#where(self.version.subquery(&attr_rel_var))
            .where_opt(
                self.space_id.as_ref().map(|space_id| space_id.subquery(&attr_rel_var, "space_id", None))
            )
            .where_opt(
                self.value.as_ref().map(|value| value.subquery(&attr_node_var, "value", None))
            )
            .where_opt(
                self.value_type.as_ref().map(|value_type| value_type.subquery(&attr_node_var, "value_type", None))
            )
            .params(attr_id_var, self.attribute.clone())
    }
}
