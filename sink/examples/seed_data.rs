use chrono::Local;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use grc20_core::{
    block::BlockMetadata,
    entity::EntityNodeRef,
    ids, indexer_ids,
    mapping::{triple, Query, RelationEdge, Triple, Value},
    neo4rs, relation, system_ids,
};
use grc20_sdk::models::space;

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
const POLYMTL_ID: &str = "Mu7ddiBnwZH1LvpDTpKcvq";
const MAUD_COHEN_ID: &str = "DVurPdLUZi7Ajfv9BC3ADm";
const CIVIL_ENGINEERING_ID: &str = "YEZVCYJTudKVreLEWuxFXV";
const SOFTWARE_ENGINEERING_ID: &str = "MPxRvh35rnDeRJNEJLU1YF";
const COMPUTER_ENGINEERING_ID: &str = "JjoWPp8LiCKVZiWtE5iZaJ";
const MECANICAL_ENGINEERING_ID: &str = "8bCuTuWqL3dxALLff1Awdb";
const OLIVIER_GENDREAU_ID: &str = "9Bj46RXQzHQq25WNPY4Lw";
const FR_SPACE_ID: &str = "RkTkM28NSx3WZuW33vZUjx";
const FR_QC_SPACE_ID: &str = "Lc9L7StPfXMFGWw45utaTY";
const DIRECTOR_PROP: &str = "G49gECRJmW6BwqHaENF5nS";
const PROGRAM_TYPE: &str = "GfugZRvoWmQhkjMcFJHg49";
const SCHOOL_TYPE: &str = "M89C7wwdJVaCW9rAVQpJbY";
const PROGRAM_PROP: &str = "5bwj7yNukCHoJnW8ksgZY";
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

    let dt = Local::now();

    let block = BlockMetadata {
        cursor: "random_cursor".to_string(),
        block_number: 0,
        timestamp: dt.to_utc(),
        request_id: "request_id".to_string(),
    };

    create_type(
        &neo4j,
        &embedding_model,
        "Space",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(system_ids::SPACE_TYPE),
        None,
    )
    .await?;

    space::builder(FR_SPACE_ID, "0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c")
        .build()
        .insert(&neo4j, &block, indexer_ids::INDEXER_SPACE_ID, "0")
        .send()
        .await?;

    space::builder(FR_QC_SPACE_ID, "0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c")
        .build()
        .insert(&neo4j, &block, indexer_ids::INDEXER_SPACE_ID, "0")
        .send()
        .await?;

    space::builder(
        system_ids::ROOT_SPACE_ID,
        "0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c",
    )
    .build()
    .insert(&neo4j, &block, indexer_ids::INDEXER_SPACE_ID, "0")
    .send()
    .await?;

    insert_relation(
        &neo4j,
        FR_QC_SPACE_ID,
        indexer_ids::PARENT_SPACE,
        FR_SPACE_ID,
        indexer_ids::INDEXER_SPACE_ID,
    )
    .await?;
    insert_relation(
        &neo4j,
        FR_SPACE_ID,
        indexer_ids::PARENT_SPACE,
        system_ids::ROOT_SPACE_ID,
        indexer_ids::INDEXER_SPACE_ID,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        FR_QC_SPACE_ID,
        system_ids::NAME_PROPERTY,
        "Quebec",
        FR_QC_SPACE_ID,
    )
    .await?;
    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        FR_QC_SPACE_ID,
        system_ids::DESCRIPTION_PROPERTY,
        "The space for Quebec related content",
        FR_QC_SPACE_ID,
    )
    .await?;
    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        FR_SPACE_ID,
        system_ids::NAME_PROPERTY,
        "Francophonie",
        FR_SPACE_ID,
    )
    .await?;

    // Create some common types
    create_type(
        &neo4j,
        &embedding_model,
        "Person",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(system_ids::PERSON_TYPE),
        None,
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "Event",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(EVENT_TYPE),
        None,
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "City",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(CITY_TYPE),
        None,
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "Program",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(PROGRAM_TYPE),
        None,
    )
    .await?;

    create_type(
        &neo4j,
        &embedding_model,
        "School",
        [],
        [system_ids::NAME_PROPERTY, system_ids::DESCRIPTION_PROPERTY,],
        Some(SCHOOL_TYPE),
        None,
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Event location",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(CITY_TYPE),
        Some(EVENT_LOCATION_PROP),
        None,
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Speakers",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::PERSON_TYPE),
        Some(SPEAKERS_PROP),
        None,
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Side events",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(EVENT_TYPE),
        Some(SIDE_EVENTS),
        None,
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Director",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::PERSON_TYPE),
        Some(DIRECTOR_PROP),
        None,
    )
    .await?;

    create_property(
        &neo4j,
        &embedding_model,
        "Program",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::PERSON_TYPE),
        Some(PROGRAM_PROP),
        None,
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
            (system_ids::NAME_PROPERTY, "Alice"),
            (
                system_ids::DESCRIPTION_PROPERTY,
                "Speaker at Rust Conference 2023",
            ),
        ],
        [],
        Some(ALICE_ID),
        None,
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
        None,
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
        None,
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
        None,
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
        None,
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
        None,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Maud Cohen",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(MAUD_COHEN_ID),
        None,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Olivier Gendreau",
        None,
        [system_ids::PERSON_TYPE],
        [],
        [],
        Some(OLIVIER_GENDREAU_ID),
        None,
    )
    .await?;

    //Create programs entities
    create_entity(
        &neo4j,
        &embedding_model,
        "Software Engineering",
        Some("Description of the software engineering program at Polytl."),
        [PROGRAM_TYPE],
        [],
        [(DIRECTOR_PROP, OLIVIER_GENDREAU_ID)],
        Some(SOFTWARE_ENGINEERING_ID),
        None,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        SOFTWARE_ENGINEERING_ID,
        system_ids::NAME_PROPERTY,
        "Génie logiciel",
        FR_SPACE_ID,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        SOFTWARE_ENGINEERING_ID,
        system_ids::NAME_ATTRIBUTE,
        "Génie logiciel",
        FR_QC_SPACE_ID,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        SOFTWARE_ENGINEERING_ID,
        system_ids::DESCRIPTION_ATTRIBUTE,
        "Description du programme de génie logiciel à Polytechnique Montréal.",
        FR_QC_SPACE_ID,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Computer Engineering",
        Some("Description of the Computer engineering program at Polymtl."),
        [PROGRAM_TYPE],
        [],
        [],
        Some(COMPUTER_ENGINEERING_ID),
        None,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        SOFTWARE_ENGINEERING_ID,
<<<<<<< HEAD
        system_ids::NAME_PROPERTY,
        "Génie informatique",
=======
        system_ids::NAME_ATTRIBUTE,
        "Génie logiciel",
>>>>>>> main
        FR_SPACE_ID,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Civil Engineering",
        Some("Description of the program of civil engineering at polymtl"),
        [PROGRAM_TYPE],
        [],
        [],
        Some(CIVIL_ENGINEERING_ID),
        None,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Mecanical Engineering",
        None,
        [PROGRAM_TYPE],
        [],
        [],
        Some(MECANICAL_ENGINEERING_ID),
        None,
    )
    .await?;

    create_entity(
        &neo4j,
        &embedding_model,
        "Polytechnique Montreal",
        None,
        [SCHOOL_TYPE],
        [],
        [
            (DIRECTOR_PROP, MAUD_COHEN_ID),
            (PROGRAM_PROP, CIVIL_ENGINEERING_ID),
            (PROGRAM_PROP, SOFTWARE_ENGINEERING_ID),
            (PROGRAM_PROP, COMPUTER_ENGINEERING_ID),
        ],
        Some(POLYMTL_ID),
        None,
    )
    .await?;

    insert_attribute_with_embedding(
        &neo4j,
        &embedding_model,
        POLYMTL_ID,
        system_ids::NAME_ATTRIBUTE,
        "École Polytechnique Montréal",
        FR_QC_SPACE_ID,
    )
    .await?;

    insert_relation(
        &neo4j,
        POLYMTL_ID,
        PROGRAM_PROP,
        SOFTWARE_ENGINEERING_ID,
        FR_QC_SPACE_ID,
    )
    .await?;
    insert_relation(
        &neo4j,
        POLYMTL_ID,
        PROGRAM_PROP,
        CIVIL_ENGINEERING_ID,
        FR_QC_SPACE_ID,
    )
    .await?;
    insert_relation(
        &neo4j,
        POLYMTL_ID,
        PROGRAM_PROP,
        MECANICAL_ENGINEERING_ID,
        FR_QC_SPACE_ID,
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
        None,
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
        None,
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
        None,
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
        None,
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
        None,
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
        None,
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
        Triple::new(system_ids::CHECKBOX, system_ids::NAME_PROPERTY, "Checkbox"),
        Triple::new(system_ids::TIME, system_ids::NAME_PROPERTY, "Time"),
        Triple::new(system_ids::TEXT, system_ids::NAME_PROPERTY, "Text"),
        Triple::new(system_ids::URL, system_ids::NAME_PROPERTY, "Url"),
        Triple::new(system_ids::NUMBER, system_ids::NAME_PROPERTY, "Number"),
        Triple::new(system_ids::POINT, system_ids::NAME_PROPERTY, "Point"),
        Triple::new(system_ids::IMAGE, system_ids::NAME_PROPERTY, "Image"),
        // System types
        Triple::new(
            system_ids::PROPERTY_TYPE,
            system_ids::NAME_PROPERTY,
            "Attribute",
        ),
        Triple::new(system_ids::SCHEMA_TYPE, system_ids::NAME_PROPERTY, "Type"),
        Triple::new(
            system_ids::RELATION_SCHEMA_TYPE,
            system_ids::NAME_PROPERTY,
            "Relation schema type",
        ),
        Triple::new(
            system_ids::RELATION_TYPE,
            system_ids::NAME_PROPERTY,
            "Relation instance type",
        ),
        // Properties
        Triple::new(
            system_ids::PROPERTIES,
            system_ids::NAME_PROPERTY,
            "Properties",
        ),
        Triple::new(
            system_ids::TYPES_ATTRIBUTE,
            system_ids::NAME_PROPERTY,
            "Types",
        ),
        Triple::new(
            system_ids::VALUE_TYPE_ATTRIBUTE,
            system_ids::NAME_PROPERTY,
            "Value Type",
        ),
        Triple::new(
            system_ids::RELATION_TYPE_ATTRIBUTE,
            system_ids::NAME_PROPERTY,
            "Relation type attribute",
        ),
        Triple::new(
            system_ids::RELATION_INDEX,
            system_ids::NAME_PROPERTY,
            "Relation index",
        ),
        Triple::new(
            system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
            system_ids::NAME_PROPERTY,
            "Relation value type",
        ),
        Triple::new(system_ids::NAME_PROPERTY, system_ids::NAME_PROPERTY, "Name"),
        Triple::new(
            system_ids::DESCRIPTION_PROPERTY,
            system_ids::NAME_PROPERTY,
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
        Some(system_ids::PROPERTY_TYPE),
        Some(system_ids::PROPERTIES),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Types",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::SCHEMA_TYPE),
        Some(system_ids::TYPES_ATTRIBUTE),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Value Type",
        system_ids::RELATION_SCHEMA_TYPE,
        None::<&str>,
        Some(system_ids::VALUE_TYPE_ATTRIBUTE),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation type attribute",
        system_ids::RELATION_SCHEMA_TYPE,
        None::<&str>,
        Some(system_ids::RELATION_TYPE_ATTRIBUTE),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation index",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::RELATION_INDEX),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Relation value type",
        system_ids::RELATION_SCHEMA_TYPE,
        Some(system_ids::SCHEMA_TYPE),
        Some(system_ids::RELATION_TYPE_ATTRIBUTE),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Name",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::NAME_PROPERTY),
        None,
    )
    .await?;

    create_property(
        neo4j,
        &embedding_model,
        "Description",
        system_ids::TEXT,
        None::<&str>,
        Some(system_ids::DESCRIPTION_PROPERTY),
        None,
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
            system_ids::NAME_PROPERTY,
            system_ids::DESCRIPTION_PROPERTY,
        ],
        Some(system_ids::SCHEMA_TYPE),
        None,
    )
    .await?;

    create_type(
        neo4j,
        &embedding_model,
        "Relation schema type",
        [system_ids::RELATION_SCHEMA_TYPE],
        [system_ids::RELATION_VALUE_RELATIONSHIP_TYPE],
        Some(system_ids::RELATION_SCHEMA_TYPE),
        None,
    )
    .await?;

    create_type(
        neo4j,
        &embedding_model,
        "Attribute",
        [system_ids::SCHEMA_TYPE],
        [
            system_ids::VALUE_TYPE_ATTRIBUTE,
            system_ids::NAME_PROPERTY,
            system_ids::DESCRIPTION_PROPERTY,
        ],
        Some(system_ids::PROPERTY_TYPE),
        None,
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
        None,
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
    space_id: Option<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let entity_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let name = name.into();

    let space_id = space_id.as_deref().unwrap_or(system_ids::ROOT_SPACE_ID);

    // Set: Entity.name
    triple::insert_many(neo4j, &block, space_id, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &entity_id,
            system_ids::NAME_PROPERTY,
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
        triple::insert_many(neo4j, &block, space_id, DEFAULT_VERSION)
            .triples(vec![Triple::new(
                &entity_id,
                system_ids::DESCRIPTION_PROPERTY,
                description,
            )])
            .send()
            .await?;
    }

    // Set: Entity > TYPES_ATTRIBUTE > Type[]
    set_types(neo4j, &entity_id, types).await?;

    // Set: Entity.*
    triple::insert_many(neo4j, &block, space_id, DEFAULT_VERSION)
        .triples(
            properties
                .into_iter()
                .map(|(property_id, value)| Triple::new(&entity_id, property_id, value)),
        )
        .send()
        .await?;

    // Set: Entity > RELATIONS > Relation[]
    relation::insert_many::<RelationEdge<EntityNodeRef>>(neo4j, &block, space_id, DEFAULT_VERSION)
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

pub async fn insert_attribute(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    attribute_id: impl Into<String>,
    attribute_value: impl Into<String>,
    space_id: impl Into<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let attribute_id = attribute_id.into();
    let attribute_value = attribute_value.into();
    let space_id = space_id.into();
    let entity_id = entity_id.into();

    triple::insert_one(
        neo4j,
        &block,
        space_id,
        DEFAULT_VERSION,
        Triple::new(entity_id.clone(), attribute_id, attribute_value),
    )
    .send()
    .await?;
    Ok(entity_id)
}

pub async fn insert_relation(
    neo4j: &neo4rs::Graph,
    entity_from_id: impl Into<String>,
    relation_id: impl Into<String>,
    entity_to_id: impl Into<String>,
    space_id: impl Into<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let entity_from_id = entity_from_id.into();
    let relation_id = relation_id.into();
    let space_id = space_id.into();
    let entity_to_id = entity_to_id.into();

    relation::insert_one(
        neo4j,
        &block,
        space_id,
        DEFAULT_VERSION,
        RelationEdge::new(
            "id".to_string(),
            entity_from_id,
            entity_to_id,
            relation_id.clone(),
            Value::text(relation_id.clone()),
        ),
    )
    .send()
    .await?;
    Ok(relation_id)
}

pub async fn insert_attribute_with_embedding(
    neo4j: &neo4rs::Graph,
    embedding_model: &TextEmbedding,
    entity_id: impl Into<String>,
    attribute_id: impl Into<String>,
    attribute_value: impl Into<String>,
    space_id: impl Into<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let attribute_id = attribute_id.into();
    let attribute_value = attribute_value.into();
    let space_id = space_id.into();
    let entity_id = entity_id.into();

    triple::insert_one(
        neo4j,
        &block,
        space_id,
        DEFAULT_VERSION,
        Triple::with_embedding(
            &entity_id,
            attribute_id,
            attribute_value.clone(),
            embedding_model
                .embed(vec![attribute_value], Some(1))
                .unwrap_or(vec![Vec::<f32>::new()])
                .get(0)
                .unwrap_or(&Vec::<f32>::new())
                .iter()
                .map(|&x| x as f64)
                .collect(),
        ),
    )
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
    space_id: Option<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();
    let type_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let name = name.into();

    let mut types_vec: Vec<&str> = types.into_iter().collect();
    if !types_vec.contains(&system_ids::SCHEMA_TYPE) {
        types_vec.push(system_ids::SCHEMA_TYPE);
    }

    let space_id = space_id.as_deref().unwrap_or(system_ids::ROOT_SPACE_ID);

    // Set: Type.name
    triple::insert_many(neo4j, &block, space_id, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &type_id,
            system_ids::NAME_PROPERTY,
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
    relation::insert_many::<RelationEdge<EntityNodeRef>>(neo4j, &block, space_id, DEFAULT_VERSION)
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
    space_id: Option<String>,
) -> anyhow::Result<String> {
    let block = BlockMetadata::default();

    let property_id = id.map(Into::into).unwrap_or_else(|| ids::create_geo_id());
    let string_name = name.into();

    let space_id = space_id.as_deref().unwrap_or(system_ids::ROOT_SPACE_ID);

    // Set: Property.name
    triple::insert_many(neo4j, &block, space_id, DEFAULT_VERSION)
        .triples(vec![Triple::with_embedding(
            &property_id,
            system_ids::NAME_PROPERTY,
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
        space_id,
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

    set_types(neo4j, &property_id, [system_ids::PROPERTY_TYPE]).await?;

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
