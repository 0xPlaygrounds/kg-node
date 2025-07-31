use super::models::CreateRelation;
use neo4rs::BoltType;

/// Creates an InsertManyQuery for batch inserting relations.
///
/// # Arguments
/// * `neo4j` - The Neo4j graph connection.
/// * `relations` - A vector of relations to insert.
///
/// # Returns
/// An instance of `InsertManyQuery`.
pub fn insert_many(neo4j: neo4rs::Graph) -> InsertManyQuery {
    InsertManyQuery::new(neo4j)
}

/// Query type for batch inserting relations in Neo4j.
#[derive(Clone)]
pub struct InsertManyQuery {
    /// The relations to insert.
    pub relations: Vec<CreateRelation>,
    neo4j: neo4rs::Graph,
}

impl InsertManyQuery {
    /// Creates a new InsertManyQuery.
    ///
    /// # Arguments
    /// * `neo4j` - The Neo4j graph connection.
    /// * `relations` - A vector of relations to insert.
    ///
    /// # Returns
    /// An instance of `InsertManyQuery`.
    pub fn new(neo4j: neo4rs::Graph) -> Self {
        Self {
            neo4j,
            relations: Vec::new(),
        }
    }

    /// Sets the relations to be inserted.
    pub fn relations(mut self, relations: Vec<CreateRelation>) -> Self {
        self.relations = relations;
        self
    }

    /// Adds a relation to the query.
    pub fn relation(mut self, relation: CreateRelation) -> Self {
        self.relations.push(relation);
        self
    }

    /// Sends the batch insert query to Neo4j.
    ///
    /// # Errors
    /// Returns a `neo4rs::Error` if the query fails.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            UNWIND $relations AS relation
            MATCH (from_entity:Entity {id: relation.from_entity})
            MATCH (to_entity:Entity {id: relation.to_entity})
            CREATE (from_entity)-[r:RELATION]->(to_entity)
            SET r = relation
            SET from_entity:$(relation.labels)
        ";

        let relations: Vec<BoltType> = self.relations.into_iter().map(BoltType::from).collect();

        let query = neo4rs::query(cypher).param("relations", relations);
        self.neo4j.run(query).await?;
        Ok(())
    }
}
