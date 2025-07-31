use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{update_one, UpdateEntity},
    relation::{insert_one, CreateRelation},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_insert_one_relation() {
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
    update_one(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relation
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
    .position("test_position")
    .verified(true);

    // Insert the relation
    insert_one(&neo4j, relation)
        .send()
        .await
        .expect("Failed to insert relation");

    // Verify the relation was created
    let relation_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (from:Entity {id: $from_id})-[r:RELATION]->(to:Entity {id: $to_id})
                 RETURN r.id as id, r.type as type, r.entity as entity, r.position as position, r.verified as verified"
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
    let retrieved_type: String = relation_result.get("type").unwrap();
    let retrieved_entity: String = relation_result.get("entity").unwrap();
    let retrieved_position: String = relation_result.get("position").unwrap();
    let retrieved_verified: bool = relation_result.get("verified").unwrap();

    assert_eq!(retrieved_id, relation_id.to_string());
    assert_eq!(retrieved_type, relation_type.to_string());
    assert_eq!(retrieved_entity, relation_entity.to_string());
    assert_eq!(retrieved_position, "test_position");
    assert_eq!(retrieved_verified, true);
}

#[tokio::test]
async fn test_insert_one_relation_with_optional_fields() {
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
    update_one(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relation with all optional fields
    let relation_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();
    let from_space = Uuid::new_v4();
    let from_version = Uuid::new_v4();
    let to_space = Uuid::new_v4();
    let to_version = Uuid::new_v4();

    let relation = CreateRelation::new(
        relation_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    )
    .from_space(from_space)
    .from_version(from_version)
    .to_space(to_space)
    .to_version(to_version)
    .position("complex_position")
    .verified(false);

    // Insert the relation
    insert_one(&neo4j, relation)
        .send()
        .await
        .expect("Failed to insert relation");

    // Verify the relation was created with all fields
    let relation_result = neo4j
        .execute(
            neo4rs::query(
                "MATCH (from:Entity {id: $from_id})-[r:RELATION]->(to:Entity {id: $to_id})
                 RETURN r.id as id, r.from_space as from_space, r.from_version as from_version,
                        r.to_space as to_space, r.to_version as to_version, r.verified as verified",
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
    let retrieved_from_version: String = relation_result.get("from_version").unwrap();
    let retrieved_to_space: String = relation_result.get("to_space").unwrap();
    let retrieved_to_version: String = relation_result.get("to_version").unwrap();
    let retrieved_verified: bool = relation_result.get("verified").unwrap();

    assert_eq!(retrieved_id, relation_id.to_string());
    assert_eq!(retrieved_from_space, from_space.to_string());
    assert_eq!(retrieved_from_version, from_version.to_string());
    assert_eq!(retrieved_to_space, to_space.to_string());
    assert_eq!(retrieved_to_version, to_version.to_string());
    assert_eq!(retrieved_verified, false);
}
