use axum::{extract::Json, http::StatusCode};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{TryStreamExt, future::join_all, pin_mut};
use grc20_core::{
    entity::{self, Entity, EntityFilter, EntityNode},
    mapping::{
        Query, QueryStream, RelationEdge, prop_filter,
        triple::{self, SemanticSearchResult},
    },
    neo4rs,
    relation::{self, RelationFilter},
    system_ids,
};
use grc20_sdk::models::BaseEntity;
use regex::Regex;
use rig::{agent::Agent, completion::Prompt, providers::gemini::completion::CompletionModel};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

pub struct AutomaticSearchAgent {
    neo4j: neo4rs::Graph,
    pub embedding_model: Arc<TextEmbedding>,
    pub fast_agent: Agent<CompletionModel>,
    pub thinking_agent: Agent<CompletionModel>,
}

impl AutomaticSearchAgent {
    pub fn new(neo4j: neo4rs::Graph, gemini_api_key: &str) -> Self {
        let preamble = "You are a knowledge graph helper to answer natural language questions.";
        let gemini_client = rig::providers::gemini::Client::new(gemini_api_key);

        Self {
            neo4j,
            embedding_model: Arc::new(
                TextEmbedding::try_new(
                    InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
                )
                .expect("Failed to initialize embedding model"),
            ),
            fast_agent: gemini_client
                .agent("gemini-2.5-flash-lite")
                .temperature(0.4)
                .preamble(preamble)
                .build(),
            thinking_agent: gemini_client
                .agent("gemini-2.5-flash")
                .temperature(0.4)
                .preamble(preamble)
                .build(),
        }
    }

