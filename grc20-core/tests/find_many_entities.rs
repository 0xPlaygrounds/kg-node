use std::collections::HashMap;

use futures::TryStreamExt;
use grc20_core::mapping::{
    entity::{find_many, update_one, UpdateEntity},
    query_utils::{asc, value_filter, PropertyFilter},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_find_many_entities_basic() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity1_id = Uuid::new_v4();
    let entity2_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Create two test entities
    let entity1 = UpdateEntity {
        id: entity1_id,
        values: {
            let v = Value::new(property_id, "value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    let entity2 = UpdateEntity {
        id: entity2_id,
        values: {
            let v = Value::new(property_id, "value2");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity1, space_id)
        .send()
        .await
        .expect("Failed to insert entity1");

    update_one(&neo4j, entity2, space_id)
        .send()
        .await
        .expect("Failed to insert entity2");

    // Find all entities
    let results: Vec<_> = find_many(&neo4j)
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 2);
    let mut found_ids = results.iter().map(|e| e.id).collect::<Vec<_>>();
    found_ids.sort();
    let mut expected_ids = vec![entity1_id, entity2_id];
    expected_ids.sort();
    assert_eq!(found_ids, expected_ids);
}

#[tokio::test]
async fn test_find_many_entities_with_limit() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Create three test entities
    for i in 0..3 {
        let entity_id = Uuid::new_v4();
        let entity = UpdateEntity {
            id: entity_id,
            values: {
                let v = Value::new(property_id, format!("value{}", i));
                HashMap::from([(v.property, v)])
            },
            embedding: None,
        };

        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Find entities with limit of 2
    let results: Vec<_> = find_many(&neo4j)
        .limit(2)
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_find_many_entities_with_skip() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Create three test entities
    for i in 0..3 {
        let entity_id = Uuid::new_v4();
        let entity = UpdateEntity {
            id: entity_id,
            values: {
                let v = Value::new(property_id, format!("value{}", i));
                HashMap::from([(v.property, v)])
            },
            embedding: None,
        };

        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Find entities with skip of 1
    let results: Vec<_> = find_many(&neo4j)
        .skip(1)
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_find_many_entities_with_id_filter() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity1_id = Uuid::new_v4();
    let entity2_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Create two test entities
    let entity1 = UpdateEntity {
        id: entity1_id,
        values: {
            let v = Value::new(property_id, "value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    let entity2 = UpdateEntity {
        id: entity2_id,
        values: {
            let v = Value::new(property_id, "value2");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity1, space_id)
        .send()
        .await
        .expect("Failed to insert entity1");

    update_one(&neo4j, entity2, space_id)
        .send()
        .await
        .expect("Failed to insert entity2");

    // Find entities with ID filter
    let results: Vec<_> = find_many(&neo4j)
        .id(value_filter::value(entity1_id))
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, entity1_id);
}

#[tokio::test]
async fn test_find_many_entities_with_property_filter() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity1_id = Uuid::new_v4();
    let entity2_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    // Create two test entities with different values
    let entity1 = UpdateEntity {
        id: entity1_id,
        values: {
            let v = Value::new(property_id, "target_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    let entity2 = UpdateEntity {
        id: entity2_id,
        values: {
            let v = Value::new(property_id, "other_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, entity1, space_id)
        .send()
        .await
        .expect("Failed to insert entity1");

    update_one(&neo4j, entity2, space_id)
        .send()
        .await
        .expect("Failed to insert entity2");

    // Find entities with property filter
    let results: Vec<_> = find_many(&neo4j)
        .property(PropertyFilter::new(property_id).value("target_value"))
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, entity1_id);
}

#[tokio::test]
async fn test_find_many_entities_with_multiple_attributes() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property1_id = Uuid::new_v4();
    let property2_id = Uuid::new_v4();

    // Create entity with multiple properties
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

    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Find entities with multiple attribute filters
    let results: Vec<_> = find_many(&neo4j)
        .properties(vec![
            PropertyFilter::new(property1_id).value("value1"),
            PropertyFilter::new(property2_id).value("value2"),
        ])
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, entity_id);
}

#[tokio::test]
async fn test_find_many_entities_with_order_by() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();
    let mut entity_ids = Vec::new();

    // Create entities with ordered values
    for i in 0..3 {
        let entity_id = Uuid::new_v4();
        let entity = UpdateEntity {
            id: entity_id,
            values: {
                let v = Value::new(property_id, format!("value{}", i));
                HashMap::from([(v.property, v)])
            },
            embedding: None,
        };

        entity_ids.push(entity_id);
        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Find entities ordered by ID ascending
    let results: Vec<_> = find_many(&neo4j)
        .order_by(asc(property_id))
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 3);

    // Verify order (should be sorted by ID)
    let result_ids: Vec<Uuid> = results.iter().map(|e| e.id).collect();
    assert_eq!(result_ids, entity_ids);
}

#[tokio::test]
async fn test_find_many_entities_empty_result() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Find entities when none exist
    let results: Vec<_> = find_many(&neo4j)
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_find_many_entities_complex_query() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();
    let target_entity_id = Uuid::new_v4();

    // Create multiple entities
    for i in 0..5 {
        let entity_id = if i == 2 {
            target_entity_id
        } else {
            Uuid::new_v4()
        };
        let entity = UpdateEntity {
            id: entity_id,
            values: {
                let v = Value::new(property_id, format!("value{}", i));
                HashMap::from([(v.property, v)])
            },
            embedding: None,
        };

        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Complex query: skip 1, limit 2, with property filter, ordered
    let results: Vec<_> = find_many(&neo4j)
        .property(
            PropertyFilter::new(property_id).value(value_filter::value_in(vec![
                "value1".to_string(),
                "value2".to_string(),
            ])),
        )
        .skip(1)
        .limit(2)
        .order_by(asc(property_id))
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_find_many_entities_multiple_spaces() {
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

    // Find entities
    let results: Vec<_> = find_many(&neo4j)
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    let entity = &results[0];
    assert_eq!(entity.id, entity_id);
    assert_eq!(entity.values.len(), 2);

    // Verify both spaces are present
    assert!(entity.values.contains_key(&space1_id));
    assert!(entity.values.contains_key(&space2_id));
}

#[tokio::test]
async fn test_find_many_entities_builder_methods() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

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

    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Test mutable methods
    let mut query = find_many(&neo4j);
    query.property_mut(PropertyFilter::new(property_id).value("test_value"));
    query.order_by_mut(asc(property_id));
    query.properties_mut(vec![]); // Empty additional attributes

    let results: Vec<_> = query
        .send()
        .await
        .expect("Failed to execute find query")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, entity_id);
}
