use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::TryStreamExt;
use grc20_core::{
    entity::{self, utils::TypesFilter},
    neo4rs, system_ids,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let neo4j = neo4rs::Graph::new("neo4j://localhost:7687", "neo4j", "neo4j").await?;

    let embedding_model =
        TextEmbedding::try_new(InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true))
            .expect("Failed to initialize embedding model");

    let query = "Person";

    let embedding = embedding_model
        .embed(vec![&query], None)
        .expect("Failed to get embedding")
        .pop()
        .expect("Embedding is empty")
        .into_iter()
        .map(|v| v as f64)
        .collect::<Vec<_>>();

    let results = entity::exact_search::exact_search(&neo4j, embedding)
        .filter(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
        .limit(5)
        .send()
        .await
        .expect("Failed to execute search query")
        .map_ok(|result| result.into_entity())
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect search results");

    for entity in results {
        println!(
            "Found entity: {} ({})",
            entity.id,
            entity.names().join(", ")
        );
    }

    Ok(())
}
