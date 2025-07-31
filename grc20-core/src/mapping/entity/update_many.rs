use super::models::{UnsetEntityValues, UpdateEntity};
use neo4rs::BoltType;
use uuid::Uuid;

/// Trait for types that can be used to update entities.
pub trait EntityUpdate {
    /// Returns the UUID of the entity.
    fn id(&self) -> Uuid;
}

impl EntityUpdate for UpdateEntity {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl EntityUpdate for UnsetEntityValues {
    fn id(&self) -> Uuid {
        self.id
    }
}

/// Creates an UpdateManyQuery for batch updating entities.
///
/// # Arguments
/// * `neo4j` - The Neo4j graph connection.
/// * `entities` - A vector of entities to update.
/// * `space_id` - The space UUID for the update.
///
/// # Returns
/// An instance of `UpdateManyQuery`.
pub fn update_many<T>(neo4j: neo4rs::Graph, space_id: Uuid) -> UpdateManyQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    UpdateManyQuery::new(neo4j, space_id)
}

/// Query type for batch updating entities in Neo4j.
#[derive(Clone)]
pub struct UpdateManyQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    /// The entity updates to process.
    pub updates: Vec<T>,
    /// The space UUID for the update.
    pub space_id: Uuid,
    neo4j: neo4rs::Graph,
}

impl<T> UpdateManyQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    /// Creates a new UpdateManyQuery.
    ///
    /// # Arguments
    /// * `neo4j` - The Neo4j graph connection.
    /// * `entities` - A vector of entities to update.
    /// * `space_id` - The space UUID for the update.
    ///
    /// # Returns
    /// An instance of `UpdateManyQuery`.
    pub fn new(neo4j: neo4rs::Graph, space_id: Uuid) -> Self {
        Self {
            neo4j,
            updates: Vec::new(),
            space_id,
        }
    }

    /// Sets the updates to be updated.
    pub fn updates(mut self, updates: Vec<T>) -> Self {
        self.updates = updates;
        self
    }

    /// Adds an update to the update query.
    pub fn update(mut self, update: T) -> Self {
        self.updates.push(update);
        self
    }

    /// Sends the batch update query to Neo4j.
    ///
    /// # Errors
    /// Returns a `neo4rs::Error` if the query fails.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            UNWIND $updates AS update
            MERGE (e:Entity {id: update.id})
            MERGE (e)-[:PROPERTIES {space_id: $space_id}]->(p:Properties)
            WITH e, p, update
            UNWIND update.values AS value
            SET p[value.property] = value.value
            SET p.embedding = update.embedding
        ";

        let updates: Vec<BoltType> = self.updates.into_iter().map(Into::into).collect();

        let query = neo4rs::query(cypher)
            .param("updates", updates)
            .param("space_id", self.space_id.to_string());
        self.neo4j.run(query).await?;
        Ok(())
    }
}
