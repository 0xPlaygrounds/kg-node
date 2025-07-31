use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{update_one, UpdateEntity},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_update_one_entity() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entity with properties
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "test_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entity
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Verify the entity was created
    let entity_result = neo4j
        .execute(
            neo4rs::query("MATCH (e:Entity {id: $id}) RETURN e.id as id")
                .param("id", entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_id: String = entity_result.get("id").unwrap();
    assert_eq!(retrieved_id, entity_id.to_string());

    // Verify the properties node and relationship were created
    let properties_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (e:Entity {id: $entity_id})-[r:PROPERTIES]->(p:Properties)
             RETURN r.space_id as space_id, p as properties",
            )
            .param("entity_id", entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_space_id: String = properties_result.get("space_id").unwrap();
    assert_eq!(retrieved_space_id, space_id.to_string());

    let properties: HashMap<String, String> = properties_result.get("properties").unwrap();
    assert_eq!(
        properties.get(&property_id.to_string()),
        Some(&"test_value".to_string())
    );
}

#[tokio::test]
async fn test_update_one_entity_multiple_properties() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entity with multiple properties in one space
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property1_id = Uuid::new_v4();
    let property2_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let values = vec![
                Value::new(property1_id, "value1"),
                Value::new(property2_id, "value2"),
            ];
            HashMap::from_iter(values.into_iter().map(|v| (v.property, v)))
        },
        embedding: None,
    };

    // Insert the entity
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Verify the properties node was created with both properties
    let properties_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (e:Entity {id: $entity_id})-[r:PROPERTIES]->(p:Properties)
             RETURN r.space_id as space_id, p as properties",
            )
            .param("entity_id", entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_space_id: String = properties_result.get("space_id").unwrap();
    assert_eq!(retrieved_space_id, space_id.to_string());

    let properties: HashMap<String, String> = properties_result.get("properties").unwrap();
    assert_eq!(
        properties.get(&property1_id.to_string()),
        Some(&"value1".to_string())
    );
    assert_eq!(
        properties.get(&property2_id.to_string()),
        Some(&"value2".to_string())
    );
}
