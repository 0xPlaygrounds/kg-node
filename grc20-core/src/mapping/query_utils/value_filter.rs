use neo4rs::BoltType;
use uuid::Uuid;

use crate::value::Value;

use super::query_builder::WhereClause;

pub fn value<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value(value)
}

pub fn value_gt<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value_gt(value)
}

pub fn value_gte<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value_gte(value)
}

pub fn value_lt<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value_lt(value)
}

pub fn value_lte<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value_lte(value)
}

pub fn value_not<T>(value: impl Into<T>) -> ValueFilter<T> {
    ValueFilter::default().value_not(value)
}

pub fn value_in<T>(values: Vec<T>) -> ValueFilter<T> {
    ValueFilter::default().value_in(values)
}

pub fn value_not_in<T>(values: Vec<T>) -> ValueFilter<T> {
    ValueFilter::default().value_not_in(values)
}

pub fn value_exists<T>() -> ValueFilter<T> {
    ValueFilter::default().exists(true)
}

impl From<&str> for ValueFilter<String> {
    fn from(value: &str) -> Self {
        ValueFilter::default().value(value.to_string())
    }
}

impl From<Uuid> for ValueFilter<String> {
    fn from(value: Uuid) -> Self {
        ValueFilter::default().value(value.to_string())
    }
}

impl<T> From<T> for ValueFilter<T> {
    fn from(value: T) -> Self {
        ValueFilter::default().value(value)
    }
}

impl<T> From<Vec<T>> for ValueFilter<T> {
    fn from(value: Vec<T>) -> Self {
        ValueFilter::default().value_in(value)
    }
}

impl<T: ToString> ValueFilter<T> {
    pub fn as_string_filter(&self) -> ValueFilter<String> {
        ValueFilter {
            value: self.value.as_ref().map(|v| v.to_string()),
            value_gt: self.value_gt.as_ref().map(|v| v.to_string()),
            value_gte: self.value_gte.as_ref().map(|v| v.to_string()),
            value_lt: self.value_lt.as_ref().map(|v| v.to_string()),
            value_lte: self.value_lte.as_ref().map(|v| v.to_string()),
            value_not: self.value_not.as_ref().map(|v| v.to_string()),
            value_in: self
                .value_in
                .as_ref()
                .map(|v| v.into_iter().map(|v| v.to_string()).collect()),
            value_not_in: self
                .value_not_in
                .as_ref()
                .map(|v| v.into_iter().map(|v| v.to_string()).collect()),
            exists: self.exists,
        }
    }
}

/// Filter for property P of node N
#[derive(Clone, Debug)]
pub struct ValueFilter<T> {
    value: Option<T>,
    value_gt: Option<T>,
    value_gte: Option<T>,
    value_lt: Option<T>,
    value_lte: Option<T>,
    value_not: Option<T>,
    value_in: Option<Vec<T>>,
    value_not_in: Option<Vec<T>>,
    exists: Option<bool>, // This field is not used in the current implementation, but can be used to check if the property exists
                          // or: Option<Vec<PropFilter<T>>>,
}

impl<T> Default for ValueFilter<T> {
    fn default() -> Self {
        Self {
            value: None,
            value_gt: None,
            value_gte: None,
            value_lt: None,
            value_lte: None,
            value_not: None,
            value_in: None,
            value_not_in: None,
            exists: None, // Default to None, meaning no existence check
        }
    }
}

