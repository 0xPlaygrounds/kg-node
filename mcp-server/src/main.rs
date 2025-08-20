use clap::{Args, Parser};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{TryStreamExt, future::join_all};
use grc20_core::{
    entity::{
        self, Entity, EntityFilter, EntityNode, EntityRelationFilter, utils::TraverseRelation,
    },
    mapping::{
        Query, QueryStream, RelationEdge, prop_filter, query_utils::RelationDirection, triple,
    },
    neo4rs, relation, system_ids,
};
use grc20_sdk::models::BaseEntity;
use mcp_server::input_types::{self, SearchTraversalInputFilter};
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
    tool,
    transport::sse_server::{SseServer, SseServerConfig},
};
use serde_json::{Value, json};
use std::{collections::HashSet, sync::Arc, time::Instant, vec};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

const BIND_ADDRESS: &str = "0.0.0.0:8080";

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

    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    let (sse_server, router) = SseServer::new(config);

    // Do something with the router, e.g., add routes or middleware

    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;

    let ct = sse_server.config.ct.child_token();

    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse server cancelled");
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "sse server shutdown with error");
        }
    });

    let ct = sse_server.with_service(move || KnowledgeGraph::new(neo4j.clone()));

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

#[derive(Clone)]
pub struct KnowledgeGraph {
    neo4j: neo4rs::Graph,
    pub embedding_model: Arc<TextEmbedding>,
}

