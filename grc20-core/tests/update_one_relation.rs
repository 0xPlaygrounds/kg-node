use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{update_one as insert_entity, UpdateEntity},
    relation::{insert_one, update_one, CreateRelation, UnsetRelationFields, UpdateRelation},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_update_one_relation() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities first
    let from_entity_id = Uuid::new_v4();
    let to_entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let from_entity = UpdateEntity {
        id: from_entity_id,
        values: {
            let v = Value::new(property_id, "from_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let to_entity = UpdateEntity {
        id: to_entity_id,
        values: {
            let v = Value::new(property_id, "to_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entities
    insert_entity(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    insert_entity(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create and insert initial relation
    let relation_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation = CreateRelation::new(
        relation_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    )
    .position("initial_position")
    .verified(false);

    insert_one(&neo4j, relation)
        .send()
        .await
        .expect("Failed to insert relation");

    // Create update for the relation
    let new_from_space = Uuid::new_v4();
    let new_to_version = Uuid::new_v4();

    let update_relation = UpdateRelation {
        id: relation_id,
        from_space: Some(new_from_space),
        from_version: None,
        to_space: None,
        to_version: Some(new_to_version),
        position: Some("updated_position".to_string()),
        verified: Some(true),
    };

    // Update the relation
    update_one(&neo4j, update_relation)
        .send()
        .await
        .expect("Failed to update relation");

    // Verify the relation was updated
    let relation_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (from:Entity {id: $from_id})-[r:RELATION]->(to:Entity {id: $to_id})
                 RETURN r.id as id, r.from_space as from_space, r.to_version as to_version,
                        r.position as position, r.verified as verified",
            )
            .param("from_id", from_entity_id.to_string())
            .param("to_id", to_entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_id: String = relation_result.get("id").unwrap();
    let retrieved_from_space: String = relation_result.get("from_space").unwrap();
    let retrieved_to_version: String = relation_result.get("to_version").unwrap();
    let retrieved_position: String = relation_result.get("position").unwrap();
    let retrieved_verified: bool = relation_result.get("verified").unwrap();

    assert_eq!(retrieved_id, relation_id.to_string());
    assert_eq!(retrieved_from_space, new_from_space.to_string());
    assert_eq!(retrieved_to_version, new_to_version.to_string());
    assert_eq!(retrieved_position, "updated_position");
    assert_eq!(retrieved_verified, true);
}

#[tokio::test]
async fn test_update_one_relation_partial() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities first
    let from_entity_id = Uuid::new_v4();
    let to_entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let from_entity = UpdateEntity {
        id: from_entity_id,
        values: {
            let v = Value::new(property_id, "from_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let to_entity = UpdateEntity {
        id: to_entity_id,
        values: {
            let v = Value::new(property_id, "to_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entities
    insert_entity(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    insert_entity(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create and insert initial relation with all fields
    let relation_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();
    let initial_from_space = Uuid::new_v4();
    let initial_to_space = Uuid::new_v4();

    let relation = CreateRelation::new(
        relation_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    )
    .from_space(initial_from_space)
    .to_space(initial_to_space)
    .position("initial_position")
    .verified(false);

    insert_one(&neo4j, relation)
        .send()
        .await
        .expect("Failed to insert relation");

    // Create partial update (only update position)
    let update_relation = UpdateRelation {
        id: relation_id,
        from_space: None,
        from_version: None,
        to_space: None,
        to_version: None,
        position: Some("partially_updated_position".to_string()),
        verified: None,
    };

    // Update the relation
    update_one(&neo4j, update_relation)
        .send()
        .await
        .expect("Failed to update relation");

    // Verify only position was updated, other fields remain unchanged
    let relation_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (from:Entity {id: $from_id})-[r:RELATION]->(to:Entity {id: $to_id})
                 RETURN r.id as id, r.from_space as from_space, r.to_space as to_space,
                        r.position as position, r.verified as verified",
            )
            .param("from_id", from_entity_id.to_string())
            .param("to_id", to_entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_id: String = relation_result.get("id").unwrap();
    let retrieved_from_space: String = relation_result.get("from_space").unwrap();
    let retrieved_to_space: String = relation_result.get("to_space").unwrap();
    let retrieved_position: String = relation_result.get("position").unwrap();
    let retrieved_verified: bool = relation_result.get("verified").unwrap();

    // Check that only position was updated
    assert_eq!(retrieved_id, relation_id.to_string());
    assert_eq!(retrieved_from_space, initial_from_space.to_string()); // Unchanged
    assert_eq!(retrieved_to_space, initial_to_space.to_string()); // Unchanged
    assert_eq!(retrieved_position, "partially_updated_position"); // Updated
    assert_eq!(retrieved_verified, false); // Unchanged
}

#[tokio::test]
async fn test_unset_relation_fields() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities first
    let from_entity_id = Uuid::new_v4();
    let to_entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let from_entity = UpdateEntity {
        id: from_entity_id,
        values: {
            let v = Value::new(property_id, "from_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let to_entity = UpdateEntity {
        id: to_entity_id,
        values: {
            let v = Value::new(property_id, "to_value");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    // Insert the entities
    insert_entity(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    insert_entity(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create and insert initial relation with all optional fields set
    let relation_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();
    let initial_from_space = Uuid::new_v4();
    let initial_from_version = Uuid::new_v4();
    let initial_to_space = Uuid::new_v4();
    let initial_to_version = Uuid::new_v4();

    let relation = CreateRelation::new(
        relation_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    )
    .from_space(initial_from_space)
    .from_version(initial_from_version)
    .to_space(initial_to_space)
    .to_version(initial_to_version)
    .position("initial_position")
    .verified(true);

    insert_one(&neo4j, relation)
        .send()
        .await
        .expect("Failed to insert relation");

    // Create unset operation (unset from_space, position, and verified)
    let unset_fields = UnsetRelationFields {
        id: relation_id,
        from_space: Some(true),    // Unset this field
        from_version: Some(false), // Keep this field
        to_space: None,            // Keep this field (None means don't change)
        to_version: Some(false),   // Keep this field
        position: Some(true),      // Unset this field
        verified: Some(true),      // Unset this field
    };

    // Apply the unset operation
    update_one(&neo4j, unset_fields)
        .send()
        .await
        .expect("Failed to unset relation fields");

    // Verify the fields were unset correctly
    let relation_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (from:Entity {id: $from_id})-[r:RELATION]->(to:Entity {id: $to_id})
                 RETURN r.id as id, r.from_space as from_space, r.from_version as from_version,
                        r.to_space as to_space, r.to_version as to_version,
                        r.position as position, r.verified as verified",
            )
            .param("from_id", from_entity_id.to_string())
            .param("to_id", to_entity_id.to_string()),
        )
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    let retrieved_id: String = relation_result.get("id").unwrap();
    let retrieved_from_space: Option<String> = relation_result.get("from_space").ok();
    let retrieved_from_version: String = relation_result.get("from_version").unwrap();
    let retrieved_to_space: String = relation_result.get("to_space").unwrap();
    let retrieved_to_version: String = relation_result.get("to_version").unwrap();
    let retrieved_position: Option<String> = relation_result.get("position").ok();
    let retrieved_verified: Option<bool> = relation_result.get("verified").ok();

    // Verify the results
    assert_eq!(retrieved_id, relation_id.to_string());
    assert_eq!(retrieved_from_space, None); // Should be unset (null)
    assert_eq!(retrieved_from_version, initial_from_version.to_string()); // Should remain
    assert_eq!(retrieved_to_space, initial_to_space.to_string()); // Should remain
    assert_eq!(retrieved_to_version, initial_to_version.to_string()); // Should remain
    assert_eq!(retrieved_position, None); // Should be unset (null)
    assert_eq!(retrieved_verified, None); // Should be unset (null)
}
