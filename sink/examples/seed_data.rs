use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use grc20_core::{
    block::BlockMetadata,
    entity::EntityNodeRef,
    ids,
    mapping::{triple, Query, RelationEdge, Triple},
    neo4rs, relation, system_ids,
};

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

const NEO4J_URL: &str = "bolt://localhost:7687";
const NEO4J_USER: &str = "neo4j";
const NEO4J_PASSWORD: &str = "password";

const DEFAULT_VERSION: &str = "0";

const EVENT_TYPE: &str = "LmVu35JFfyGW2B4TCkRq5r";
const CITY_TYPE: &str = "7iULQxoxfxMXxhccYmWJVZ";
const EVENT_LOCATION_PROP: &str = "5hJcLH7zd6auNs8br859UJ";
const SPEAKERS_PROP: &str = "6jVaNgq31A8eAHQ6iBm6aG";
const RUSTCONF_2023: &str = "WNaUUp4WdPJtdnchrSxQYA";
const JSCONF_2024: &str = "L6rgWLHrUxgME5ZTi3WWVx";
const ALICE_ID: &str = "QGGFVgMWJGQCPLpme8iCdZ";
const BOB_ID: &str = "SQmjDM5WrfPNafdpFPFtno";
const CAROL_ID: &str = "BsiZXi6G9QpyZ47Eq87iSE";
const DAVE_ID: &str = "8a2MNSg4myMVXXpXnE2Yti";
const SAN_FRANCISCO_ID: &str = "2tvbXLHW1GCkE1LvgQFMLF";
const NEW_YORK_ID: &str = "FEiviAcKw5jkNH75vBoJ44";
const SIDE_EVENTS: &str = "As4CaMsDuGLqpRCVyjuYAN";
const RUST_ASYNC_WORKSHOP_SIDEEVENT: &str = "QPZnckrRUebWjdwQZTR7Ka";
const RUST_HACKATHON_SIDEEVENT: &str = "ReJ5RRMqTer9qfr87Yjexp";
const JOE_ID: &str = "MpR7wuVWyXV988F5NWZ21r";
const CHRIS_ID: &str = "ScHYh4PpRpyuvY2Ab4Znf5";
const _: &str = "Mu7ddiBnwZH1LvpDTpKcvq";
const _: &str = "DVurPdLUZi7Ajfv9BC3ADm";
const _: &str = "MPxRvh35rnDeRJNEJLU1YF";
const _: &str = "JjoWPp8LiCKVZiWtE5iZaJ";
const _: &str = "8bCuTuWqL3dxALLff1Awdb";
const _: &str = "9Bj46RXQzHQq25WNPY4Lw";
const _: &str = "RkTkM28NSx3WZuW33vZUjx";
const _: &str = "Lc9L7StPfXMFGWw45utaTY";
const _: &str = "G49gECRJmW6BwqHaENF5nS";
const _: &str = "GfugZRvoWmQhkjMcFJHg49";
const _: &str = "5bwj7yNukCHoJnW8ksgZY";
const _: &str = "GKXfCXBAJ2oAufgETPcFK7";
const _: &str = "X6q73SFySo5u2BuQrYUxR5";
const _: &str = "S2etHTe7W92QbXz32QWimW";
const _: &str = "UV2buTZhfviv7CYTR41APA";
const _: &str = "2ASGaR78dDZAiXM1oeLgDp";
const _: &str = "9EKE5gNaCCb1sMF8BZoGvU";
const _: &str = "TTbAuVjFb9TLsvMjtRJpKi";
const _: &str = "HJDgxUcnjzvWhjX9r3zNua";
const _: &str = "2FySkRW5LnWaf2dN4i214o";
const _: &str = "Em2QUUXS7HDaCGtQ2h5YVc";
const _: &str = "CdPyBWaMAmCUmyutWoVStQ";
const _: &str = "L3xF6a8gbxxVRoCyBs373N";
const _: &str = "WE4GbaJ1eHtQZaG516Pb9j";
const _: &str = "J7ocdxruhsZHBjVGZbPbZJ";
const _: &str = "3QCECHDBpVjd3ZSNYVRUsW";
const _: &str = "CWesNo9yeRdNaKKk8LGoxr";
const _: &str = "DeWmJcSYrxKQ794BgphfmS";
const _: &str = "JCf7JGmhXog1swmX7JVV";
const _: &str = "NmGh6yGqFuHw3F885SHeJj";
const _: &str = "8EjgLrZYP9pzhpzqf82T99";
const _: &str = "7df1NGiRjFtVGVwaDZTPPC";
const _: &str = "YyATjD7HyDrVq4SKkQGBu";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let neo4j = neo4rs::Graph::new(NEO4J_URL, NEO4J_USER, NEO4J_PASSWORD)
        .await
        .expect("Failed to connect to Neo4j");

    let embedding_model = TextEmbedding::try_new(
        InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
    )?;

    // Reset and bootstrap the database
    reset_db(&neo4j).await?;
    bootstrap(&neo4j, &embedding_model).await?;

    // Create some common types
    create_type(
        &neo4j,
        &embedding_model,
        "Person",
        [],
        [
            system_ids::NAME_ATTRIBUTE,
            system_ids::DESCRIPTION_ATTRIBUTE,
        ],
        Some(system_ids::PERSON_TYPE),
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "Event",
        [],
        [
            system_ids::NAME_ATTRIBUTE,
            system_ids::DESCRIPTION_ATTRIBUTE,
        ],
        Some(EVENT_TYPE),
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "City",
        [],
        [
            system_ids::NAME_ATTRIBUTE,
            system_ids::DESCRIPTION_ATTRIBUTE,
        ],
        Some(CITY_TYPE),
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Event location",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(CITY_TYPE),
        Some(EVENT_LOCATION_PROP),
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Speakers",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::PERSON_TYPE),
        Some(SPEAKERS_PROP),
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Side events",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(EVENT_TYPE),
        Some(SIDE_EVENTS),
    )
    .await?;

    // Create person entities
    create_entity(
        &neo4j,
        &embedding_model,
        "Alice",
        None,
        [system_ids::PERSON_TYPE],
        [
            (system_ids::NAME_ATTRIBUTE, "Alice"),
            (
                system_ids::DESCRIPTION_ATTRIBUTE,
                "Speaker at Rust Conference 2023",
            ),
        ],
        [],
        Some(ALICE_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Bob",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(BOB_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Carol",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(CAROL_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Dave",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(DAVE_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Joe",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(JOE_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Chris",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(CHRIS_ID),
    )
    .await?;

    // Create city entities
    create_entity(
        &neo4j,
        &embedding_model,
        "San Francisco",
        Some("City in California"),
        [CITY_TYPE],
        [],
        [],
        Some(SAN_FRANCISCO_ID),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "New York",
        Some("City in New York State"),
        [CITY_TYPE],
        [],
        [],
        Some(NEW_YORK_ID),
    )
    .await?;

    // Create events entities
    // Create side event entities for RustConf 2023
    create_entity(
        &neo4j,
        &embedding_model,
        "Rust Async Workshop",
        Some("A hands-on workshop about async programming in Rust"),
        [EVENT_TYPE],
        [],
        [
            (EVENT_LOCATION_PROP, SAN_FRANCISCO_ID),
            (SPEAKERS_PROP, JOE_ID),
        ],
        Some(RUST_ASYNC_WORKSHOP_SIDEEVENT),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "RustConf Hackathon",
        Some("A hackathon for RustConf 2023 attendees"),
        [EVENT_TYPE],
        [],
        [
            (EVENT_LOCATION_PROP, SAN_FRANCISCO_ID),
            (SPEAKERS_PROP, CHRIS_ID),
        ],
        Some(RUST_HACKATHON_SIDEEVENT),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Rust Conference 2023",
        Some("A conference about Rust programming language"),
        [EVENT_TYPE],
        [],
        [
            (SPEAKERS_PROP, ALICE_ID),                    // Alice
            (SPEAKERS_PROP, BOB_ID),                      // Bob
            (EVENT_LOCATION_PROP, SAN_FRANCISCO_ID),      // San Francisco
            (SIDE_EVENTS, RUST_ASYNC_WORKSHOP_SIDEEVENT), // Rust Async Workshop
            (SIDE_EVENTS, RUST_HACKATHON_SIDEEVENT),      // RustConf Hackathon
        ],
        Some(RUSTCONF_2023),
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "JavaScript Summit 2024",
        Some("A summit for JavaScript enthusiasts and professionals"),
        [EVENT_TYPE],
        [],
        [
            (SPEAKERS_PROP, CAROL_ID),          // Carol
            (SPEAKERS_PROP, DAVE_ID),           // Dave
            (EVENT_LOCATION_PROP, NEW_YORK_ID), // New York
        ],
        Some(JSCONF_2024),
    )
    .await?;

    Ok(())
}

pub async fn bootstrap(
    neo4j: &neo4rs::Graph,
    embedding_model: &TextEmbedding,
) -> anyhow::Result<()> {
    let triples = vec![
        // Value types
        Triple::new(system_ids::CHECKBOX, system_ids::NAME_ATTRIBUTE, "Checkbox"),
        Triple::new(system_ids::TIME, system_ids::NAME_ATTRIBUTE, "Time"),
        Triple::new(system_ids::TEXT, system_ids::NAME_ATTRIBUTE, "Text"),
        Triple::new(system_ids::URL, system_ids::NAME_ATTRIBUTE, "Url"),
        Triple::new(system_ids::NUMBER, system_ids::NAME_ATTRIBUTE, "Number"),
        Triple::new(system_ids::POINT, system_ids::NAME_ATTRIBUTE, "Point"),
        Triple::new(system_ids::IMAGE, system_ids::NAME_ATTRIBUTE, "Image"),
        // System types
        Triple::new(
            system_ids::ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Attribute",
        ),
        Triple::new(system_ids::SCHEMA_TYPE, system_ids::NAME_ATTRIBUTE, "Type"),
        Triple::new(
            system_ids::RELATION_SCHEMA_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Relation schema type",
        ),
        Triple::new(
            system_ids::RELATION_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Relation instance type",
        ),
        // Properties
        Triple::new(
            system_ids::PROPERTIES,
            system_ids::NAME_ATTRIBUTE,
            "Properties",
        ),
        Triple::new(
            system_ids::TYPES_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Types",
        ),
        Triple::new(
            system_ids::VALUE_TYPE_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Value Type",
        ),
        Triple::new(
            system_ids::RELATION_TYPE_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Relation type attribute",
        ),
        Triple::new(
            system_ids::RELATION_INDEX,
            system_ids::NAME_ATTRIBUTE,
            "Relation index",
        ),
        Triple::new(
            system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Relation value type",
        ),
        Triple::new(
            system_ids::NAME_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Name",
        ),
        Triple::new(
            system_ids::DESCRIPTION_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Description",
        ),
    ];

    // Compute embeddings
    let embeddings =
        embedding_model.embed(triples.iter().map(|t| &t.value.value).collect(), None)?;

    let triples_with_embeddings = triples
        .into_iter()
        .zip(embeddings)
        .map(|(triple, embedding)| {
            let embedding = embedding.into_iter().map(|e| e as f64).collect();
            Triple::with_embedding(triple.entity, triple.attribute, triple.value, embedding)
        });

    triple::insert_many(
        &neo4j,
        &BlockMetadata::default(),
        system_ids::ROOT_SPACE_ID,
        DEFAULT_VERSION,
    )
    .triples(triples_with_embeddings)
    .send()
    .await
    .expect("Failed to insert triples");

    // Create properties
    create_property(
        neo4j,
        &embedding_model,
        "Properties",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::ATTRIBUTE),
        Some(system_ids::PROPERTIES),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Types",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::SCHEMA_TYPE),
        Some(system_ids::TYPES_ATTRIBUTE),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Value Type",
        system_ids::RELATION_SCHEMA_TYPE,
        None::<&str>,
        Some(system_ids::VALUE_TYPE_ATTRIBUTE),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation type attribute",
        system_ids::RELATION_SCHEMA_TYPE,
        None::<&str>,
        Some(system_ids::RELATION_TYPE_ATTRIBUTE),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation index",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::RELATION_INDEX),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation value type",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::SCHEMA_TYPE),
        Some(system_ids::RELATION_TYPE_ATTRIBUTE),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Name",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::NAME_ATTRIBUTE),
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Description",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::DESCRIPTION_ATTRIBUTE),
    )
    .await?;

    // Create types
    create_type(
        neo4j,
        &embedding_model,
        "Type",
        [system_ids::SCHEMA_TYPE],
        [
            system_ids::TYPES_ATTRIBUTE,
            system_ids::PROPERTIES,
            system_ids::NAME_ATTRIBUTE,
            system_ids::DESCRIPTION_ATTRIBUTE,
        ],
        Some(system_ids::SCHEMA_TYPE),
    )
    .await?;

    create_type(
        neo4j,
        &embedding_model,
        "Relation schema type",
        [system_ids::RELATION_SCHEMA_TYPE],
        [system_ids::RELATION_VALUE_RELATIONSHIP_TYPE],
        Some(system_ids::RELATION_SCHEMA_TYPE),
    )
    .await?;

    create_type(
        neo4j,
        &embedding_model,
        "Attribute",
        [system_ids::SCHEMA_TYPE],
        [
            system_ids::VALUE_TYPE_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            system_ids::DESCRIPTION_ATTRIBUTE,
        ],
        Some(system_ids::ATTRIBUTE),
    )
    .await?;

    create_type(
        neo4j,
        &embedding_model,
        "Relation instance type",
        [system_ids::RELATION_TYPE],
        [
            system_ids::RELATION_TYPE_ATTRIBUTE,
            system_ids::RELATION_INDEX,
        ],
        Some(system_ids::RELATION_TYPE),
    )
    .await?;

    Ok(())
}

