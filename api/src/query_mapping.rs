use std::collections::HashSet;

use juniper::{LookAheadSelection, ScalarValue};

#[derive(Default)]
pub struct QueryMapper {
    node_counter: i32,
    relation_counter: i32,
    match_statements: Vec<String>,
    return_statement_vars: HashSet<String>,
}

impl QueryMapper {
    pub fn node_var(&self) -> String {
        format!("n{}", self.node_counter)
    }

    pub fn relation_var(&self) -> String {
        format!("r{}", self.relation_counter)
    }

    pub fn select_root_node<S: ScalarValue>(
        mut self,
        id: &str,
        selection: &LookAheadSelection<'_, S>,
    ) -> Self {
        let node_var = self.node_var();
        self.node_counter += 1;

        self.match_statements
            .push(format!("MATCH ({} {{id: \"{id}\"}})", node_var));
        self.return_statement_vars.insert(node_var.clone());

        selection
            .children()
            .iter()
            .fold(self, |query, child| match child.field_original_name() {
                "relations" => query.select_node_relations(&node_var, child),
                _ => query,
            })
    }

    pub fn select_node_relations<S: ScalarValue>(
        mut self,
        node_var: &str,
        selection: &LookAheadSelection<'_, S>,
    ) -> Self {
        let to_var = self.node_var();
        let relation_var = self.relation_var();
        self.relation_counter += 1;
        self.node_counter += 1;

        self.match_statements.push(format!(
            "MATCH ({}) -[{}]-> ({})",
            node_var, relation_var, to_var
        ));
        self.return_statement_vars.insert(self.relation_var());

        selection
            .children()
            .iter()
            .fold(self, |query, child| match child.field_original_name() {
                "from" => query.select_relation_from(node_var, child),
                "to" => query.select_relation_to(node_var, child),
                _ => query,
            })
    }

    pub fn select_relation_from<S: ScalarValue>(
        mut self,
        node_var: &str,
        selection: &LookAheadSelection<'_, S>,
    ) -> Self {
        self.return_statement_vars.insert(node_var.to_string());

        selection
            .children()
            .iter()
            .fold(self, |query, child| match child.field_original_name() {
                "relations" => query.select_node_relations(node_var, child),
                _ => query,
            })
    }

    pub fn select_relation_to<S: ScalarValue>(
        mut self,
        node_var: &str,
        selection: &LookAheadSelection<'_, S>,
    ) -> Self {
        self.return_statement_vars.insert(node_var.to_string());

        selection
            .children()
            .iter()
            .fold(self, |query, child| match child.field_original_name() {
                "relations" => query.select_node_relations(node_var, child),
                _ => query,
            })
    }

    pub fn build(self) -> String {
        format!(
            "{}\nRETURN {}",
            self.match_statements.join(",\n"),
            self.return_statement_vars
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
