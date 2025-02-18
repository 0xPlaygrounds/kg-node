//! This example demonstrates simple default integration with [`axum`].

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::Method,
    response::{Html, Json},
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use clap::{Args, Parser};
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{extract::JuniperRequest, graphiql, playground, response::JuniperResponse};
use sdk::neo4rs;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{context::KnowledgeGraph, schema::Query};

type Schema =
    RootNode<'static, Query, EmptyMutation<KnowledgeGraph>, EmptySubscription<KnowledgeGraph>>;

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

    let schema = Schema::new(
        Query,
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
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .route("/", get(homepage))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(KnowledgeGraph::new(Arc::new(neo4j))))
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

async fn custom_graphql(
    Extension(schema): Extension<Arc<Schema>>,
    Extension(kg): Extension<KnowledgeGraph>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    JuniperResponse(request.execute(&*schema, &kg).await)
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
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