pub async fn create_entity(
    neo4j: &neo4rs::Graph,
    embedding_model: &TextEmbedding,
    name: impl Into<String>,
    description: Option<&str>,
    types: impl IntoIterator<Item = &str>,
    properties: impl IntoIterator<Item = (&str, &str)>,
    relations: impl IntoIterator<Item = (&str, &str)>,
    id: Option<&str>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let entity_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let name = name.into();

    // Set: Entity.name
    triple::insert_many(neo4j, &block, system_ids::ROOT_SPACE_ID, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &entity_id,
            system_ids::NAME_ATTRIBUTE,
            name.clone(),
            embedding_model
                .embed(vec![name], Some(1))
                .unwrap_or(vec![Vec::<f32>::new()])
                .get(0)
                .unwrap_or(&Vec::<f32>::new())
                .iter()
                .map(|&x| x as f64)
                .collect(),
        )])
        .send()
        .await?;

    // Set: Entity.description
    if let Some(description) = description {
        triple::insert_many(neo4j, &block, system_ids::ROOT_SPACE_ID, DEFAULT_VERSION)
            .triples(vec![Triple::new(
                &entity_id,
                system_ids::DESCRIPTION_ATTRIBUTE,
                description,
            )])
            .send()
            .await?;
    }

    // Set: Entity > TYPES_ATTRIBUTE > Type[]
    set_types(neo4j, &entity_id, types).await?;

    // Set: Entity.*
    triple::insert_many(neo4j, &block, system_ids::ROOT_SPACE_ID, DEFAULT_VERSION)
        .triples(
            properties
                .into_iter()
                .map(|(property_id, value)| Triple::new(&entity_id, property_id, value)),
        )
        .send()
        .await?;

    // Set: Entity > RELATIONS > Relation[]
    relation::insert_many::<RelationEdge<EntityNodeRef>>(
        neo4j,
        &block,
        system_ids::ROOT_SPACE_ID,
        DEFAULT_VERSION,
    )
    .relations(relations.into_iter().map(|(relation_type, target_id)| {
        RelationEdge::new(
            ids::create_geo_id(),
            &entity_id,
            target_id,
            relation_type,
            "0",
        )
    }))
    .send()
    .await?;

    Ok(entity_id)
}

