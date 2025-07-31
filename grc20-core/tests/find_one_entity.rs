use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{find_one, update_one, UpdateEntity},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_find_one_entity_exists() {
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

    // Find the entity
    let found_entity = find_one(&neo4j, entity_id)
        .send()
        .await
        .expect("Failed to execute find query")
        .expect("Entity should exist");

    // Verify the entity was found correctly
    assert_eq!(found_entity.id, entity_id);
    assert_eq!(found_entity.values.len(), 1);

    let space_values = found_entity
        .values
        .get(&space_id)
        .expect("Space should exist");
    assert_eq!(space_values.len(), 1);
    assert_eq!(space_values[&property_id].value, "test_value");
}

#[tokio::test]
async fn test_find_one_entity_not_exists() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let non_existent_id = Uuid::new_v4();

    // Try to find non-existent entity
    let result = find_one(&neo4j, non_existent_id)
        .send()
        .await
        .expect("Failed to execute find query");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_one_entity_multiple_properties() {
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

    // Find the entity
    let found_entity = find_one(&neo4j, entity_id)
        .send()
        .await
        .expect("Failed to execute find query")
        .expect("Entity should exist");

    // Verify the entity was found correctly
    assert_eq!(found_entity.id, entity_id);
    assert_eq!(found_entity.values.len(), 1);

    let space_values = found_entity
        .values
        .get(&space_id)
        .expect("Space should exist");
    assert_eq!(space_values.len(), 2);

    // Check both properties exist (order may vary)
    let mut found_properties = HashMap::new();
    for (_, value) in space_values {
        found_properties.insert(value.property, &value.value);
    }

    assert_eq!(
        *found_properties
            .get(&property1_id)
            .expect("property1 exists"),
        "value1"
    );
    assert_eq!(
        *found_properties
            .get(&property2_id)
            .expect("property2 exists"),
        "value2"
    );
}

#[tokio::test]
async fn test_find_one_entity_multiple_spaces() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity_id = Uuid::new_v4();
    let space1_id = Uuid::new_v4();
    let space2_id = Uuid::new_v4();
    let property1_id = Uuid::new_v4();
    let property2_id = Uuid::new_v4();

    // Insert entity with properties in first space
    let entity1 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property1_id, "value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity1, space1_id)
        .send()
        .await
        .expect("Failed to insert entity in space1");

    // Insert entity with properties in second space
    let entity2 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property2_id, "value2");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity2, space2_id)
        .send()
        .await
        .expect("Failed to insert entity in space2");

    // Find the entity
    let found_entity = find_one(&neo4j, entity_id)
        .send()
        .await
        .expect("Failed to execute find query")
        .expect("Entity should exist");

    // Verify the entity was found correctly with both spaces
    assert_eq!(found_entity.id, entity_id);
    assert_eq!(found_entity.values.len(), 2);

    let space1_values = found_entity
        .values
        .get(&space1_id)
        .expect("Space1 should exist");
    assert_eq!(space1_values.len(), 1);
    assert_eq!(space1_values[&property1_id].value, "value1");

    let space2_values = found_entity
        .values
        .get(&space2_id)
        .expect("Space2 should exist");
    assert_eq!(space2_values.len(), 1);
    assert_eq!(space2_values[&property2_id].value, "value2");
}

#[tokio::test]
async fn test_find_one_entity_no_properties() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity_id = Uuid::new_v4();

    // Create entity node without properties
    neo4j
        .run(neo4rs::query("CREATE (e:Entity {id: $id})").param("id", entity_id.to_string()))
        .await
        .expect("Failed to create entity");

    // Find the entity
    let found_entity = find_one(&neo4j, entity_id)
        .send()
        .await
        .expect("Failed to execute find query")
        .expect("Entity should exist");

    // Verify the entity was found correctly with no properties
    assert_eq!(found_entity.id, entity_id);
    assert!(found_entity.values.is_empty());
}
