use std::collections::HashMap;

use futures::TryStreamExt;
use grc20_core::{
    ids::system_ids,
    mapping::{
        entity::{search, update_one, SemanticSearchQuery, SemanticSearchResult, UpdateEntity},
        query_utils::Subquery,
        value::models::Value,
    },
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_semantic_search_entity_basic() {
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
    assert_eq!(results[0].entity_id, entity1_id);
    assert_eq!(results[0].spaces, vec![space_id]);
    assert!(results[0].score > results[1].score);

    // Second result should be entity2
    assert_eq!(results[1].entity_id, entity2_id);
    assert_eq!(results[1].spaces, vec![space_id]);

    // Test conversion to Entity
    let (entity, score) = results[0].clone().into_entity_with_score();
    assert_eq!(entity.id, entity1_id);
    assert_eq!(entity.values.len(), 1);
    let space_values = entity.values.get(&space_id).expect("Space should exist");
    assert_eq!(space_values.len(), 1);
    assert_eq!(
        space_values[&system_ids::NAME_PROPERTY].value,
        "first entity"
    );
    assert_eq!(score, results[0].score);
}

#[tokio::test]
async fn test_semantic_search_entity_with_limit() {
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
async fn test_semantic_search_entity_with_skip() {
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
        assert_eq!(skipped_results[0].entity_id, all_results[1].entity_id);
    }
}

#[tokio::test]
async fn test_semantic_search_entity_no_results() {
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
async fn test_semantic_search_entity_multiple_spaces() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create entity with properties in multiple spaces
    let entity_id = Uuid::new_v4();
    let space1_id = Uuid::new_v4();
    let space2_id = Uuid::new_v4();

    // Insert entity with properties in first space
    let entity1 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "entity in space 1");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![1.0, 0.0, 0.0]),
    };

    update_one(&neo4j, entity1, space1_id)
        .send()
        .await
        .expect("Failed to insert entity in space1");

    // Insert entity with properties in second space
    let entity2 = UpdateEntity {
        id: entity_id,
        values: {
            let v = Value::new(system_ids::NAME_PROPERTY, "entity in space 2");
            HashMap::from([(v.property, v)])
        },
        embedding: Some(vec![1.0, 0.0, 0.0]),
    };

    update_one(&neo4j, entity2, space2_id)
        .send()
        .await
        .expect("Failed to insert entity in space2");

    // Perform semantic search
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return one result with both spaces
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, entity_id);
    assert_eq!(results[0].spaces.len(), 2);
    assert!(results[0].spaces.contains(&space1_id));
    assert!(results[0].spaces.contains(&space2_id));

    // Convert to Entity and verify both spaces are present
    let entity = results[0].clone().into_entity();
    assert_eq!(entity.id, entity_id);
    assert_eq!(entity.values.len(), 2);
    assert!(entity.values.contains_key(&space1_id));
    assert!(entity.values.contains_key(&space2_id));
}

#[tokio::test]
async fn test_semantic_search_entity_multiple_properties() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create entity with multiple properties
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property1_id = Uuid::new_v4();
    let property2_id = Uuid::new_v4();

    let entity = UpdateEntity {
        id: entity_id,
        values: {
            let values = vec![
                Value::new(property1_id, "first property"),
                Value::new(property2_id, "second property"),
            ];
            HashMap::from_iter(values.into_iter().map(|v| (v.property, v)))
        },
        embedding: Some(vec![1.0, 0.0, 0.0]),
    };

    // Insert the entity
    update_one(&neo4j, entity, space_id)
        .send()
        .await
        .expect("Failed to insert entity");

    // Perform semantic search
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    // Should return one result for the entity
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, entity_id);

    // Convert to Entity and verify both properties are present
    let entity = results[0].clone().into_entity();
    assert_eq!(entity.id, entity_id);
    assert_eq!(entity.values.len(), 1);
    let space_values = entity.values.get(&space_id).expect("Space should exist");
    assert_eq!(space_values.len(), 2);

    // Check that both properties are represented
    let property_ids: std::collections::HashSet<Uuid> =
        space_values.iter().map(|(_, v)| v.property).collect();
    assert!(property_ids.contains(&property1_id));
    assert!(property_ids.contains(&property2_id));
}

#[tokio::test]
async fn test_semantic_search_entity_query_builder() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entity
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

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
    assert_eq!(results[0].entity_id, entity_id);
}

#[tokio::test]
async fn test_semantic_search_entity_score_ordering() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create entities with different similarity to search vector
    let space_id = Uuid::new_v4();
    let property_id = system_ids::NAME_PROPERTY;

    let most_similar_id = Uuid::new_v4();
    let least_similar_id = Uuid::new_v4();
    let moderately_similar_id = Uuid::new_v4();

    let entities = vec![
        // Most similar (should score highest)
        UpdateEntity {
            id: most_similar_id,
            values: {
                let v = Value::new(property_id, "most similar");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![1.0, 0.0, 0.0]),
        },
        // Least similar (should score lowest)
        UpdateEntity {
            id: least_similar_id,
            values: {
                let v = Value::new(property_id, "least similar");
                HashMap::from([(v.property, v)])
            },
            embedding: Some(vec![0.0, 0.0, 1.0]),
        },
        // Moderately similar (should score in between)
        UpdateEntity {
            id: moderately_similar_id,
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
    assert_eq!(results[0].entity_id, most_similar_id);
    let first_entity = results[0].clone().into_entity();
    let first_space_values = first_entity
        .values
        .get(&space_id)
        .expect("Space should exist");
    assert_eq!(first_space_values[&property_id].value, "most similar");

    // Last result should be the least similar entity
    assert_eq!(results[2].entity_id, least_similar_id);
    let last_entity = results[2].clone().into_entity();
    let last_space_values = last_entity
        .values
        .get(&space_id)
        .expect("Space should exist");
    assert_eq!(last_space_values[&property_id].value, "least similar");
}

#[tokio::test]
async fn test_semantic_search_entity_conversion_methods() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entity
    let entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

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

    // Perform semantic search
    let search_vector = vec![1.0, 0.0, 0.0];
    let results: Vec<SemanticSearchResult> = search(&neo4j, search_vector)
        .send()
        .await
        .expect("Failed to execute search")
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(results.len(), 1);
    let result = &results[0];

    // Test into_entity_with_score()
    let (entity_with_score, score) = result.clone().into_entity_with_score();
    assert_eq!(entity_with_score.id, entity_id);
    assert_eq!(score, result.score);

    // Test into_entity()
    let entity_only = result.clone().into_entity();
    assert_eq!(entity_only.id, entity_id);
    assert_eq!(entity_only.values, entity_with_score.values);
}

#[tokio::test]
async fn test_semantic_search_query_with_traversals() {
    use grc20_core::entity::utils::RelationTraversal;
    use grc20_core::relation::utils::RelationDirection;

    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create a sample search vector
    let search_vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    // Create traversals
    let traversal1 = RelationTraversal::default()
        .relation_type_id(uuid::Uuid::new_v4())
        .direction(RelationDirection::From);
    let traversal2 = RelationTraversal::default()
        .relation_type_id(uuid::Uuid::new_v4())
        .direction(RelationDirection::To);

    // Build the query with traversals
    let query = SemanticSearchQuery::new(&neo4j, search_vector)
        .limit(50)
        .skip(10)
        .threshold(0.8)
        .traversal(traversal1)
        .traversal(traversal2);

    println!("{}", query.subquery().compile());
}
