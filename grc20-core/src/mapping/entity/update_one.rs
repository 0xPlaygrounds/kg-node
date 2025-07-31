use super::models::{UnsetEntityValues, UpdateEntity};
use neo4rs::BoltType;
use uuid::Uuid;

pub trait EntityUpdate {
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

pub fn update_one<T>(neo4j: &neo4rs::Graph, update: T, space_id: Uuid) -> UpdateOneQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    UpdateOneQuery::new(neo4j, update, space_id)
}

#[derive(Clone)]
pub struct UpdateOneQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    pub update: T,
    pub space_id: Uuid,
    neo4j: neo4rs::Graph,
}

impl<T> UpdateOneQuery<T>
where
    T: EntityUpdate + Into<BoltType> + Clone,
{
    pub fn new(neo4j: &neo4rs::Graph, update: T, space_id: Uuid) -> Self {
        Self {
            neo4j: neo4j.clone(),
            update,
            space_id,
        }
    }

    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            MERGE (e:Entity {id: $update.id})
            MERGE (e)-[:PROPERTIES {space_id: $space_id}]->(p:Properties)
            WITH e, p
            UNWIND $update.values AS value
            SET p[value.property] = value.value
            SET p.embedding = $update.embedding
        ";

        let query = neo4rs::query(cypher)
            .param("update", self.update)
            .param("space_id", self.space_id.to_string());
        self.neo4j.run(query).await?;
        Ok(())
    }
}
