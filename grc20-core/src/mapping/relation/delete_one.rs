use uuid::Uuid;

pub fn delete_one(neo4j: neo4rs::Graph, id: Uuid) -> DeleteOneQuery {
    DeleteOneQuery::new(neo4j, id)
}

#[derive(Clone)]
pub struct DeleteOneQuery {
    pub id: Uuid,
    neo4j: neo4rs::Graph,
}

impl DeleteOneQuery {
    pub fn new(neo4j: neo4rs::Graph, id: Uuid) -> Self {
        Self { neo4j, id }
    }

    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            MATCH (from)-[r:RELATION {id: $relation_id}]->(to)
            REMOVE from:$(to.id)
            DELETE r
        ";

        let query = neo4rs::query(cypher).param("relation_id", self.id.to_string());
        self.neo4j.run(query).await?;
        Ok(())
    }
}