impl<T> ValueFilter<T> {
    pub fn value(mut self, value: impl Into<T>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn value_mut(&mut self, value: impl Into<T>) {
        self.value = Some(value.into());
    }

    pub fn value_gt(mut self, value: impl Into<T>) -> Self {
        self.value_gt = Some(value.into());
        self
    }

    pub fn value_gt_mut(&mut self, value: impl Into<T>) {
        self.value_gt = Some(value.into());
    }

    pub fn value_gte(mut self, value: impl Into<T>) -> Self {
        self.value_gte = Some(value.into());
        self
    }

    pub fn value_gte_mut(&mut self, value: impl Into<T>) {
        self.value_gte = Some(value.into());
    }

    pub fn value_lt(mut self, value: impl Into<T>) -> Self {
        self.value_lt = Some(value.into());
        self
    }

    pub fn value_lt_mut(&mut self, value: impl Into<T>) {
        self.value_lt = Some(value.into());
    }

    pub fn value_lte(mut self, value: impl Into<T>) -> Self {
        self.value_lte = Some(value.into());
        self
    }

    pub fn value_lte_mut(&mut self, value: impl Into<T>) {
        self.value_lte = Some(value.into());
    }

    pub fn value_not(mut self, value: impl Into<T>) -> Self {
        self.value_not = Some(value.into());
        self
    }

    pub fn value_not_mut(&mut self, value: impl Into<T>) {
        self.value_not = Some(value.into());
    }

    pub fn value_in(mut self, values: Vec<T>) -> Self {
        self.value_in = Some(values);
        self
    }

    pub fn value_in_mut(&mut self, values: Vec<T>) {
        self.value_in = Some(values);
    }

    pub fn value_not_in(mut self, values: Vec<T>) -> Self {
        self.value_not_in = Some(values);
        self
    }

    pub fn value_not_in_mut(&mut self, values: Vec<T>) {
        self.value_not_in = Some(values);
    }

    pub fn exists(mut self, exists: bool) -> Self {
        self.exists = Some(exists);
        self
    }
}

impl<T: Clone + Into<BoltType>> ValueFilter<T> {
    /// Compiles the attribute filter into a [WhereClause] Neo4j subquery that will apply
    /// a filter on the `key` field of the `node_var` node(s) (i.e.: `{node_var}.{key}`).
    ///
    /// If `expr` is set, then it will used as the filter target instead of the above.
    ///
    /// For example, given the following [PropFilter] (which creates a property filter)
    /// ```rust
    /// # fn main() {
    /// # use std::collections::HashMap;
    /// # use grc20_core::mapping::{PropFilter, Subquery};
    /// let prop_filter = PropFilter::default()
    ///     .value_not("Bob")
    ///     .value_lt("Gary");
    ///
    /// let query = prop_filter.subquery("e", "name", None);
    /// assert_eq!(
    ///     query.compile(),
    ///     "WHERE e.name <> $e_name_value_not\nAND e.name < $e_name_value_lt"
    /// )
    /// assert_eq!(
    ///     query.params(),
    ///     HashMap::from([
    ///         ("e_name_value_not", "Bob"),
    ///         ("e_name_value_lt", "Gary")
    ///     ])
    /// )
    ///
    /// let query = prop_filter.subquery("e", "name", Some("my_expr"));
    /// assert_eq!(
    ///     query.compile(),
    ///     "WHERE my_expr <> $e_name_value_not\nAND my_expr < $e_name_value_lt"
    /// )
    /// # }
    /// ```
    pub fn subquery(&self, node_var: &str, key: &str, expr: Option<&str>) -> WhereClause {
        let mut where_clause = WhereClause::default();

        let expr = expr
            .map(|e| e.to_string())
            .unwrap_or(format!("{node_var}.`{key}`"));

        if let Some(value) = &self.value {
            let param_key = format!("{node_var}_{key}_value");
            where_clause = where_clause
                .clause(format!("{expr} = ${param_key}"))
                .set_param(param_key, value.clone());
        }

        if let Some(value_gt) = &self.value_gt {
            let param_key = format!("{node_var}_{key}_value_gt");
            where_clause = where_clause
                .clause(format!("{expr} > ${param_key}"))
                .set_param(param_key, value_gt.clone());
        }

        if let Some(value_gte) = &self.value_gte {
            let param_key = format!("{node_var}_{key}_value_gte");
            where_clause = where_clause
                .clause(format!("{expr} >= ${param_key}"))
                .set_param(param_key, value_gte.clone());
        }

        if let Some(value_lt) = &self.value_lt {
            let param_key = format!("{node_var}_{key}_value_lt");
            where_clause = where_clause
                .clause(format!("{expr} < ${param_key}"))
                .set_param(param_key, value_lt.clone());
        }

        if let Some(value_lte) = &self.value_lte {
            let param_key = format!("{node_var}_{key}_value_lte");
            where_clause = where_clause
                .clause(format!("{expr} <= ${param_key}"))
                .set_param(param_key, value_lte.clone());
        }

        if let Some(value_not) = &self.value_not {
            let param_key = format!("{node_var}_{key}_value_not");
            where_clause = where_clause
                .clause(format!("{expr} <> ${param_key}"))
                .set_param(param_key, value_not.clone());
        }

        if let Some(value_in) = &self.value_in {
            let param_key = format!("{node_var}_{key}_value_in");
            where_clause = where_clause
                .clause(format!("{expr} IN ${param_key}"))
                .set_param(param_key, value_in.clone());
        }

        if let Some(value_not_in) = &self.value_not_in {
            let param_key = format!("{node_var}_{key}_value_not_in");
            where_clause = where_clause
                .clause(format!("{expr} NOT IN ${param_key}"))
                .set_param(param_key, value_not_in.clone());
        }

        if where_clause.params.is_empty() {
            match self.exists {
                Some(true) => {
                    where_clause = where_clause.clause(format!("{expr} IS NOT NULL"));
                }
                Some(false) => {
                    where_clause = where_clause.clause(format!("{expr} IS NULL"));
                }
                _ => (),
            }
        }

        where_clause
    }
}

impl<T: Into<Value>> ValueFilter<T> {
    pub fn as_string(self) -> ValueFilter<String> {
        ValueFilter {
            value: self.value.map(|v| v.into().value),
            value_gt: self.value_gt.map(|v| v.into().value),
            value_gte: self.value_gte.map(|v| v.into().value),
            value_lt: self.value_lt.map(|v| v.into().value),
            value_lte: self.value_lte.map(|v| v.into().value),
            value_not: self.value_not.map(|v| v.into().value),
            value_in: self
                .value_in
                .map(|v| v.into_iter().map(|v| v.into().value).collect()),
            value_not_in: self
                .value_not_in
                .map(|v| v.into_iter().map(|v| v.into().value).collect()),
            exists: self.exists,
        }
    }
}