/// Creates a type with the given name, types, and properties.
pub async fn create_type(
    neo4j: &neo4rs::Graph,
    embedding_model: &TextEmbedding,
    name: impl Into<String>,
    types: impl IntoIterator<Item = &str>,
    properties: impl IntoIterator<Item = &str>,
    id: Option<&str>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let type_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let name = name.into();

    let mut types_vec: Vec<&str> = types.into_iter().collect();
    if !types_vec.contains(&system_ids::SCHEMA_TYPE) {
        types_vec.push(system_ids::SCHEMA_TYPE);
    }

    // Set: Type.name
    triple::insert_many(neo4j, &block, system_ids::ROOT_SPACE_TYPE, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &type_id,
            system_ids::NAME_ATTRIBUTE,
            name.clone(),
            embedding_model
                .embed(vec![name], Some(1))
                .unwrap_or(vec![Vec::<f32>::new()])
                .get(0)
                .unwrap_or(&Vec::<f32>::new())
                .iter()
                .map(|&x| x as f64)
                .collect(),
        )])
        .send()
        .await?;

    // Set: Type > TYPES_ATTRIBUTE > Type[]
    set_types(neo4j, &type_id, types_vec).await?;

    // Set: Type > PROPERTIES > Property[]
    relation::insert_many::<RelationEdge<EntityNodeRef>>(
        neo4j,
        &block,
        system_ids::ROOT_SPACE_ID,
        DEFAULT_VERSION,
    )
    .relations(properties.into_iter().map(|property_id| {
        RelationEdge::new(
            ids::create_geo_id(),
            &type_id,
            system_ids::PROPERTIES,
            property_id,
            "0",
        )
    }))
    .send()
    .await?;

    Ok(type_id)
}

