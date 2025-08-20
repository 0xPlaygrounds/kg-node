use axum::{Router, extract::Json, http::StatusCode, routing::post};
use clap::{Args, Parser};
use grc20_core::neo4rs;
use std::sync::Arc;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

use crate::{automatic_search::AutomaticSearchAgent, full_ai_search::FullAISearchAgent};

mod automatic_search;
mod full_ai_search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = AppArgs::parse();

    let neo4j = neo4rs::Graph::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    let full_ai_search_agent =
        Arc::new(FullAISearchAgent::new(neo4j.clone(), &args.gemini_api_key));
    let automatic_search_agent = Arc::new(AutomaticSearchAgent::new(
        neo4j.clone(),
        &args.gemini_api_key,
    ));

    let app = Router::new()
        .route("/question_ai", post(handler_full_ai_search))
        .route("/question", post(handler_automatic_search))
        .with_state((full_ai_search_agent, automatic_search_agent));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn handler_full_ai_search(
    axum::extract::State((search_agent, _)): axum::extract::State<(
        Arc<FullAISearchAgent>,
        Arc<AutomaticSearchAgent>,
    )>,
    Json(question): Json<String>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!("The question asked to the knowledge graph is: {question}");
    search_agent.natural_language_question(question).await
}

async fn handler_automatic_search(
    axum::extract::State((_, search_agent)): axum::extract::State<(
        Arc<FullAISearchAgent>,
        Arc<AutomaticSearchAgent>,
    )>,
    Json(question): Json<String>,
) -> Result<Json<String>, StatusCode> {
    tracing::info!("The question asked to the knowledge graph is: {question}");
    search_agent.natural_language_question(question).await
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
    #[arg(long)]
    gemini_api_key: String,
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
