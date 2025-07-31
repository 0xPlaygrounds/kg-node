use super::models::CreateRelation;

pub fn insert_one(neo4j: &neo4rs::Graph, relation: CreateRelation) -> InsertOneQuery {
    InsertOneQuery::new(neo4j, relation)
}

#[derive(Clone)]
pub struct InsertOneQuery {
    pub relation: CreateRelation,
    neo4j: neo4rs::Graph,
}

impl InsertOneQuery {
    pub fn new(neo4j: &neo4rs::Graph, relation: CreateRelation) -> Self {
        Self {
            neo4j: neo4j.clone(),
            relation,
        }
    }

    pub async fn send(self) -> Result<(), neo4rs::Error> {
        let cypher = "
            MATCH (from_entity:Entity {id: $relation.from_entity})
            MATCH (to_entity:Entity {id: $relation.to_entity})
            CREATE (from_entity)-[r:RELATION]->(to_entity)
            SET r = $relation
            SET from_entity:$($relation.labels)
        ";

        let query = neo4rs::query(cypher).param("relation", self.relation);
        self.neo4j.run(query).await?;
        Ok(())
    }
}
