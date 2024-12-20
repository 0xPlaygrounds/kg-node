/// Wrapper around neo4rs::Query to allow for type-safe queries.
/// `T` is the type of the result of the query.
pub struct Query<T> {
    pub query: neo4rs::Query,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Query<T> {
    pub fn new(query: &str) -> Self {
        Self {
            query: neo4rs::query(query),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn param<U: Into<neo4rs::BoltType>>(mut self, key: &str, value: U) -> Self {
        self.query = self.query.param(key, value);
        self
    }
}