/// Creates a property with the given name and value type.
/// If `relation_value_type` is provided, it will be set as the relation value type (
/// Note: if that is the case, then `value_type` should be the system_ids::RELATION_SCHEMA_TYPE type).
pub async fn create_property(
    neo4j: &neo4rs::Graph,
    embedding_model: &TextEmbedding,
    name: impl Into<String>,
    value_type: impl Into<String>,
    relation_value_type: Option<impl Into<String>>,
    id: Option<impl Into<String>>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();

    let property_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let string_name = name.into();

    // Set: Property.name
    triple::insert_many(neo4j, &block, system_ids::ROOT_SPACE_ID, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &property_id,
            system_ids::NAME_ATTRIBUTE,
            string_name.clone(),
            embedding_model
                .embed(vec![string_name], Some(1))
                .unwrap_or(vec![Vec::<f32>::new()])
                .get(0)
                .unwrap_or(&Vec::<f32>::new())
                .iter()
                .map(|&x| x as f64)
                .collect(),
        )])
        .send()
        .await?;

    // Set: Property > VALUE_TYPE > ValueType
    relation::insert_one::<RelationEdge<EntityNodeRef>>(
        neo4j,
        &block,
        system_ids::ROOT_SPACE_ID,
        DEFAULT_VERSION,
        RelationEdge::new(
            ids::create_geo_id(),
            property_id.clone(),
            system_ids::VALUE_TYPE_ATTRIBUTE,
            value_type.into(),
            "0",
        ),
    )
    .send()
    .await?;

    if let Some(relation_value_type) = relation_value_type {
        // Set: Property > RELATION_VALUE_RELATIONSHIP_TYPE > RelationValueType
        relation::insert_one::<RelationEdge<EntityNodeRef>>(
            neo4j,
            &block,
            system_ids::ROOT_SPACE_ID,
            DEFAULT_VERSION,
            RelationEdge::new(
                ids::create_geo_id(),
                property_id.clone(),
                system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
                relation_value_type.into(),
                "0",
            ),
        )
        .send()
        .await?;
    }

    set_types(neo4j, &property_id, [system_ids::ATTRIBUTE]).await?;

    Ok(property_id)
}

