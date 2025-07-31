use crate::mapping::query_utils::{QueryBuilder, Subquery};

use super::models::Property;

pub fn insert_one(neo4j: &neo4rs::Graph, property: Property) -> InsertOneQuery {
    InsertOneQuery::new(neo4j, property)
}

/// Query struct for inserting a single property into the database.
#[derive(Clone)]
pub struct InsertOneQuery {
    /// The property to be inserted.
    pub property: Property,
    neo4j: neo4rs::Graph,
}

impl InsertOneQuery {
    /// Creates a new InsertOneQuery from a Property.
    pub fn new(neo4j: &neo4rs::Graph, property: Property) -> Self {
        InsertOneQuery {
            property,
            neo4j: neo4j.clone(),
        }
    }
}

impl InsertOneQuery {
    /// Executes the query to insert the property into the Neo4j database.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let query = QueryBuilder::default()
            .subqueries(vec![
                "MERGE (p:Entity {id: $prop.id})",
                "SET p.data_type = $prop.data_type",
            ])
            .params("prop", self.property);

        self.neo4j.run(query.build()).await?;

        Ok(())
    }
}
