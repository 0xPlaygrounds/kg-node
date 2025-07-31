use super::models::{UnsetRelationFields, UpdateRelation};
use neo4rs::BoltType;
use uuid::Uuid;

/// Trait for types that can be used to update relations.
pub trait RelationUpdate {
    /// Returns the UUID of the relation.
    fn id(&self) -> Uuid;
}

impl RelationUpdate for UpdateRelation {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl RelationUpdate for UnsetRelationFields {
    fn id(&self) -> Uuid {
        self.id
    }
}

/// Creates an UpdateManyQuery for batch updating relations.
///
/// # Arguments
/// * `neo4j` - The Neo4j graph connection.
/// * `update_data` - A vector of relation updates to apply.
///
/// # Returns
/// An instance of `UpdateManyQuery`.
pub fn update_many<T>(neo4j: neo4rs::Graph, update_data: Vec<T>) -> UpdateManyQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    UpdateManyQuery::new(neo4j, update_data)
}

/// Query type for batch updating relations in Neo4j.
#[derive(Clone)]
pub struct UpdateManyQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    /// The relation updates to apply.
    pub update_data: Vec<T>,
    neo4j: neo4rs::Graph,
}

impl<T> UpdateManyQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    /// Creates a new UpdateManyQuery.
    ///
    /// # Arguments
    /// * `neo4j` - The Neo4j graph connection.
    /// * `update_data` - A vector of relation updates to apply.
    ///
    /// # Returns
    /// An instance of `UpdateManyQuery`.
    pub fn new(neo4j: neo4rs::Graph, update_data: Vec<T>) -> Self {
        Self { neo4j, update_data }
    }

    /// Sends the batch update query to Neo4j.
    ///
    /// # Errors
    /// Returns a `neo4rs::Error` if the query fails.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            UNWIND $updates AS update
            MATCH ()-[r:RELATION {id: update.id}]->()
            SET r += update
        ";

        let updates: Vec<BoltType> = self.update_data.into_iter().map(Into::into).collect();

        let query = neo4rs::query(cypher).param("updates", updates);

        self.neo4j.run(query).await?;
        Ok(())
    }
}
