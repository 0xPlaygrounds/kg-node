//! This example demonstrates simple default integration with [`axum`].

use std::{net::SocketAddr, sync::Arc};

use axum::{
    response::Html,
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use chrono::{DateTime, Utc};
use clap::{Args, Parser};
use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, Executor, GraphQLEnum, GraphQLInputObject,
    GraphQLObject, RootNode, ScalarValue,
};
use juniper_axum::{extract::JuniperRequest, graphiql, playground, response::JuniperResponse};
use sdk::{mapping, system_ids};
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
    /// Returns a single entity identified by its ID and space ID
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        // version_id: Option<String>,
    ) -> Option<Entity> {
        // let query = QueryMapper::default().select_root_node(&id, &executor.look_ahead()).build();
        // tracing::info!("Query: {}", query);

        mapping::Entity::<mapping::Triples>::find_by_id(&executor.context().0.neo4j, &id, &space_id)
            .await
            .expect("Failed to find entity")
            .map(Entity::from)
    }

    /// Returns multiple entities according to the provided space ID and filter
    async fn entities<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        // version_id: Option<String>,
        filter: Option<EntityFilter>,
    ) -> Vec<Entity> {
        // let query = QueryMapper::default().select_root_node(&id, &executor.look_ahead()).build();
        // tracing::info!("Query: {}", query);

        match filter {
            Some(EntityFilter { types: Some(types) }) if !types.is_empty() => {
                mapping::Entity::<mapping::Triples>::find_by_types(
                    &executor.context().0.neo4j,
                    &types,
                    &space_id,
                )
                .await
                .expect("Failed to find entities")
                .into_iter()
                .map(Entity::from)
                .collect::<Vec<_>>()
            }
            _ => mapping::Entity::<mapping::Triples>::find_all(
                &executor.context().0.neo4j,
                &space_id,
            )
            .await
            .expect("Failed to find entities")
            .into_iter()
            .map(Entity::from)
            .collect::<Vec<_>>(),
        }
    }

    /// Returns a single relation identified by its ID and space ID
    async fn relation<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        // version_id: Option<String>,
    ) -> Option<Relation> {
        mapping::Relation::<mapping::Triples>::find_by_id(
            &executor.context().0.neo4j,
            &id,
            &space_id,
        )
        .await
        .expect("Failed to find relation")
        .map(|rel| rel.into())
    }

    /// Returns multiple relations according to the provided space ID and filter
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        // version_id: Option<String>,
        filter: Option<RelationFilter>,
    ) -> Vec<Relation> {
        match filter {
            Some(RelationFilter {
                relation_types: Some(types),
            }) if !types.is_empty() => mapping::Relation::<mapping::Triples>::find_by_types(
                &executor.context().0.neo4j,
                &types,
                &space_id,
            )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>(),
            _ => mapping::Relation::<mapping::Triples>::find_all(
                &executor.context().0.neo4j,
                &space_id,
            )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>(),
        }
    }
}

/// Entity filter input object
#[derive(Debug, GraphQLInputObject)]
struct EntityFilter {
    /// Filter by entity types
    types: Option<Vec<String>>,
}

