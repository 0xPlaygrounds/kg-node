use super::models::Value;
use crate::error::DatabaseError;
use uuid::Uuid;

/// Query struct for finding a single value by entity ID, property ID, and space ID
#[derive(Clone)]
pub struct FindOneQuery {
    pub space_id: Uuid,
    pub entity_id: Uuid,
    pub property_id: Uuid,
    neo4j: neo4rs::Graph,
}

impl FindOneQuery {
    /// Create a new FindOneQuery instance
    pub fn new(neo4j: &neo4rs::Graph, space_id: Uuid, entity_id: Uuid, property_id: Uuid) -> Self {
        Self {
            neo4j: neo4j.clone(),
            space_id,
            entity_id,
            property_id,
        }
    }

    /// Execute the query and return the found Value
    pub async fn send(self) -> Result<Option<Value>, DatabaseError> {
        let cypher = "
            MATCH (e:Entity {id: $entity_id})-[:PROPERTIES {space_id: $space_id}]->(p:Properties)
            WHERE p[$property_key] IS NOT NULL
            RETURN p[$property_key] as value
        ";

        let property_key = self.property_id.to_string();

        let query = neo4rs::query(cypher)
            .param("entity_id", self.entity_id.to_string())
            .param("space_id", self.space_id.to_string())
            .param("property_key", property_key);

        let mut result = self.neo4j.execute(query).await?;

        if let Some(row) = result.next().await? {
            let value_data: String = match row.get("value") {
                Ok(val) => val,
                Err(_) => return Ok(None),
            };

            let value = Value {
                property: self.property_id,
                value: value_data,
                options: None, // Options not supported in current update_one implementation
            };

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

/// Create a new FindOneQuery instance
pub fn find_one(
    neo4j: &neo4rs::Graph,
    space_id: Uuid,
    entity_id: Uuid,
    property_id: Uuid,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, space_id, entity_id, property_id)
}
