use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct QueryPart {
    /// Unwind clauses, e.g.: "$foo AS bar"
    pub(crate) unwind_clauses: Vec<String>,

    /// Match clauses, e.g.: "(n {id: "123"})", "(n:Foo)", "(n)-[:REL]->(m)"
    pub(crate) match_clauses: Vec<String>,

    /// Where clauses, e.g.: "n.foo = $foo", "n.bar IN $bar"
    pub(crate) where_clauses: Vec<String>,

    /// With clauses, e.g.: "n", "n.bar", "m"
    pub(crate) with_clauses: Option<(String, Box<Self>)>,

    /// Return clauses, e.g.: "n", "n.bar", "m"
    pub(crate) return_clauses: Vec<String>,

    /// Order by clauses, e.g.: "n.foo", "n.bar DESC"
    pub(crate) order_by_clauses: Vec<String>,

    pub(crate) limit: Option<usize>,

    pub(crate) skip: Option<usize>,

    /// Parameters to be passed to the query
    pub(crate) params: HashMap<String, neo4rs::BoltType>,
}

pub fn match_query(query: impl Into<String>) -> QueryPart {
    QueryPart {
        match_clauses: vec![query.into()],
        ..Default::default()
    }
}

pub fn where_query(query: impl Into<String>) -> QueryPart {
    QueryPart {
        where_clauses: vec![query.into()],
        ..Default::default()
    }
}

pub fn return_query(query: impl Into<String>) -> QueryPart {
    QueryPart {
        return_clauses: vec![query.into()],
        ..Default::default()
    }
}

pub fn order_by_query(query: impl Into<String>) -> QueryPart {
    QueryPart {
        order_by_clauses: vec![query.into()],
        ..Default::default()
    }
}

impl QueryPart {
    pub fn unwind_clause(mut self, clause: impl Into<String>) -> Self {
        self.unwind_clauses.push(clause.into());
        self
    }

    // Builder methods
    pub fn match_clause(mut self, clause: impl Into<String>) -> Self {
        self.match_clauses.push(clause.into());
        self
    }

    pub fn where_clause(mut self, clause: impl Into<String>) -> Self {
        self.where_clauses.push(clause.into());
        self
    }

    pub fn return_clause(mut self, clause: impl Into<String>) -> Self {
        // Not the most efficient but important to keep the return clauses unique
        let clause = clause.into();
        if !self.return_clauses.iter().any(|x| *x == clause) {
            self.return_clauses.push(clause);
        }
        self
    }

    pub fn with_clause(mut self, clause: impl Into<String>, other: QueryPart) -> Self {
        self.with_clauses = Some((clause.into(), Box::new(other)));
        self
    }

    pub fn order_by_clause(mut self, clause: impl Into<String>) -> Self {
        // Not the most efficient but important to keep the return clauses unique
        let clause = clause.into();
        if !self.order_by_clauses.iter().any(|x| *x == clause) {
            self.order_by_clauses.push(clause);
        }
        self
    }

    pub fn params(mut self, key: impl Into<String>, value: impl Into<neo4rs::BoltType>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.match_clauses.is_empty()
            && self.where_clauses.is_empty()
            && self.return_clauses.is_empty()
            && self.with_clauses.is_none()
            && self.order_by_clauses.is_empty()
    }

    pub fn merge_mut(&mut self, other: QueryPart) {
        self.match_clauses.extend(other.match_clauses);
        self.where_clauses.extend(other.where_clauses);
        self.return_clauses.extend(other.return_clauses);
        if self.with_clauses.is_none() {
            self.with_clauses = other.with_clauses;
        }
        self.order_by_clauses.extend(other.order_by_clauses);
        self.params.extend(other.params);
    }

    pub fn merge(mut self, other: QueryPart) -> Self {
        self.merge_mut(other);
        self
    }

    pub fn combine(parts: Vec<QueryPart>) -> QueryPart {
        parts
            .into_iter()
            .fold(QueryPart::default(), |acc, part| acc.merge(part))
    }

