//! This example demonstrates simple default integration with [`axum`].

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::Method,
    response::{Html, Json},
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use cache::{CacheConfig, KgCache};
use clap::{Args, Parser};
use grc20_core::neo4rs;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{extract::JuniperRequest, graphiql, playground, response::JuniperResponse};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{context::KnowledgeGraph, schema::RootQuery};

type Schema =
    RootNode<'static, RootQuery, EmptyMutation<KnowledgeGraph>, EmptySubscription<KnowledgeGraph>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level();
    init_tracing();

    let args = AppArgs::parse();

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

    let schema = Schema::new(
        RootQuery,
        EmptyMutation::<KnowledgeGraph>::new(),
        EmptySubscription::<KnowledgeGraph>::new(),
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), custom_graphql),
        )
        // .route(
        //     "/subscriptions",
        //     get(ws::<Arc<Schema>>(ConnectionConfig::new(()))),
        // )
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/cursor", get(cursor))
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .route("/", get(homepage))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(KnowledgeGraph::new(Arc::new(neo4j), cache)))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));
    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    #[clap(flatten)]
    cache_args: CacheArgs,
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

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

async fn custom_graphql(
    Extension(schema): Extension<Arc<Schema>>,
    Extension(kg): Extension<KnowledgeGraph>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    JuniperResponse(request.execute(&*schema, &kg).await)
}

async fn homepage() -> Html<&'static str> {
    "<html><h1>KG API</h1>\
           <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
           <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
    </html>"
        .into()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "component": "api",
        "status": "ok",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "git": {
            "tag": env!("GIT_TAG"),
            "commit": env!("GIT_COMMIT"),
        },
    }))
}

async fn cursor(
    Extension(kg): Extension<KnowledgeGraph>,
) -> Json<Option<grc20_sdk::models::Cursor>> {
    // let cursor = grc20_core::mapping::triple::find_one(
    //     &kg.0,
    //     indexer_ids::CURSOR_ATTRIBUTE,
    //     indexer_ids::CURSOR_ID,
    //     indexer_ids::INDEXER_SPACE_ID,
    //     Some("0".to_string()),
    // )
    // .send()
    // .await
    // .unwrap();

    // let block_number = grc20_core::mapping::triple::find_one(
    //     &kg.0,
    //     indexer_ids::BLOCK_NUMBER_ATTRIBUTE,
    //     indexer_ids::CURSOR_ID,
    //     indexer_ids::INDEXER_SPACE_ID,
    //     Some("0".to_string()),
    // )
    // .send()
    // .await
    // .unwrap();
    let cursor = grc20_sdk::models::Cursor::load(&kg.neo4j)
        .await
        .unwrap()
        .map(|cursor| cursor.attributes);

    Json(cursor)
}
