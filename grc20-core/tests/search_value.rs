use std::collections::HashMap;

use futures::TryStreamExt;
use grc20_core::{
    ids::system_ids,
    mapping::{
        entity::{update_one, UpdateEntity},
        value::{
            models::Value,
            search::{search, SemanticSearchQuery, SemanticSearchResult},
        },
    },
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_semantic_search_basic() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities with embeddings
    let entity1_id = Uuid::new_v4();
    let entity2_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();

    // Create entities with similar embeddings (should score well)
    let entity1 = UpdateEntity {
        id: entity1_id,
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "first entity");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![1.0, 0.0, 0.0]), // Close to search vector
    };

    let entity2 = UpdateEntity {
        id: entity2_id,
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "second entity");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![0.0, 1.0, 0.0]), // Different from search vector
    };

    // Insert the entities
    update_one(&neo4j, entity1, space_id)
        .send()
        .await
        .expect("Failed to insert entity1");

    update_one(&neo4j, entity2, space_id)
        .send()
        .await
        .expect("Failed to insert entity2");

    // Perform semantic search with vector similar to entity1
    let search_vector = vec![0.9, 0.1, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return both entities, ordered by similarity score
    assert_eq!(results.len(), 2);

    // First result should be entity1 (closer to search vector)
    assert_eq!(results[0].entity, entity1_id);
    assert_eq!(results[0].value.property, system_ids::NAME_PROPERTY);
    assert_eq!(results[0].value.value, "first entity");
    assert_eq!(results[0].space_id, space_id);
    assert!(results[0].score > results[1].score);

    // Second result should be entity2
    assert_eq!(results[1].entity, entity2_id);
    assert_eq!(results[1].value.property, system_ids::NAME_PROPERTY);
    assert_eq!(results[1].value.value, "second entity");
    assert_eq!(results[1].space_id, space_id);
}

#[tokio::test]
async fn test_semantic_search_with_limit() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create multiple test entities
    let space_id = Uuid::new_v4();

    let entities = vec![
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 1");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![1.0, 0.0, 0.0]),
        },
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 2");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.8, 0.2, 0.0]),
        },
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 3");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.6, 0.4, 0.0]),
        },
    ];

    // Insert all entities
    for entity in entities {
        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Perform semantic search with limit
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .limit(2)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return only 2 results due to limit
    assert_eq!(results.len(), 2);

    // Results should be ordered by similarity score (highest first)
    assert!(results[0].score >= results[1].score);
}

#[tokio::test]
async fn test_semantic_search_with_skip() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create multiple test entities
    let space_id = Uuid::new_v4();

    let entities = vec![
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 1");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![1.0, 0.0, 0.0]),
        },
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 2");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.8, 0.2, 0.0]),
        },
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(system_ids::NAME_PROPERTY, "entity 3");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.6, 0.4, 0.0]),
        },
    ];

    // Insert all entities
    for entity in entities {
        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // First, get all results to establish baseline
    let search_vector = vec![1.0, 0.0, 0.0];
    let all_results: Vec<SemanticSearchResult> = search(&neo4j, search_vector.clone())
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Now get results with skip=1
    let skipped_results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .skip(1)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should have one less result due to skip
    assert_eq!(skipped_results.len(), all_results.len() - 1);

    // First result with skip should match second result without skip
    if all_results.len() > 1 {
        assert_eq!(skipped_results[0].entity, all_results[1].entity);
    }
}

#[tokio::test]
async fn test_semantic_search_no_results() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Perform semantic search on empty database
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return empty results
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_semantic_search_multiple_spaces() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create entities in different spaces
    let space1_id = Uuid::new_v4();
    let space2_id = Uuid::new_v4();

    let entity1 = UpdateEntity {
        id: Uuid::new_v4(),
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "entity in space 1");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![1.0, 0.0, 0.0]),
    };

    let entity2 = UpdateEntity {
        id: Uuid::new_v4(),
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "entity in space 2");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![0.9, 0.1, 0.0]),
    };

    // Insert entities in different spaces
    update_one(&neo4j, entity1, space1_id)
        .send()
        .await
        .expect("Failed to insert entity1");

    update_one(&neo4j, entity2, space2_id)
        .send()
        .await
        .expect("Failed to insert entity2");

    // Perform semantic search
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return entities from both spaces
    assert_eq!(results.len(), 2);

    // Check that both spaces are represented
    let space_ids: std::collections::HashSet<Uuid> = results.iter().map(|r| r.space_id).collect();
    assert!(space_ids.contains(&space1_id));
    assert!(space_ids.contains(&space2_id));
}

#[tokio::test]
async fn test_semantic_search_query_builder() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entity
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = system_ids::NAME_PROPERTY;

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(property_id, "test entity");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![1.0, 0.0, 0.0]),
    };

    // Insert the entity
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Test builder pattern methods
    let search_vector = vec![1.0, 0.0, 0.0];
    let query = SemanticSearchQuery::new(&neo4j, search_vector)
        .limit(10)
        .skip(0);

    let results: Vec<SemanticSearchResult> = query
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity, entity_id);
}

#[tokio::test]
async fn test_semantic_search_score_ordering() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create entities with different similarity to search vector
    let space_id = Uuid::new_v4();
    let property_id = system_ids::NAME_PROPERTY;

    let entities = vec![
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(property_id, "most similar");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![1.0, 0.0, 0.0]),
        },
        // Least similar (should score lowest)
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(property_id, "least similar");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.0, 0.0, 1.0]),
        },
        // Moderately similar (should score in between)
        UpdateEntity {
            id: Uuid::new_v4(),
            values: {
                let v = Value::new(property_id, "moderately similar");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.7, 0.7, 0.0]),
        },
    ];

    // Insert all entities
    for entity in entities {
        update_one(&neo4j, entity, space_id)
            .send()
            .await
            .expect("Failed to insert entity");
    }

    // Perform semantic search with vector closest to first entity
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return all 3 entities
    assert_eq!(results.len(), 3);

    // Results should be ordered by score (highest first)
    assert!(results[0].score >= results[1].score);
    assert!(results[1].score >= results[2].score);

    // First result should be the most similar entity
    assert_eq!(results[0].value.value, "most similar");

    // Last result should be the least similar entity
    assert_eq!(results[2].value.value, "least similar");
}
