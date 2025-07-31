use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{update_one, UpdateEntity},
    value::{
        find_one,
        models::{Options, TextOptions, Value},
    },
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_find_one_value_basic() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data
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

    // Insert the entity with its properties
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Find the value
    let found_value = find_one(&neo4j, space_id, entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value should exist");

    // Verify the found value
    assert_eq!(found_value.property, property_id);
    assert_eq!(found_value.value, "test_value");
    assert_eq!(found_value.options, None);
}

// Note: Options-related tests are commented out because the current update_one
// implementation doesn't store options in the database yet.
// These tests can be uncommented when update_one is enhanced to support options storage.

#[tokio::test]
async fn test_find_one_value_not_found() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();
    let different_property_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "test_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entity with one property
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Try to find a different property that doesn't exist
    let found_value = find_one(&neo4j, space_id, entity_id, different_property_id)
        .send()
        .await
        .expect("Failed to execute find_one query");

    // Should return None for non-existent property
    assert_eq!(found_value, None);
}

#[tokio::test]
async fn test_find_one_value_wrong_space() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let different_space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "test_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entity in one space
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Try to find the value in a different space
    let found_value = find_one(&neo4j, different_space_id, entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query");

    // Should return None when looking in wrong space
    assert_eq!(found_value, None);
}

#[tokio::test]
async fn test_find_one_value_wrong_entity() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data
    let entity_id = Uuid::new_v4();
    let different_entity_id = Uuid::new_v4();
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

    // Try to find the value for a different entity
    let found_value = find_one(&neo4j, space_id, different_entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query");

    // Should return None when looking for wrong entity
    assert_eq!(found_value, None);
}

#[tokio::test]
async fn test_find_one_value_multiple_properties() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data with multiple properties
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property1_id = Uuid::new_v4();
    let property2_id = Uuid::new_v4();
    let property3_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let values = vec![
                Value::new(property1_id, "first_value"),
                Value::new(property2_id, "second_value"),
                Value::new(property3_id, "third_value"),
            ];
            HashMap::from_iter(values.into_iter().map(|v| (v.property, v)))
        },
        embedding: None,
    };

    // Insert the entity with multiple properties
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Find each property individually
    let found_value1 = find_one(&neo4j, space_id, entity_id, property1_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value 1 should exist");

    let found_value2 = find_one(&neo4j, space_id, entity_id, property2_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value 2 should exist");

    let found_value3 = find_one(&neo4j, space_id, entity_id, property3_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value 3 should exist");

    // Verify each found value
    assert_eq!(found_value1.property, property1_id);
    assert_eq!(found_value1.value, "first_value");

    assert_eq!(found_value2.property, property2_id);
    assert_eq!(found_value2.value, "second_value");

    assert_eq!(found_value3.property, property3_id);
    assert_eq!(found_value3.value, "third_value");
}

#[tokio::test]
async fn test_find_one_value_multiple_spaces() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data
    let entity_id = Uuid::new_v4();
    let space1_id = Uuid::new_v4();
    let space2_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Insert the same entity with different values in different spaces
    let entity1 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "value_in_space1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    let entity2 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "value_in_space2");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity1, space1_id)
        .send()
        .await
        .expect("Failed to insert entity in space1");

    update_one(&neo4j, entity2, space2_id)
        .send()
        .await
        .expect("Failed to insert entity in space2");

    // Find the value in each space
    let found_value1 = find_one(&neo4j, space1_id, entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value in space1 should exist");

    let found_value2 = find_one(&neo4j, space2_id, entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value in space2 should exist");

    // Verify each found value corresponds to its space
    assert_eq!(found_value1.property, property_id);
    assert_eq!(found_value1.value, "value_in_space1");

    assert_eq!(found_value2.property, property_id);
    assert_eq!(found_value2.value, "value_in_space2");
}

#[tokio::test]
async fn test_find_one_value_options_always_none() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test data (options are ignored by current update_one implementation)
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let mut value = Value::new(property_id, "Test value");
    value.options = Some(Options::Text(TextOptions {
        language: Some("en".to_string()),
    }));

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let v = value;
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entity with its properties
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Find the value
    let found_value = find_one(&neo4j, space_id, entity_id, property_id)
        .send()
        .await
        .expect("Failed to execute find_one query")
        .expect("Value should exist");

    // Verify the found value - options should always be None with current implementation
    assert_eq!(found_value.property, property_id);
    assert_eq!(found_value.value, "Test value");
    assert_eq!(found_value.options, None); // Options not stored by current update_one
}
