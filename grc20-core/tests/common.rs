use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

const BOLT_PORT: u16 = 7687;
const HTTP_PORT: u16 = 7474;

pub async fn setup_neo4j_container() -> (neo4rs::Graph, ContainerAsync<GenericImage>) {
    let container = GenericImage::new("neo4j", "2025.01.0-community")
        .with_wait_for(WaitFor::Duration {
            length: std::time::Duration::from_secs(5),
        })
        .with_exposed_port(BOLT_PORT.tcp())
        .with_exposed_port(HTTP_PORT.tcp())
        .with_env_var("NEO4J_AUTH", "none")
        .with_env_var("NEO4J_PLUGINS", "[\"apoc\"]")
        .start()
        .await
        .expect("Failed to start Neo 4J container");

    let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
    let host = container.get_host().await.unwrap().to_string();

    let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
        .await
        .unwrap();

    // Create vector index for semantic search
    neo4j
        .run(neo4rs::query(
            "CREATE VECTOR INDEX vector_index IF NOT EXISTS FOR (p:Properties) ON (p.embedding) OPTIONS {indexConfig: {`vector.dimensions`: 3, `vector.similarity_function`: 'cosine'}}"
        ))
        .await
        .expect("Failed to create vector index");

    (neo4j, container)
}
