//! This example demonstrates simple default integration with [`axum`].

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

// use api::query_mapping::QueryMapper;
use axum::{
    response::Html,
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use clap::{Args, Parser};
use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, Executor, GraphQLScalar, RootNode,
    ScalarValue,
};
use juniper_axum::{extract::JuniperRequest, graphiql, playground, response::JuniperResponse};
use sink::kg;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct KnowledgeGraph(Arc<sink::kg::Client>);

impl juniper::Context for KnowledgeGraph {}

#[derive(Clone)]
pub struct Query;

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Query {
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: Option<String>,
        version_id: Option<String>,
    ) -> Option<Entity> {
        // let query = QueryMapper::default().select_root_node(&id, &executor.look_ahead()).build();
        // tracing::info!("Query: {}", query);

        let query = match (space_id, version_id) {
            (Some(space_id), _) => {
                sdk::neo4rs::query("MATCH (n {id: $id, space_id: $space_id}) RETURN n")
                    .param("id", id)
                    .param("space_id", space_id)
            }
            (None, _) => sdk::neo4rs::query("MATCH (n {id: $id}) RETURN n").param("id", id),
        };

        executor
            .context()
            .0
            .find_node::<HashMap<String, serde_json::Value>>(query)
            .await
            .expect("Failed to find node")
            .map(Entity::from)
    }
}

// Attributes GraphQL scalar
#[derive(Clone, Debug, GraphQLScalar)]
#[graphql(with = attributes)]
pub struct Attributes(HashMap<String, serde_json::Value>);

mod attributes {
    use juniper::{InputValue, ParseScalarResult, ScalarToken, ScalarValue, Value};

    use super::*;

    fn serde_to_graphql<S: ScalarValue>(v: &serde_json::Value) -> Value<S> {
        match v {
            serde_json::Value::String(s) => Value::scalar(s.clone()),
            serde_json::Value::Number(n) => Value::scalar(n.as_i64().unwrap() as i32),
            serde_json::Value::Bool(b) => Value::scalar(*b),
            serde_json::Value::Array(a) => Value::List(a.iter().map(serde_to_graphql).collect()),
            _ => Value::null(),
        }
    }

    pub(super) fn to_output<S: ScalarValue>(v: &Attributes) -> Value<S> {
        Value::Object(v.0.iter().fold(
            juniper::Object::with_capacity(v.0.len()),
            |mut obj, (k, v)| {
                obj.add_field(k, serde_to_graphql(v));
                obj
            },
        ))
    }

    pub(super) fn from_input<S: ScalarValue>(_v: &InputValue<S>) -> Result<Attributes, String> {
        // v.as_string_value()
        //     .map(|s| StringOrInt::String(s.into()))
        //     .or_else(|| v.as_int_value().map(StringOrInt::Int))
        //     .ok_or_else(|| format!("Expected `String` or `Int`, found: {v}"))
        unimplemented!()
    }

    pub(super) fn parse_token<S: ScalarValue>(_t: ScalarToken<'_>) -> ParseScalarResult<S> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    id: String,
    space_id: String,
    types: Vec<String>,
    attributes: Attributes,
}

impl From<kg::mapping::Node<HashMap<String, serde_json::Value>>> for Entity {
    fn from(node: kg::mapping::Node<HashMap<String, serde_json::Value>>) -> Self {
        Self {
            id: node.id().to_string(),
            space_id: node.space_id().to_string(),
            types: node.types,
            attributes: Attributes(node.attributes.attributes),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Entity {
    fn id(&self) -> &str {
        &self.id
    }

    fn space_id(&self) -> &str {
        &self.space_id
    }

    fn types(&self) -> &[String] {
        &self.types
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        executor
            .context()
            .0
            .find_relation_from_node::<HashMap<String, serde_json::Value>>(&self.id)
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    id: String,
    r#type: String,
    attributes: Attributes,
    // to: Node,
    // from: Node,
}

impl From<kg::mapping::Relation<HashMap<String, serde_json::Value>>> for Relation {
    fn from(node: kg::mapping::Relation<HashMap<String, serde_json::Value>>) -> Self {
        Self {
            id: node.id().to_string(),
            r#type: node.relation_type,
            attributes: Attributes(node.attributes.attributes),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Relation {
    fn id(&self) -> &str {
        &self.id
    }

    fn r#type(&self) -> &str {
        &self.r#type
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    async fn from<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        executor
            .context()
            .0
            .find_node_from_relation::<HashMap<String, serde_json::Value>>(&self.id)
            .await
            .expect("Failed to find node")
            .map(Entity::from)
            .unwrap()
    }

    async fn to<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        executor
            .context()
            .0
            .find_node_to_relation::<HashMap<String, serde_json::Value>>(&self.id)
            .await
            .expect("Failed to find node")
            .map(Entity::from)
            .unwrap()
    }
}

type Schema =
    RootNode<'static, Query, EmptyMutation<KnowledgeGraph>, EmptySubscription<KnowledgeGraph>>;

async fn homepage() -> Html<&'static str> {
    "<html><h1>juniper_axum/simple example</h1>\
           <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
           <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
    </html>"
        .into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level();
    init_tracing();

    let args = AppArgs::parse();

    let kg_client = sink::kg::Client::new(
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

    let app = Router::new()
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), custom_graphql),
        )
        // .route(
        //     "/subscriptions",
        //     get(ws::<Arc<Schema>>(ConnectionConfig::new(()))),
        // )
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .route("/", get(homepage))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(KnowledgeGraph(Arc::new(kg_client))));

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
