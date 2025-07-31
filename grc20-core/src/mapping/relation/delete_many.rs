use neo4rs::BoltType;
use uuid::Uuid;

/// Creates a DeleteManyQuery for batch deleting relations.
///
/// # Arguments
/// * `neo4j` - The Neo4j graph connection.
/// * `ids` - A vector of relation UUIDs to delete.
///
/// # Returns
/// An instance of `DeleteManyQuery`.
pub fn delete_many(neo4j: neo4rs::Graph, ids: Vec<Uuid>) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, ids)
}

/// Query type for batch deleting relations in Neo4j.
#[derive(Clone)]
pub struct DeleteManyQuery {
    /// The relation UUIDs to delete.
    pub ids: Vec<Uuid>,
    neo4j: neo4rs::Graph,
}

impl DeleteManyQuery {
    /// Creates a new DeleteManyQuery.
    ///
    /// # Arguments
    /// * `neo4j` - The Neo4j graph connection.
    /// * `ids` - A vector of relation UUIDs to delete.
    ///
    /// # Returns
    /// An instance of `DeleteManyQuery`.
    pub fn new(neo4j: neo4rs::Graph, ids: Vec<Uuid>) -> Self {
        Self { neo4j, ids }
    }

    /// Sends the batch delete query to Neo4j.
    ///
    /// # Errors
    /// Returns a `neo4rs::Error` if the query fails.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            UNWIND $relation_ids AS relation_id
            MATCH ()-[r:RELATION {id: relation_id}]->()
            DELETE r
        ";

        let relation_ids: Vec<BoltType> = self
            .ids
            .into_iter()
            .map(|id| id.to_string().into())
            .collect();

        let query = neo4rs::query(cypher).param("relation_ids", relation_ids);
        self.neo4j.run(query).await?;
        Ok(())
    }
}