/// Relation filter input object
#[derive(Debug, GraphQLInputObject)]
struct RelationFilter {
    /// Filter by relation types
    relation_types: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Entity {
    id: String,
    types: Vec<String>,
    space_id: String,
    created_at: DateTime<Utc>,
    created_at_block: String,
    updated_at: DateTime<Utc>,
    updated_at_block: String,
    attributes: Vec<Triple>,
}

impl From<mapping::Entity<mapping::Triples>> for Entity {
    fn from(entity: mapping::Entity<mapping::Triples>) -> Self {
        Self {
            id: entity.attributes.id,
            types: entity.types,
            space_id: entity.attributes.system_properties.space_id.clone(),
            created_at: entity.attributes.system_properties.created_at,
            created_at_block: entity.attributes.system_properties.created_at_block,
            updated_at: entity.attributes.system_properties.updated_at,
            updated_at_block: entity.attributes.system_properties.updated_at_block,
            attributes: entity
                .attributes
                .attributes
                .into_iter()
                .map(|(key, triple)| Triple {
                    space_id: entity.attributes.system_properties.space_id.clone(),
                    attribute: key,
                    value: triple.value,
                    value_type: triple.value_type.into(),
                    options: Options {
                        format: triple.options.format,
                        unit: triple.options.unit,
                        language: triple.options.language,
                    },
                })
                .collect(),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Entity object
impl Entity {
    /// Entity ID
    fn id(&self) -> &str {
        &self.id
    }

    /// Entity name (if available)
    fn name(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::NAME)
            .map(|triple| triple.value.as_str())
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        &self.space_id
    }

    fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    fn created_at_block(&self) -> &str {
        &self.created_at_block
    }

    fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    fn updated_at_block(&self) -> &str {
        &self.updated_at_block
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        if self.types.contains(&system_ids::RELATION_TYPE.to_string()) {
            // Since relations are also entities, and a relation's types are modelled differently
            // in Neo4j, we need to check fetch types differently if the entity is a relation.
            // mapping::Relation::<mapping::Triples>::find_types(
            //     &executor.context().0.neo4j,
            //     &self.id,
            //     &self.space_id,
            // )
            // .await
            // .expect("Failed to find relations")
            // .into_iter()
            // .map(|rel| rel.into())
            // .collect::<Vec<_>>()

            // For now, we'll just return the relation type
            mapping::Entity::<mapping::Triples>::find_by_id(
                &executor.context().0.neo4j,
                system_ids::RELATION_TYPE,
                &self.space_id,
            )
            .await
            .expect("Failed to find types")
            .map(|rel| vec![rel.into()])
            .unwrap_or(vec![])
        } else {
            mapping::Entity::<mapping::Triples>::find_types(
                &executor.context().0.neo4j,
                &self.id,
                &self.space_id,
            )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
        }
    }

    /// Attributes of the entity
    fn attributes(&self) -> &[Triple] {
        &self.attributes
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        mapping::Entity::<mapping::Triples>::find_relations::<mapping::Triples>(
            &executor.context().0.neo4j,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }
}

impl From<mapping::ValueType> for ValueType {
    fn from(value_type: mapping::ValueType) -> Self {
        match value_type {
            mapping::ValueType::Text => Self::Text,
            mapping::ValueType::Number => Self::Number,
            mapping::ValueType::Checkbox => Self::Checkbox,
            mapping::ValueType::Url => Self::Url,
            mapping::ValueType::Time => Self::Time,
            mapping::ValueType::Point => Self::Point,
        }
    }
}

#[derive(Debug)]
pub struct Relation {
    id: String,
    relation_types: Vec<String>,
    space_id: String,
    created_at: DateTime<Utc>,
    created_at_block: String,
    updated_at: DateTime<Utc>,
    updated_at_block: String,
    attributes: Vec<Triple>,
}

impl From<mapping::Relation<mapping::Triples>> for Relation {
    fn from(relation: mapping::Relation<mapping::Triples>) -> Self {
        Self {
            id: relation.attributes.id,
            relation_types: relation.types,
            space_id: relation.attributes.system_properties.space_id.clone(),
            created_at: relation.attributes.system_properties.created_at,
            created_at_block: relation
                .attributes
                .system_properties
                .created_at_block
                .clone(),
            updated_at: relation.attributes.system_properties.updated_at,
            updated_at_block: relation
                .attributes
                .system_properties
                .updated_at_block
                .clone(),
            attributes: relation
                .attributes
                .attributes
                .iter()
                .map(|(key, triple)| Triple {
                    // entiti: triple.entity,
                    space_id: relation.attributes.system_properties.space_id.clone(),
                    attribute: key.to_string(),
                    value: triple.value.clone(),
                    value_type: triple.value_type.clone().into(),
                    options: Options {
                        format: triple.options.format.clone(),
                        unit: triple.options.unit.clone(),
                        language: triple.options.language.clone(),
                    },
                })
                .collect(),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Relation object
///
/// Note: Relations are also entities, but they have a different structure in the database.
/// In other words, the Relation object is a "view" on a relation entity. All relations
/// can also be queried as entities.
impl Relation {
    /// Relation ID
    fn id(&self) -> &str {
        &self.id
    }

    /// Relation name (if available)
    fn name(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::NAME)
            .map(|triple| triple.value.as_str())
    }

    fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    fn created_at_block(&self) -> &str {
        &self.created_at_block
    }

    fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    fn updated_at_block(&self) -> &str {
        &self.updated_at_block
    }

    /// Attributes of the relation
    fn attributes(&self) -> &[Triple] {
        &self.attributes
    }

    /// Relation types of the relation
    async fn relation_types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        mapping::Entity::<mapping::Triples>::find_by_ids(
            &executor.context().0.neo4j,
            &self.relation_types,
            &self.space_id,
        )
        .await
        .expect("Failed to find types")
        .into_iter()
        .filter(|rel| rel.id() != system_ids::RELATION_TYPE)
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }

    /// Entity from which the relation originates
    async fn from<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        mapping::Relation::<mapping::Triples>::find_from::<mapping::Triples>(
            &executor.context().0.neo4j,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find node")
        .map(Entity::from)
        .unwrap()
    }

    /// Entity to which the relation points
    async fn to<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        mapping::Relation::<mapping::Triples>::find_to::<mapping::Triples>(
            &executor.context().0.neo4j,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find node")
        .map(Entity::from)
        .unwrap()
    }

    /// Relations outgoing from the relation
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        mapping::Entity::<mapping::Triples>::find_relations::<mapping::Triples>(
            &executor.context().0.neo4j,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
struct Triple {
    space_id: String,
    attribute: String,
    value: String,
    value_type: ValueType,
    options: Options,
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Triple {
    /// Attribute ID of the triple
    fn attribute(&self) -> &str {
        &self.attribute
    }

    /// Value of the triple
    fn value(&self) -> &str {
        &self.value
    }

    /// Value type of the triple
    fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// Options of the triple (if any)
    fn options(&self) -> &Options {
        &self.options
    }

    /// Name of the attribute (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        mapping::Entity::<mapping::Named>::find_by_id(
            &executor.context().0.neo4j,
            &self.attribute,
            &self.space_id,
        )
        .await
        .expect("Failed to find attribute entity")
        .and_then(|entity| entity.name())
    }
}

#[derive(Debug, GraphQLEnum)]
pub enum ValueType {
    Text,
    Number,
    Checkbox,
    Url,
    Time,
    Point,
}

#[derive(Debug, GraphQLObject)]
struct Options {
    pub format: Option<String>,
    pub unit: Option<String>,
    pub language: Option<String>,
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
