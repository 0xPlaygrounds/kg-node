use futures::TryStreamExt;
use std::collections::HashMap;

use grc20_core::mapping::{
    entity::{update_one, UpdateEntity},
    query_utils::ValueFilter,
    relation::{find_many, insert_one, CreateRelation},
    value::models::Value,
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_find_many_relations_no_filters() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities
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

    update_one(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relations
    let relation1_id = Uuid::new_v4();
    let relation2_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation1 = CreateRelation::new(
        relation1_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    );

    let relation2 = CreateRelation::new(
        relation2_id,
        relation_type,
        from_entity_id,
        to_entity_id,
        relation_entity,
    );

    insert_one(&neo4j, relation1)
        .send()
        .await
        .expect("Failed to insert relation1");

    insert_one(&neo4j, relation2)
        .send()
        .await
        .expect("Failed to insert relation2");

    // Find all relations
    let results = find_many(&neo4j)
        .send()
        .await
        .expect("Failed to execute find_many query");

    let relations: Vec<_> = results
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert!(relations.len() >= 2, "Should find at least 2 relations");

    let found_ids: std::collections::HashSet<_> = relations.iter().map(|r| r.id).collect();
    assert!(found_ids.contains(&relation1_id), "Should find relation1");
    assert!(found_ids.contains(&relation2_id), "Should find relation2");
}

#[tokio::test]
async fn test_find_many_relations_with_from_filter() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities
    let from_entity1_id = Uuid::new_v4();
    let from_entity2_id = Uuid::new_v4();
    let to_entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let from_entity1 = UpdateEntity {
        id: from_entity1_id,
        values: {
            let v = Value::new(property_id, "from_value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let from_entity2 = UpdateEntity {
        id: from_entity2_id,
        values: {
            let v = Value::new(property_id, "from_value2");
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

    update_one(&neo4j, from_entity1, space_id)
        .send()
        .await
        .expect("Failed to insert from entity1");

    update_one(&neo4j, from_entity2, space_id)
        .send()
        .await
        .expect("Failed to insert from entity2");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relations
    let relation1_id = Uuid::new_v4();
    let relation2_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation1 = CreateRelation::new(
        relation1_id,
        relation_type,
        from_entity1_id, // Different from entity
        to_entity_id,
        relation_entity,
    );

    let relation2 = CreateRelation::new(
        relation2_id,
        relation_type,
        from_entity2_id, // Different from entity
        to_entity_id,
        relation_entity,
    );

    insert_one(&neo4j, relation1)
        .send()
        .await
        .expect("Failed to insert relation1");

    insert_one(&neo4j, relation2)
        .send()
        .await
        .expect("Failed to insert relation2");

    // Find relations with from filter
    let results = find_many(&neo4j)
        .from(ValueFilter::from(from_entity1_id))
        .send()
        .await
        .expect("Failed to execute find_many query");

    let relations: Vec<_> = results
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(relations.len(), 1, "Should find exactly 1 relation");
    assert_eq!(relations[0].id, relation1_id, "Should find relation1");
    assert_eq!(
        relations[0].from_entity, from_entity1_id,
        "Should have correct from_entity"
    );
}

#[tokio::test]
async fn test_find_many_relations_with_to_filter() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities
    let from_entity_id = Uuid::new_v4();
    let to_entity1_id = Uuid::new_v4();
    let to_entity2_id = Uuid::new_v4();
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
    let to_entity1 = UpdateEntity {
        id: to_entity1_id,
        values: {
            let v = Value::new(property_id, "to_value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let to_entity2 = UpdateEntity {
        id: to_entity2_id,
        values: {
            let v = Value::new(property_id, "to_value2");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };

    update_one(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    update_one(&neo4j, to_entity1, space_id)
        .send()
        .await
        .expect("Failed to insert to entity1");

    update_one(&neo4j, to_entity2, space_id)
        .send()
        .await
        .expect("Failed to insert to entity2");

    // Create test relations
    let relation1_id = Uuid::new_v4();
    let relation2_id = Uuid::new_v4();
    let relation_type = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation1 = CreateRelation::new(
        relation1_id,
        relation_type,
        from_entity_id,
        to_entity1_id, // Different to entity
        relation_entity,
    );

    let relation2 = CreateRelation::new(
        relation2_id,
        relation_type,
        from_entity_id,
        to_entity2_id, // Different to entity
        relation_entity,
    );

    insert_one(&neo4j, relation1)
        .send()
        .await
        .expect("Failed to insert relation1");

    insert_one(&neo4j, relation2)
        .send()
        .await
        .expect("Failed to insert relation2");

    // Find relations with to filter
    let results = find_many(&neo4j)
        .to(ValueFilter::from(to_entity1_id))
        .send()
        .await
        .expect("Failed to execute find_many query");

    let relations: Vec<_> = results
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(relations.len(), 1, "Should find exactly 1 relation");
    assert_eq!(relations[0].id, relation1_id, "Should find relation1");
    assert_eq!(
        relations[0].to_entity, to_entity1_id,
        "Should have correct to_entity"
    );
}

#[tokio::test]
async fn test_find_many_relations_with_type_filter() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities
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

    update_one(&neo4j, from_entity, space_id)
        .send()
        .await
        .expect("Failed to insert from entity");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relations with different types
    let relation1_id = Uuid::new_v4();
    let relation2_id = Uuid::new_v4();
    let relation_type1 = Uuid::new_v4();
    let relation_type2 = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation1 = CreateRelation::new(
        relation1_id,
        relation_type1, // Different type
        from_entity_id,
        to_entity_id,
        relation_entity,
    );

    let relation2 = CreateRelation::new(
        relation2_id,
        relation_type2, // Different type
        from_entity_id,
        to_entity_id,
        relation_entity,
    );

    insert_one(&neo4j, relation1)
        .send()
        .await
        .expect("Failed to insert relation1");

    insert_one(&neo4j, relation2)
        .send()
        .await
        .expect("Failed to insert relation2");

    // Find relations with type filter
    let results = find_many(&neo4j)
        .r#type(relation_type1)
        .send()
        .await
        .expect("Failed to execute find_many query");

    let relations: Vec<_> = results
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(relations.len(), 1, "Should find exactly 1 relation");
    assert_eq!(relations[0].id, relation1_id, "Should find relation1");
    assert_eq!(
        relations[0].r#type, relation_type1,
        "Should have correct type"
    );
}

#[tokio::test]
async fn test_find_many_relations_with_combined_filters() {
    let (neo4j, _container) = common::setup_neo4j_container().await;

    // Create test entities
    let from_entity1_id = Uuid::new_v4();
    let from_entity2_id = Uuid::new_v4();
    let to_entity_id = Uuid::new_v4();
    let space_id = Uuid::new_v4();
    let property_id = Uuid::new_v4();

    let from_entity1 = UpdateEntity {
        id: from_entity1_id,
        values: {
            let v = Value::new(property_id, "from_value1");
            HashMap::from([(v.property, v)])
        },
        embedding: None,
    };
    let from_entity2 = UpdateEntity {
        id: from_entity2_id,
        values: {
            let v = Value::new(property_id, "from_value2");
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

    update_one(&neo4j, from_entity1, space_id)
        .send()
        .await
        .expect("Failed to insert from entity1");

    update_one(&neo4j, from_entity2, space_id)
        .send()
        .await
        .expect("Failed to insert from entity2");

    update_one(&neo4j, to_entity, space_id)
        .send()
        .await
        .expect("Failed to insert to entity");

    // Create test relations
    let relation1_id = Uuid::new_v4();
    let relation2_id = Uuid::new_v4();
    let relation_type1 = Uuid::new_v4();
    let relation_type2 = Uuid::new_v4();
    let relation_entity = Uuid::new_v4();

    let relation1 = CreateRelation::new(
        relation1_id,
        relation_type1,
        from_entity1_id, // Match this
        to_entity_id,
        relation_entity,
    );

    let relation2 = CreateRelation::new(
        relation2_id,
        relation_type2, // Different type - should not match
        from_entity1_id,
        to_entity_id,
        relation_entity,
    );

    insert_one(&neo4j, relation1)
        .send()
        .await
        .expect("Failed to insert relation1");

    insert_one(&neo4j, relation2)
        .send()
        .await
        .expect("Failed to insert relation2");

    // Find relations with both from and type filters
    let results = find_many(&neo4j)
        .from(ValueFilter::from(from_entity1_id))
        .r#type(relation_type1)
        .send()
        .await
        .expect("Failed to execute find_many query");

    let relations: Vec<_> = results
        .try_collect()
        .await
        .expect("Failed to collect results");

    assert_eq!(relations.len(), 1, "Should find exactly 1 relation");
    assert_eq!(relations[0].id, relation1_id, "Should find relation1");
    assert_eq!(
        relations[0].from_entity, from_entity1_id,
        "Should have correct from_entity"
    );
    assert_eq!(
        relations[0].r#type, relation_type1,
        "Should have correct type"
    );
}