#[tool(tool_box)]
impl KnowledgeGraph {
    #[allow(dead_code)]
    pub fn new(neo4j: neo4rs::Graph) -> Self {
        Self {
            neo4j,
            embedding_model: Arc::new(
                TextEmbedding::try_new(
                    InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
                )
                .expect("Failed to initialize embedding model"),
            ),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    // Tools that are available in the mcp server
    #[tool(description = include_str!("../resources/search_type_description.md"))]
    async fn search_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let filter_type = EntityFilter::default().relations(
            EntityRelationFilter::default()
                .relation_type(prop_filter::value(system_ids::TYPES_ATTRIBUTE))
                .to_id(prop_filter::value(system_ids::SCHEMA_TYPE)),
        );

        let search_ids = self
            .query_search(query, None, filter_type)
            .await
            .unwrap_or_default();

        Ok(CallToolResult::success(
            join_all(search_ids.into_iter().map(|id| async {
                Content::json(self.format_from_id(id).await).expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_relation_type_description.md"))]
    async fn search_relation_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for relation types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let filter_relation_type = EntityFilter::default().relations(
            EntityRelationFilter::default()
                .relation_type(prop_filter::value(system_ids::VALUE_TYPE_ATTRIBUTE))
                .to_id(prop_filter::value(system_ids::RELATION_SCHEMA_TYPE)),
        );

        let search_ids = self
            .query_search(query, None, filter_relation_type)
            .await
            .unwrap_or_default();

        Ok(CallToolResult::success(
            join_all(search_ids.into_iter().map(|id| async {
                Content::json(self.format_from_id(id).await).expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_entity_using_ids_description.md"))]
    async fn search_entity_using_ids(
        &self,
        #[tool(param)]
        #[schemars(description = "A filter of the relation(s) to traverse from the query")]
        search_traversal_filter: SearchTraversalInputFilter,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("SearchTraversalFilter query: {:?}", search_traversal_filter);

        let embedding = self.create_embedding(vec![&search_traversal_filter.query]);

        let traversal_filters: Vec<Result<TraverseRelation, McpError>> =
            match search_traversal_filter.traversal_filter {
                Some(traversal_filter_input) => {
                    join_all(traversal_filter_input.into_iter().map(|filter| async move {
                        Ok(TraverseRelation::default()
                            .direction(match filter.direction {
                                Some(direction) => match direction {
                                    input_types::RelationDirection::From => RelationDirection::From,
                                    input_types::RelationDirection::To => RelationDirection::To,
                                },
                                None => RelationDirection::Both,
                            })
                            .relation_type_id(prop_filter::value(filter.relation_type)))
                    }))
                    .await
                    .to_vec()
                }
                None => Vec::new(),
            };

        let results_search = traversal_filters
            .into_iter()
            .fold(
                entity::traversal_search::<Entity<BaseEntity>>(&self.neo4j, embedding.clone()),
                |query, result_traversal_filter: Result<_, McpError>| match result_traversal_filter
                {
                    Ok(traversal_filter) => {
                        query.filter(EntityFilter::default().traverse_relation(traversal_filter))
                    }
                    Err(_) => query,
                },
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        Ok(CallToolResult::success(vec![
            Content::json(
                results_search
                    .into_iter()
                    .map(|result| {
                        json!({
                            "id": result.entity.id(),
                            "name": result.entity.attributes.name,
                            "description": result.entity.attributes.description,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/search_entity_description.md"))]
    async fn search_entity(
        &self,
        #[tool(param)]
        #[schemars(description = "A filter of the relation(s) to traverse from the query")]
        search_traversal_filter: SearchTraversalInputFilter,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("SearchTraversalFilter query: {:?}", search_traversal_filter);

        let embedding = self.create_embedding(vec![&search_traversal_filter.query]);

        let start_filters = Instant::now();

        let traversal_filters: Vec<Result<TraverseRelation, McpError>> =
            match search_traversal_filter.traversal_filter {
                Some(traversal_filter_input) => {
                    join_all(traversal_filter_input.into_iter().map(|filter| async move {
                        let filter_relation_type = EntityFilter::default().relations(
                            EntityRelationFilter::default()
                                .relation_type(prop_filter::value(system_ids::VALUE_TYPE_ATTRIBUTE))
                                .to_id(prop_filter::value(system_ids::RELATION_SCHEMA_TYPE)),
                        );

                        let relation_ids = self
                            .query_search(filter.relation_type, None, filter_relation_type)
                            .await
                            .unwrap_or_default();

                        Ok(TraverseRelation::default()
                            .direction(match filter.direction {
                                Some(direction) => match direction {
                                    input_types::RelationDirection::From => RelationDirection::From,
                                    input_types::RelationDirection::To => RelationDirection::To,
                                },
                                None => RelationDirection::Both,
                            })
                            .relation_type_id(prop_filter::value_in(relation_ids)))
                    }))
                    .await
                    .to_vec()
                }
                None => Vec::new(),
            };

        let end_filters = Instant::now();

        let results_search = traversal_filters
            .into_iter()
            .fold(
                entity::traversal_search::<Entity<BaseEntity>>(&self.neo4j, embedding.clone()),
                |query, result_traversal_filter: Result<_, McpError>| match result_traversal_filter
                {
                    Ok(traversal_filter) => {
                        query.filter(EntityFilter::default().traverse_relation(traversal_filter))
                    }
                    Err(_) => query,
                },
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        let end_search = Instant::now();
        let search_time = end_search - end_filters;
        let filters_time = end_filters - start_filters;

        tracing::info!("filters time: {filters_time:?}");
        tracing::info!("search time: {search_time:?}");

        Ok(CallToolResult::success(vec![
            Content::json(
                results_search
                    .into_iter()
                    .map(|result| {
                        json!({
                            "id": result.entity.id(),
                            "name": result.entity.attributes.name,
                            "description": result.entity.attributes.description,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/get_entity_info_description.md"))]
    async fn get_entity_info(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Return an entity by its ID along with its attributes (name, description, etc.), relations and types"
        )]
        id: String,
    ) -> Result<CallToolResult, McpError> {
        let entity_attributes = triple::find_many(&self.neo4j)
            .entity_id(prop_filter::value(&id))
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
            })?;

        let out_relations = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
            .filter(
                relation::RelationFilter::default()
                    .from_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id_not_found",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        let in_relations = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
            .filter(
                relation::RelationFilter::default()
                    .to_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id_not_found",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        tracing::info!("Found entity with ID '{}'", id);

        let clean_up_relations = |relations: Vec<RelationEdge<EntityNode>>, is_inbound: bool| async move {
            join_all(relations
                .into_iter()
                .map(|result| async move {
                    json!({
                        "relation_id": result.relation_type,
                        "relation_type": self.get_name_of_id(result.relation_type).await.unwrap_or("No relation type".to_string()),
                        "id": if is_inbound {result.from.id.clone()} else {result.to.id.clone()},
                        "name": self.get_name_of_id(if is_inbound {result.from.id.clone()} else {result.to.id.clone()}).await.unwrap_or("No name".to_string()),
                    })
                })).await.to_vec()
        };
        let inbound_relations = clean_up_relations(in_relations, true).await;
        let outbound_relations = clean_up_relations(out_relations, false).await;

        let attributes_vec: Vec<_> = join_all(entity_attributes.into_iter().map(
            |attr| async {
                json!({
                    "attribute_name": self.get_name_of_id(attr.attribute).await.unwrap_or("No attribute name".to_string()),
                    "attribute_value": String::try_from(attr.value).unwrap_or("No attributes".to_string()),
                })
            },
        ))
        .await
        .to_vec();

        Ok(CallToolResult::success(vec![
            Content::json(json!({
                "id": id,
                "all_attributes": attributes_vec,
                "inbound_relations": inbound_relations,
                "outbound_relations": outbound_relations,
            }))
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/get_relations_between_entities_description.md"))]
    async fn get_relations_between_entities(
        &self,
        #[tool(param)]
        #[schemars(description = "The id of the first Entity to find relations")]
        entity1_id: String,
        #[tool(param)]
        #[schemars(description = "The id of the second Entity to find relations")]
        entity2_id: String,
    ) -> Result<CallToolResult, McpError> {
        let relations = entity::find_path(&self.neo4j, entity1_id.clone(), entity2_id.clone())
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_ids",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .into_iter()
            .collect::<Vec<_>>();

        Ok(CallToolResult::success(
            join_all(relations
                .into_iter()
                .map(|result| async {
                    Content::json(json!({
                    "nodes": join_all(result.nodes_ids.into_iter().map(|node_id| async {self.get_name_of_id(node_id).await.unwrap_or("No attribute name".to_string())})).await.to_vec(),
                    "relations": join_all(result.relations_ids.into_iter().map(|node_id| async {self.get_name_of_id(node_id).await.unwrap_or("No attribute name".to_string())})).await.to_vec(),
                    }))
                    .expect("Failed to create JSON content")
                }))
                .await
                .to_vec(),
        ))
    }

    #[inline(always)]
    fn create_embedding(&self, name: Vec<&str>) -> Vec<f64> {
        self.embedding_model
            .embed(name, None)
            .expect("Failed to get embedding")
            .pop()
            .expect("No embedding found")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>()
    }

    async fn query_search(
        &self,
        query: String,
        limit: Option<usize>,
        filter: EntityFilter,
    ) -> Result<Vec<String>, McpError> {
        let embedding = self.create_embedding(vec![&query]);

        let limit = limit.unwrap_or(10);
        let semantic_search_triples =
            entity::prefiltered_search::<EntityNode>(&self.neo4j, embedding.clone())
                .filter(filter.clone())
                .limit(limit)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Error: {e:?}");
                    McpError::internal_error(
                        "query_search_failed",
                        Some(json!({ "error": e.to_string() })),
                    )
                })?
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| {
                    tracing::error!("Error changing to vec: {e:?}");
                    McpError::internal_error(
                        "query_search_failed",
                        Some(json!({ "error": e.to_string() })),
                    )
                })?;

        let mut ids: Vec<String> = semantic_search_triples
            .into_iter()
            .map(|semantic_search_result| semantic_search_result.entity.id)
            .collect();

        self.dedup(&mut ids);

        Ok(ids)
    }

    fn dedup(&self, v: &mut Vec<String>) {
        let mut set = HashSet::new();

        v.retain(|x| set.insert(x.clone()));
    }

    async fn format_from_id(&self, id: String) -> Value {
        let mut value = json!({
            "entity_not_found_with_id": id.clone(),
        });

        let result_entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, id)
            .send()
            .await;

        if let Ok(Some(entity_node)) = result_entity {
            value = json!({
                "entity_id": entity_node.id(),
                "name": entity_node.attributes.name,
                "description": entity_node.attributes.description
            });
        };
        value
    }

    async fn get_name_of_id(&self, id: String) -> Result<String, McpError> {
        let entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, &id)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_name", Some(json!({ "error": e.to_string() })))
            })?
            .ok_or_else(|| {
                McpError::internal_error("entity_name_not_found", Some(json!({ "id": id })))
            })?;
        entity.attributes.name.ok_or_else(|| {
            McpError::internal_error("entity_name_not_found", Some(json!({ "id": id })))
        })
    }
}

#[tool(tool_box)]
impl ServerHandler for KnowledgeGraph {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(include_str!("../resources/instructions.md").to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
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