    pub async fn natural_language_question(
        &self,
        question: String,
    ) -> Result<Json<String>, StatusCode> {
        let string_extraction_regex = Regex::new(r#""([^"]+)""#).unwrap();

        let create_relation_filter = |search_result: SemanticSearchResult| {
            RelationFilter::default()
                .from_(EntityFilter::default().id(prop_filter::value(search_result.triple.entity)))
        };

        let main_entity = self.fast_agent.prompt(format!("Can you extract the main 1-3 Entities from which you can do a search from the original question. THE ANSWER SHOULD BE A SINGLE WORD OR A SINGLE CONCEPT IN QUOTATION MARKS. IF THERE IS MORE THAN ONE IMPORTANT CONCEPT YOU CAN EXTRACT THEM EACH IN THEIR OWN QUOTATION MARKS. Here's the question: {question}")).await.unwrap_or("".to_string());

        let entities: Vec<String> = string_extraction_regex
            .captures_iter(&main_entity)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let number_answers = entities.len();

        tracing::info!("important word found: {entities:?}");

        let important_concepts = self.fast_agent.prompt(format!("Can you extract the all keywords and important concepts from the original question. You can also add keywords related that aren't directly in the question. THE ANSWER SHOULD BE A SERIE OF SINGLE WORD OR A SINGLE CONCEPT IN QUOTATION MARKS. OUTPUT THEM EACH IN THEIR OWN QUOTATION MARKS. Here's the question: {question}")).await.unwrap_or("".to_string());

        let concepts: Vec<String> = string_extraction_regex
            .captures_iter(&important_concepts)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        tracing::info!("important concepts are: {concepts:?}");

        let concepts_embeddings: Vec<Vec<f64>> = concepts
            .into_iter()
            .map(|concept| {
                self.embedding_model
                    .embed(vec![concept], None)
                    .expect("Failed to get embedding")
                    .pop()
                    .expect("Embedding is empty")
                    .into_iter()
                    .map(|v| v as f64)
                    .collect::<Vec<_>>()
            })
            .collect();

        let expanded_paths = join_all(
            join_all(entities.into_iter().map(|entity| async {
                let semantic_search = self.search(entity, Some(1)).await.unwrap_or_default();
                self.get_ids_from_search(semantic_search, create_relation_filter)
                    .await
                    .unwrap_or_default()
            }))
            .await
            .into_iter()
            .flatten()
            .map(|sart_node_id| async {
                self.automatic_explore_node(sart_node_id, concepts_embeddings.clone())
                    .await
            }),
        )
        .await
        .to_vec();

        let answers_combined = join_all(expanded_paths.iter().map(|path| async {
                let agent_prompt = format!(
                    "From the search that was done can you provide a final answer? Here's the original question: {question}\nThe full search of nodes:{}", path.clone()
                );

                self.thinking_agent
                    .prompt(agent_prompt)
                    .await.unwrap_or("Agent error".to_string())
            })).await.join("]\n[");

        let final_answer = if number_answers == 1 {
            answers_combined
        } else {
            let final_answer_prompt = format!(
                "BASE YOUR ANSWER ONLY ON THE PARTIAL ANSWERS! YOU ANSWER ONLY THE ORIGINAL QUESTION SINCE THE MAIN USER DOESN'T CARE ABOUT PARTIAL ANSWERS. GIVE A FULL COMPLETE AND DETAILLED ANSWER. From the given partial answer from different starting points, can you provide a final single answer to the original question: {question}\n Here are the different partial answers:[{answers_combined}]"
            );
            self.fast_agent
                .prompt(final_answer_prompt)
                .await
                .map_err(|e| {
                    tracing::error!("Error: {e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
        };

        Ok(Json(final_answer))
    }

    async fn automatic_explore_node(
        &self,
        start_id: String,
        concepts_embeddings: Vec<Vec<f64>>,
    ) -> String {
        let mut seen_ids = HashSet::new();

        let start_name = self
            .get_name_of_id(start_id.clone())
            .await
            .unwrap_or("No name".to_string());

        let mut path_root = PathNode {
            id: start_id.clone(),
            relation_name: "Start".to_string(),
            relation_name_embedding: Vec::new(),
            entity_name: start_name,
            entity_name_embedding: Vec::new(),
            nodes: Vec::new(),
            is_inbound: false,
            page_rank: 0.0,
            depth: 0,
        };

        let mut stack: Vec<&mut PathNode> = vec![&mut path_root];

        while let Some(current) = stack.pop() {
            if !seen_ids.insert(current.id.clone()) {
                continue;
            }

            let neighbor_nodes = self
                .automatic_expand_entity(
                    current.id.clone(),
                    current.depth,
                    concepts_embeddings.clone(),
                )
                .await
                .unwrap_or_default();

            for neighbor in neighbor_nodes {
                current.nodes.push(PathNode {
                    nodes: Vec::new(),
                    id: neighbor.id.clone(),
                    depth: neighbor.depth + 1,
                    ..neighbor
                });
            }

            current
                .nodes
                .iter_mut()
                .for_each(|neighbor| stack.insert(0, neighbor));
        }

        self.create_path(&path_root, 0).await
    }

    async fn automatic_expand_entity(
        &self,
        base_id: String,
        depth: usize,
        concepts_embeddings: Vec<Vec<f64>>,
    ) -> Result<Vec<PathNode>, StatusCode> {
        const MAX_SAME_RELATION: usize = 10;
        const NAME_IMPORTANCE_FACTOR: f64 = 0.9;
        const MAX_DISPLAYED_NEIGHBORS: usize = 35;
        const EMBEDDING_DISTANCE_THRESHOLD: f64 = 0.65;

        let mut relation_count = HashMap::new();

        let mut relations = self
            .extract_path_nodes_to_neighbors(base_id.clone(), depth, false)
            .await
            .unwrap_or_default();
        relations.extend(
            self.extract_path_nodes_to_neighbors(base_id.clone(), depth, true)
                .await
                .unwrap_or_default(),
        );

        let mut relations_with_scores: Vec<(f64, PathNode)> = relations
            .into_iter()
            .map(|path_node| {
                // Compute all cosine distances to concept embeddings and take the best (lowest)
                let distance_score = concepts_embeddings
                    .iter()
                    .map(|concept| {
                        let name_dist =
                            self.cosine_distance_unit(concept, &path_node.entity_name_embedding);
                        let rel_dist =
                            self.cosine_distance_unit(concept, &path_node.relation_name_embedding);
                        name_dist.min(rel_dist) * NAME_IMPORTANCE_FACTOR
                            + name_dist.max(rel_dist) * (1.0 - NAME_IMPORTANCE_FACTOR)
                    })
                    .fold(f64::INFINITY, |a, b| a.min(b));

                let depth_penalty = 0.02 * depth as f64;
                let depth_centered = 3.5 - depth as f64;
                let depth_factor = depth_centered.powf(1.0 / 5.0) * 20.0;
                let page_rank_clipped = path_node.page_rank.clamp(-0.5, 0.8);
                let final_score =
                    depth_penalty + distance_score * (0.9 - page_rank_clipped / depth_factor); // /depth_factor

                (final_score, path_node)
            })
            .collect();

        relations_with_scores
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Less));

        let relations: Vec<PathNode> = relations_with_scores
            .into_iter()
            .filter(|(score, _)| score < &EMBEDDING_DISTANCE_THRESHOLD)
            .filter_map(|(_, path_node)| {
                let count = relation_count
                    .entry(path_node.relation_name.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                if *count <= MAX_SAME_RELATION {
                    Some(path_node)
                } else {
                    None
                }
            })
            .take(MAX_DISPLAYED_NEIGHBORS)
            .collect();

        Ok(relations)
    }

    async fn search(
        &self,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<SemanticSearchResult>, StatusCode> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let limit = limit.unwrap_or(10);

        let semantic_search_triples = triple::search(&self.neo4j, embedding)
            .limit(limit)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(semantic_search_triples)
    }

    async fn get_ids_from_search(
        &self,
        search_triples: Vec<SemanticSearchResult>,
        create_relation_filter: impl Fn(SemanticSearchResult) -> RelationFilter,
    ) -> Result<Vec<String>, StatusCode> {
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut result_ids: Vec<String> = Vec::new();

        for semantic_search_triple in search_triples {
            let filtered_for_types = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
                .filter(create_relation_filter(semantic_search_triple))
                .send()
                .await;

            //We only need to get the first relation since they would share the same entity id
            if let Ok(stream) = filtered_for_types {
                pin_mut!(stream);
                if let Some(edge) = stream.try_next().await.ok().flatten() {
                    let id = edge.from.id;
                    if seen_ids.insert(id.clone()) {
                        result_ids.push(id);
                    }
                }
            }
        }
        Ok(result_ids)
    }

    async fn extract_path_nodes_to_neighbors(
        &self,
        id: String,
        depth: usize,
        is_inbound: bool,
    ) -> Result<Vec<PathNode>, StatusCode> {
        const MAX_RELATIONS_CONSIDERED: usize = 100;

        let in_filter = relation::RelationFilter::default()
            .to_(EntityFilter::default().id(prop_filter::value(id.clone())));
        let out_filter = relation::RelationFilter::default()
            .from_(EntityFilter::default().id(prop_filter::value(id.clone())));

        let relations = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
            .filter(if is_inbound { in_filter } else { out_filter })
            .limit(MAX_RELATIONS_CONSIDERED)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let related_nodes = join_all(relations.into_iter().map(|result| async move {
            let page_rank: f64;
            let id: &str;

            if is_inbound {
                id = &result.from.id;
                page_rank = result.from.system_properties.page_rank.unwrap_or(0.0);
            } else {
                id = &result.to.id;
                page_rank = result.to.system_properties.page_rank.unwrap_or(0.0);
            };
            let (entity_name_embedding, entity_name) = self
                .get_name_and_embedding_of_id(id.to_string())
                .await
                .unwrap_or((Vec::new(), "No name".to_string()));
            let (relation_name_embedding, relation_name) = self
                .get_name_and_embedding_of_id(result.relation_type.clone())
                .await
                .unwrap_or((Vec::new(), "No name".to_string()));

            PathNode {
                id: id.to_string(),
                relation_name,
                relation_name_embedding,
                entity_name,
                entity_name_embedding,
                nodes: Vec::new(),
                is_inbound,
                page_rank,
                depth: depth + 1,
            }
        }))
        .await
        .to_vec();

        Ok(related_nodes)
    }

    async fn create_path(&self, path_node: &PathNode, depth: usize) -> String {
        const MAX_DISPLAY_LENGTH: usize = 8;

        let shorten_string = |word: String, max_length: usize| {
            let splitted: Vec<&str> = word.split_whitespace().collect();
            if splitted.len() > max_length {
                splitted[..max_length].join(" ") + "..."
            } else {
                splitted.join(" ")
            }
        };

        let mut base = format!(
            "{}{}-[{}]-{}{}\n",
            "  ".repeat(depth),
            if path_node.is_inbound { "<" } else { "" },
            shorten_string(path_node.relation_name.clone(), MAX_DISPLAY_LENGTH),
            if path_node.is_inbound { "" } else { ">" },
            shorten_string(path_node.entity_name.clone(), MAX_DISPLAY_LENGTH),
        );

        let rest = join_all(
            path_node
                .nodes
                .iter()
                .map(|node| async { self.create_path(node, depth + 1).await }),
        )
        .await
        .join("");

        base.push_str(&rest);
        base
    }

    async fn get_name_of_id(&self, id: String) -> Result<String, StatusCode> {
        let entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, &id)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        entity
            .attributes
            .name
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn get_name_and_embedding_of_id(
        &self,
        id: String,
    ) -> Result<(Vec<f64>, String), StatusCode> {
        let triple_result = triple::find_many(&self.neo4j)
            .entity_id(prop_filter::value(&id))
            .attribute_id(prop_filter::value(system_ids::NAME_ATTRIBUTE))
            .limit(1)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        if let Some(name_entity) = triple_result.first()
            && let Some(embedding) = &name_entity.embedding
        {
            return Ok((embedding.clone(), name_entity.value.value.clone()));
        }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }

    #[inline(always)]
    fn cosine_distance_unit(&self, a: &[f64], b: &[f64]) -> f64 {
        let mut dot = 0.0;
        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * y;
        }
        1.0 - dot
    }
}

#[derive(Clone, Debug)]
struct PathNode {
    id: String,
    relation_name: String,
    relation_name_embedding: Vec<f64>,
    entity_name: String,
    entity_name_embedding: Vec<f64>,
    nodes: Vec<PathNode>,
    is_inbound: bool,
    page_rank: f64,
    depth: usize,
}