    pub fn query(&self) -> String {
        let mut query = String::new();

        self.unwind_clauses.iter().for_each(|clause| {
            query.push_str(&format!("UNWIND {clause}\n"));
        });

        self.match_clauses.iter().for_each(|clause| {
            query.push_str(&format!("MATCH {clause}\n"));
        });

        if !self.where_clauses.is_empty() {
            query.push_str("WHERE ");
            query.push_str(&self.where_clauses.join("\nAND "));
            query.push('\n');
        }

        if !self.order_by_clauses.is_empty() {
            query.push_str("ORDER BY ");
            query.push_str(
                &self
                    .order_by_clauses
                    .iter()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            query.push('\n');
        }

        if let Some((clause, other)) = &self.with_clauses {
            query.push_str(&format!("WITH {clause}\n"));
            query.push_str(&other.query());
            query.push('\n');
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!("LIMIT {}\n", limit));
        }

        if let Some(skip) = self.skip {
            query.push_str(&format!("SKIP {}\n", skip));
        }

        if !self.return_clauses.is_empty() {
            query.push_str("RETURN ");
            query.push_str(
                &self
                    .return_clauses
                    .iter()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            query.push('\n');
        }

        query
    }

    pub fn build(self) -> neo4rs::Query {
        let query_str = self.query();
        self.get_params()
            .into_iter()
            .fold(neo4rs::query(&query_str), |query, (key, value)| {
                query.param(key, value.clone())
            })
    }

    pub fn get_params(&self) -> Vec<(&String, &neo4rs::BoltType)> {
        if let Some((_, other)) = &self.with_clauses {
            let mut params = self.params.iter().collect::<Vec<_>>();
            params.extend(other.get_params());
            params
        } else {
            self.params.iter().collect()
        }
    }
}

impl Display for QueryPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{:?}", self.query(), self.params)
    }
}

pub trait IntoQueryParts {
    fn into_query_parts(self) -> Vec<QueryPart>;

    fn merge(self) -> QueryPart
    where
        Self: Sized,
    {
        self.into_query_parts()
            .into_iter()
            .fold(QueryPart::default(), |acc, part| acc.merge(part))
    }

    fn build(self) -> neo4rs::Query
    where
        Self: Sized,
    {
        self.merge().build()
    }
}

pub trait IntoQueryPart {
    fn into_query_part(self) -> QueryPart;

    fn build(self) -> neo4rs::Query
    where
        Self: Sized,
    {
        self.into_query_part().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_query_part() {
        let query_part = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            with_clauses: None,
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: std::collections::HashMap::new(),
            ..Default::default()
        };

        assert_eq!(
            query_part.query(),
            "MATCH (n)\nWHERE n.foo = $foo\nORDER BY n.foo\nRETURN n\n"
        );
    }

    #[test]
    fn test_query_part_params() {
        let query_part = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            with_clauses: None,
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: HashMap::from([("foo".to_owned(), 123.into())]),
            ..Default::default()
        };

        assert_eq!(query_part.params.len(), 1);
        assert_eq!(query_part.params.get("foo").unwrap(), &123.into());
    }

    #[test]
    fn test_query_part_merge() {
        let query_part1 = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            with_clauses: None,
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: std::collections::HashMap::new(),
            ..Default::default()
        };

        let query_part2 = super::QueryPart {
            match_clauses: vec!["(m)".to_owned()],
            where_clauses: vec!["m.bar = $bar".to_owned()],
            with_clauses: None,
            return_clauses: vec!["m".to_owned()],
            order_by_clauses: vec!["m.bar DESC".to_owned()],
            params: std::collections::HashMap::new(),
            ..Default::default()
        };

        let merged = query_part1.merge(query_part2);

        assert_eq!(merged.query(), "MATCH (n)\nMATCH (m)\nWHERE n.foo = $foo\nAND m.bar = $bar\nORDER BY n.foo, m.bar DESC\nRETURN n, m\n");
    }

    #[test]
    fn test_query_part_merge_params() {
        let query_part1 = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            with_clauses: None,
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: HashMap::from([("foo".to_owned(), 123.into())]),
            ..Default::default()
        };

        let query_part2 = super::QueryPart {
            match_clauses: vec!["(m)".to_owned()],
            where_clauses: vec!["m.bar = $bar".to_owned()],
            with_clauses: None,
            return_clauses: vec!["m".to_owned()],
            order_by_clauses: vec!["m.bar DESC".to_owned()],
            params: HashMap::from([
                ("foo".to_owned(), 123.into()),
                ("bar".to_owned(), 456.into()),
            ]),
            ..Default::default()
        };

        let merged = query_part1.merge(query_part2);

        assert_eq!(merged.query(), "MATCH (n)\nMATCH (m)\nWHERE n.foo = $foo\nAND m.bar = $bar\nORDER BY n.foo, m.bar DESC\nRETURN n, m\n");

        assert_eq!(merged.params.len(), 2);
        assert_eq!(merged.params.get("foo").unwrap(), &123.into());
        assert_eq!(merged.params.get("bar").unwrap(), &456.into());
    }

    #[test]
    fn test_query_part_with() {
        let query_part = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            with_clauses: Some((
                "n AS node".to_owned(),
                Box::new(QueryPart {
                    return_clauses: vec!["node".to_owned()],
                    ..Default::default()
                }),
            )),
            order_by_clauses: vec!["n.foo".to_owned()],
            params: std::collections::HashMap::new(),
            ..Default::default()
        };

        assert_eq!(
            query_part.query(),
            "MATCH (n)\nWHERE n.foo = $foo\nORDER BY n.foo\nWITH n AS node\nRETURN node\n\n"
        );
    }
}
