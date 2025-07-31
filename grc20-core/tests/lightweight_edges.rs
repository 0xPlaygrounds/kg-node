#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct RelationNode {
    pub id: String,

    pub from: String,
    pub to: String,
    #[serde(deserialize_with = "deserialize_type")]
    pub relation_type: String,

    pub index: String,

    pub space_id: String,
    pub min_version: String,
    pub max_version: Option<String>,
    // /// System properties
    // #[serde(flatten)]
    // pub system_properties: SystemProperties,
}

fn deserialize_type<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let r#type: neo4rs::Type = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(r#type.0)
}

use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};

const BOLT_PORT: u16 = 7687;
const HTTP_PORT: u16 = 7474;

#[tokio::test]
async fn test_lightweight_edges() {
    let query = r#"
        CREATE (foo:Entity {id: "foo"})
        CREATE (bar:Entity {id: "bar"})
        CREATE (rel:Entity {id: "relation"})
        CREATE (foo) -[:`RELATION_TYPE` {id: "foobar_rel", entity: "relation", space_id: "root", min_version: "123", index: "0"}]-> (bar)
    "#;

    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let container = GenericImage::new("neo4j", "2025.01.0-community")
        .with_wait_for(WaitFor::Duration {
            length: std::time::Duration::from_secs(5),
        })
        .with_exposed_port(BOLT_PORT.tcp())
        .with_exposed_port(HTTP_PORT.tcp())
        .with_env_var("NEO4J_AUTH", "none")
        .start()
        .await
        .expect("Failed to start Neo 4J container");

    let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
    let host = container.get_host().await.unwrap().to_string();

    let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
        .await
        .unwrap();

    // Run the query to create the nodes and relationship
    neo4j
        .run(neo4rs::query(query))
        .await
        .expect("Failed to execute query");

    let row = neo4j
        .execute(neo4rs::query(
            r#"
            MATCH (from:Entity) -[r:`RELATION_TYPE`]-> (to:Entity)
            RETURN r{.*, relation_type: type(r), from: from.id, to: to.id} as r
        "#,
        ))
        .await
        .expect("Failed to execute query")
        .next()
        .await
        .expect("Failed to get next result")
        .expect("Failed to get result");

    println!("Row: {row:?}");

    let json: serde_json::Value = row.get("r").unwrap();

    println!("Data: {json}");

    let relation: RelationNode = row.get("r").unwrap();

    println!("Relation: {relation:?}");
}
