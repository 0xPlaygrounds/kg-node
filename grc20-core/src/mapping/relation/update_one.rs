use super::models::{UnsetRelationFields, UpdateRelation};
use neo4rs::BoltType;
use uuid::Uuid;

pub trait RelationUpdate {
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

pub fn update_one<T>(neo4j: &neo4rs::Graph, update_data: T) -> UpdateOneQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    UpdateOneQuery::new(neo4j, update_data)
}

#[derive(Clone)]
pub struct UpdateOneQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    pub update_data: T,
    neo4j: neo4rs::Graph,
}

impl<T> UpdateOneQuery<T>
where
    T: RelationUpdate + Into<BoltType> + Clone,
{
    pub fn new(neo4j: &neo4rs::Graph, update_data: T) -> Self {
        Self {
            neo4j: neo4j.clone(),
            update_data,
        }
    }

    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            MATCH ()-[r:RELATION {id: $relation_id}]->()
            SET r += $updates
        ";

        let query = neo4rs::query(cypher)
            .param("relation_id", self.update_data.id().to_string())
            .param("updates", self.update_data);

        self.neo4j.run(query).await?;
        Ok(())
    }
}