pub async fn set_types(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    types: impl IntoIterator<Item = &str>,
) -> anyhow::Result<()> {
    let block = BlockMetadata::default();
    let entity_id = entity_id.into();

    // Set: Entity > TYPES_ATTRIBUTE > Type[]
    relation::insert_many::<RelationEdge<EntityNodeRef>>(
        neo4j,
        &block,
        system_ids::ROOT_SPACE_ID,
        DEFAULT_VERSION,
    )
    .relations(types.into_iter().map(|type_id| {
        RelationEdge::new(
            ids::create_geo_id(),
            &entity_id,
            type_id,
            system_ids::TYPES_ATTRIBUTE,
            "0",
        )
    }))
    .send()
    .await?;

    Ok(())
}

pub async fn reset_db(neo4j: &neo4rs::Graph) -> anyhow::Result<()> {
    let embedding_dim = TextEmbedding::get_model_info(&EMBEDDING_MODEL)?.dim;

    // Delete indexes
    neo4j
        .run(neo4rs::query("DROP INDEX entity_id_index IF EXISTS"))
        .await?;
    neo4j
        .run(neo4rs::query("DROP INDEX relation_id_index IF EXISTS"))
        .await?;
    neo4j
        .run(neo4rs::query("DROP INDEX relation_type_index IF EXISTS"))
        .await?;
    neo4j
        .run(neo4rs::query("DROP INDEX vector_index IF EXISTS"))
        .await?;

    // Delete all nodes and relations
    neo4j
        .run(neo4rs::query("MATCH (n) DETACH DELETE n"))
        .await?;

    // Create indexes
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX entity_id_index FOR (e:Entity) ON (e.id)",
        ))
        .await?;
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX relation_id_index FOR () -[r:RELATION]-> () ON (r.id)",
        ))
        .await?;
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX relation_type_index FOR () -[r:RELATION]-> () ON (r.relation_type)",
        ))
        .await?;

    neo4j
        .run(neo4rs::query(&format!(
            "CREATE VECTOR INDEX vector_index FOR (a:Indexed) ON (a.embedding) OPTIONS {{indexConfig: {{`vector.dimensions`: {}, `vector.similarity_function`: 'COSINE'}}}}",
            embedding_dim as i64,
        )))
        .await?;

    Ok(())
}
