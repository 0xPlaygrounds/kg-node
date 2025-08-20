#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct SearchTraversalInputFilter {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traversal_filter: Option<TraversalFilter>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct TraversalFilter {
    pub relation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<RelationDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traversal_filter: Option<Box<TraversalFilter>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Clone)]
pub enum RelationDirection {
    From,
    To,
}

/// Struct returned by call to `OneOrMany::into_iter()`.
pub struct IntoIter {
    // Owned.
    next_filter: Option<TraversalFilter>,
}

/// Implement `IntoIterator` for `TraversalFilter`.
impl IntoIterator for TraversalFilter {
    type Item = TraversalFilter;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            next_filter: Some(self),
        }
    }
}

/// Implement `Iterator` for `IntoIter`.
impl Iterator for IntoIter {
    type Item = TraversalFilter;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_filter.take().map(|mut current| {
            self.next_filter = current.traversal_filter.take().map(|boxed| *boxed);
            current
        })
    }
}
