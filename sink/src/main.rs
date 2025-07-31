use std::{env, sync::Arc};

use anyhow::Error;
use axum::{response::Json, routing::get, Router};
use cache::{CacheConfig, KgCache};
use clap::{Args, Parser};
use grc20_core::entity::UpdateEntity;
use grc20_core::value::Value;
use grc20_core::{indexer_ids, neo4rs};
use sink::{events::EventHandler, metrics};
use std::time::Duration;
use substreams_utils::Sink;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const PKG_FILE: &str = "geo-substream.spkg";
const MODULE_NAME: &str = "geo_out";

const DEFAULT_START_BLOCK: u64 = 880;
const DEFAULT_END_BLOCK: u64 = 0;
const DEFAULT_HTTP_PORT: u16 = 8081;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let endpoint_url =
        env::var("SUBSTREAMS_ENDPOINT_URL").expect("SUBSTREAMS_ENDPOINT_URL not set");
    let start_block = env::var("SUBSTREAMS_START_BLOCK").unwrap_or_else(|_| {
        tracing::warn!(
            "SUBSTREAMS_START_BLOCK not set. Using default value: {}",
            DEFAULT_START_BLOCK
        );
        DEFAULT_START_BLOCK.to_string()
    });
    let end_block = env::var("SUBSTREAMS_END_BLOCK").unwrap_or_else(|_| {
        tracing::warn!(
            "SUBSTREAMS_END_BLOCK not set. Using default value: {}",
            DEFAULT_END_BLOCK
        );
        DEFAULT_END_BLOCK.to_string()
    });

    let args = AppArgs::parse();

    set_log_level();
    let _guard = init_tracing(args.log_file);

    let neo4j = neo4rs::Graph::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    let cache = if let Some(uri) = args.cache_args.memcache_uri {
        let cache_config = CacheConfig::new(vec![uri])
            .with_default_expiry(Duration::from_secs(args.cache_args.memcache_default_expiry));
        Some(Arc::new(KgCache::new(cache_config)?))
    } else {
        None
    };

    let sink = EventHandler::new(neo4j, cache)?
        .versioning(!args.no_versioning)
        .governance(!args.no_governance);

    if args.reset_db {
        reset_db(&sink).await?;
    } else {
        migration_check(&sink).await?;
    }

    start_http_server().await;

    sink.run(
        &endpoint_url,
        PKG_FILE,
        MODULE_NAME,
        start_block
            .parse()
            .unwrap_or_else(|_| panic!("Invalid start block: {start_block}! Must be integer")),
        end_block
            .parse()
            .unwrap_or_else(|_| panic!("Invalid end block: {end_block}! Must be integer")),
        Some(64),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    #[clap(flatten)]
    cache_args: CacheArgs,

    /// Whether or not to reset the database
    #[arg(long)]
    reset_db: bool,

    /// Log file path
    #[arg(long)]
    log_file: Option<String>,

    /// Whether to enable versioning
    #[arg(long, default_value = "false")]
    no_versioning: bool,

    /// Whether to index governance events
    #[arg(long, default_value = "false")]
    no_governance: bool,
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

#[derive(Debug, Args)]
struct CacheArgs {
    /// Memcache server URI (optional)
    #[arg(long, env = "memcache_uri")]
    memcache_uri: Option<String>,

    /// Default cache expiry in seconds
    #[arg(long, env = "memcache_default_expiry", default_value = "3600")]
    memcache_default_expiry: u64,
}

pub async fn reset_db(handler: &EventHandler) -> anyhow::Result<()> {
    // Delete indexes
    handler
        .neo4j()
        .run(neo4rs::query("DROP INDEX entity_id_index IF EXISTS"))
        .await?;
    handler
        .neo4j()
        .run(neo4rs::query("DROP INDEX relation_id_index IF EXISTS"))
        .await?;
    handler
        .neo4j()
        .run(neo4rs::query("DROP INDEX relation_type_index IF EXISTS"))
        .await?;
    handler
        .neo4j()
        .run(neo4rs::query("DROP INDEX vector_index IF EXISTS"))
        .await?;

    // Delete all nodes and relations
    handler
        .neo4j()
        .run(neo4rs::query("MATCH (n) DETACH DELETE n"))
        .await?;

    // Create indexes
    handler
        .neo4j()
        .run(neo4rs::query(
            "CREATE INDEX entity_id_index FOR (e:Entity) ON (e.id)",
        ))
        .await?;
    handler
        .neo4j()
        .run(neo4rs::query(
            "CREATE INDEX relation_id_index FOR () -[r:RELATION]-> () ON (r.id)",
        ))
        .await?;

    handler.neo4j()
        .run(neo4rs::query(&format!(
            "CREATE VECTOR INDEX vector_index FOR (p:Properties) ON (p.embedding) OPTIONS {{indexConfig: {{`vector.dimensions`: {}, `vector.similarity_function`: 'COSINE'}}}}",
            handler.embedding_dim()
        )))
        .await?;

    // // Bootstrap indexer entities
    // mapping::triple::insert_many(
    //     handler.neo4j(),
    //     &BlockMetadata::default(),
    //     indexer_ids::INDEXER_SPACE_ID,
    //     "0",
    // )
    // .triples(bootstrap::boostrap_indexer::triples())
    // .send()
    // .await?;
    grc20_core::entity::update_one(
        handler.neo4j(),
        UpdateEntity::new(indexer_ids::CURSOR_ID)
            .value(Value::new(indexer_ids::VERSION_ATTRIBUTE, env!("GIT_TAG"))),
        indexer_ids::INDEXER_SPACE_ID,
    )
    .send()
    .await?;

    Ok(())
}

async fn migration_check(handler: &EventHandler) -> Result<(), Error> {
    let version = grc20_core::value::find_one(
        handler.neo4j(),
        indexer_ids::INDEXER_SPACE_ID,
        indexer_ids::CURSOR_ID,
        indexer_ids::VERSION_ATTRIBUTE,
    )
    .send()
    .await?;

    if let Some(version) = version {
        if version.value != env!("GIT_TAG") {
            tracing::info!(
                "Version mismatch. Resetting the database. Old version: {}, New version: {}",
                version.value,
                env!("GIT_TAG")
            );
            reset_db(handler).await?;
        } else {
            tracing::info!("Version match: {}. No migration needed.", version.value);
        }
    } else {
        tracing::info!("No version found in the database. Resetting the database.");
        reset_db(handler).await?;
    }

    Ok(())
}

fn init_tracing(log_file: Option<String>) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    if let Some(log_file) = log_file {
        // Set the path of the log file
        let now = chrono::Utc::now();
        let file_appender = tracing_appender::rolling::never(
            ".",
            format!("{}-{log_file}", now.format("%Y-%m-%d-%H-%M-%S")),
        );
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();

        Some(_guard)
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "stdout=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        None
    }
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "component": "sink",
        "status": "ok",
    }))
}

async fn start_http_server() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::metrics_handler));

    let port = env::var("KG_SINK_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| {
            tracing::info!(
                "KG_SINK_HTTP_PORT not set, using default port {}",
                DEFAULT_HTTP_PORT
            );
            DEFAULT_HTTP_PORT
        });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to start health server on {addr}: {e}"));
    tracing::info!("Health available on {addr}/health");
    tracing::info!("Metrics available on {addr}/metrics");

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|e| panic!("failed to run HTTP server: {e}"));
    });
}
