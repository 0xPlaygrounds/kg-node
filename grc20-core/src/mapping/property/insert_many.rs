use crate::mapping::query_utils::{QueryBuilder, Subquery};

use super::models::Property;

pub fn insert_many(neo4j: neo4rs::Graph) -> InsertManyQuery {
    InsertManyQuery::new(neo4j)
}

/// Query struct for inserting multiple properties into the database.
#[derive(Clone)]
pub struct InsertManyQuery {
    /// The properties to be inserted.
    pub properties: Vec<Property>,
    neo4j: neo4rs::Graph,
}

impl InsertManyQuery {
    /// Creates a new InsertManyQuery with an empty property list.
    pub fn new(neo4j: neo4rs::Graph) -> Self {
        InsertManyQuery {
            properties: Vec::new(),
            neo4j,
        }
    }

    /// Sets the properties to be inserted.
    pub fn properties(mut self, props: Vec<Property>) -> Self {
        self.properties = props;
        self
    }

    /// Adds a property to the query.
    pub fn property(mut self, prop: Property) -> Self {
        self.properties.push(prop);
        self
    }

    /// Executes the query to insert all properties into the Neo4j database.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        // Use UNWIND for batch insertion
        let query = QueryBuilder::default()
            .subqueries(vec![
                "UNWIND $props AS prop",
                "MERGE (p:Entity {id: prop.id})",
                "SET p.data_type = prop.data_type",
            ])
            .params("props", self.properties);

        self.neo4j.run(query.build()).await?;

        Ok(())
    }
}
