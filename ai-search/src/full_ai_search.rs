use anyhow::Error;
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
};
use grc20_sdk::models::BaseEntity;
use rand::{Rng, distributions::Alphanumeric};
use regex::Regex;
use rig::{agent::Agent, completion::Prompt, providers::gemini::completion::CompletionModel};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::Mutex;

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

pub struct FullAISearchAgent {
    neo4j: neo4rs::Graph,
    pub embedding_model: Arc<TextEmbedding>,
    pub fast_agent: Agent<CompletionModel>,
    pub thinking_agent: Agent<CompletionModel>,
}

#[derive(Clone, Debug)]
struct PathNode {
    id: String,
    relation_name: String,
    entity_name: String,
    nodes: Vec<PathRef>,
    is_explored: bool,
    is_inbound: bool,
    is_hidden: bool,
    depth: usize,
}

type PathRef = Arc<Mutex<PathNode>>;

impl FullAISearchAgent {
    pub fn new(neo4j: neo4rs::Graph, gemini_api_key: &str) -> Self {
        let traversal_system_prompt = include_str!("../ressources/traversal_prompt.md");
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
                .preamble(traversal_system_prompt)
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

        let main_entity = self.thinking_agent.prompt(format!("Can you extract the main 1-3 Entities from which you can do a search from the original question. THE ANSWER SHOULD BE A SINGLE WORD OR A SINGLE CONCEPT IN QUOTATION MARKS. IF THERE IS MORE THAN ONE IMPORTANT CONCEPT YOU CAN EXTRACT THEM EACH IN THEIR OWN QUOTATION MARKS. Here's the question: {question}")).await.unwrap_or("".to_string());

        let entities: Vec<String> = string_extraction_regex
            .captures_iter(&main_entity)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let number_answers = entities.len();

        tracing::info!("important word found: {entities:?}");

        let important_concepts = self.thinking_agent.prompt(format!("Can you extract the all keywords and important concepts from the original question. You can also add keywords related that aren't directly in the question. THE ANSWER SHOULD BE A SERIE OF SINGLE WORD OR A SINGLE CONCEPT IN QUOTATION MARKS. OUTPUT THEM EACH IN THEIR OWN QUOTATION MARKS. Here's the question: {question}")).await.unwrap_or("".to_string());

        let concepts: Vec<String> = string_extraction_regex
            .captures_iter(&important_concepts)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        tracing::info!("important concepts are: {concepts:?}");

        let concepts_embeddings: Vec<Vec<f64>> = self
            .embedding_model
            .embed(concepts, None)
            .expect("Failed to get embedding")
            .into_iter()
            .map(|vec| vec.into_iter().map(|v| v as f64).collect())
            .collect::<Vec<_>>();

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
            .map(|start_node_id| async {
                self.explore_node(question.clone(), concepts_embeddings.clone(), start_node_id)
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
                    .await
                    .unwrap_or("Agent error".to_string())
            })).await
            .join("]\n[");

        let final_answer = if number_answers == 1 {
            Ok(answers_combined)
        } else {
            let final_answer_prompt = format!(
                "BASE YOUR ANSWER ONLY ON THE PARTIAL ANSWERS! YOU ANSWER ONLY THE ORIGINAL QUESTION SINCE THE MAIN USER DOESN'T CARE ABOUT PARTIAL ANSWERS. GIVE A FULL COMPLETE AND DETAILLED ANSWER. From the given partial answer from different starting points, can you provide a final single answer to the original question: {question}\n Here are the different partial answers:[{answers_combined}]"
            );
            self.thinking_agent.prompt(final_answer_prompt).await
        };

        match final_answer {
            Ok(answer) => Ok(Json(answer)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    async fn create_path(&self, path_node: &PathNode, depth: usize) -> String {
        if path_node.is_hidden || depth > 10 {
            return "".to_string();
        }
        const MAX_DISPLAY_LENGTH: usize = 8;

        let shorten_string = |word: String, max_length: usize| {
            let splitted: Vec<&str> = word.split_whitespace().collect();
            if splitted.len() > max_length && !path_node.is_explored {
                splitted[..max_length].join(" ") + "..."
            } else {
                splitted.join(" ")
            }
        };

        let mut base = format!(
            "{}{}-[{}]-{}{}{}({})\n",
            "  ".repeat(depth),
            if path_node.is_inbound { "<" } else { "" },
            shorten_string(path_node.relation_name.clone(), MAX_DISPLAY_LENGTH),
            if path_node.is_inbound { "" } else { ">" },
            if path_node.is_explored { "+" } else { "?" },
            shorten_string(path_node.entity_name.clone(), MAX_DISPLAY_LENGTH),
            path_node.id.chars().take(6).collect::<String>(),
        );

        let rest = join_all(path_node.nodes.iter().map(|node| async {
            self.create_path(&node.lock().await.clone(), depth + 1)
                .await
        }))
        .await
        .join("");

        base.push_str(&rest);
        base
    }

    async fn explore_node(
        &self,
        question: String,
        concepts_embeddings: Vec<Vec<f64>>,
        start_id: String,
    ) -> String {
        let explore_regex = Regex::new(r"<explore>(.*?)</explore>").unwrap();
        let answer_regex = Regex::new(r"<answer>(.*?)</answer>").unwrap();
        let end_regex = Regex::new(r"</end>").unwrap();

        let mut full_id_resolver: HashMap<String, String> = HashMap::new();
        let mut expansions: HashMap<String, Vec<PathRef>> = HashMap::new();
        let mut seen_ids = HashSet::new();
        let mut seen_twice = HashSet::new();
        let mut explores = Vec::new();
        let mut partial_answers = Vec::new();
        let mut potential_explore = Vec::new();

        let start_name = self
            .get_name_of_id(start_id.clone())
            .await
            .unwrap_or("No name".to_string());

        let path_root: PathRef = Arc::new(Mutex::new(PathNode {
            id: start_id.clone(),
            relation_name: "Start".to_string(),
            entity_name: start_name,
            nodes: Vec::new(),
            is_explored: false,
            is_inbound: false,
            is_hidden: false,
            depth: 0,
        }));

        potential_explore.push(path_root.clone());

        let mut stack: Vec<PathRef> = vec![path_root.clone()];
        full_id_resolver.insert(start_id.chars().take(6).collect(), start_id);

        while let Some(current) = stack.pop() {
            let current_node = current.lock().await;
            let mut warning = false;

            if !seen_ids.insert(current_node.id.clone()) {
                warning = true;
                if !seen_twice.insert(current_node.id.clone()) {
                    continue;
                }
            }

            if partial_answers.len() >= 10 {
                break;
            }

            let data = format!(
                "name: {}, description:{}",
                current_node.entity_name,
                self.get_description_of_id(current_node.id.clone())
                    .await
                    .unwrap_or("No description".to_string())
            );

            let (neighbor_nodes, hidden_options) = self
                .expand_entity(current_node.id.clone(), concepts_embeddings.clone())
                .await
                .unwrap_or_default();

            drop(current_node);
            let mut current_node_mut = current.lock().await;
            current_node_mut.is_explored = true;

            for neighbor in neighbor_nodes.clone() {
                let new_node = Arc::new(Mutex::new(PathNode {
                    nodes: Vec::new(),
                    is_explored: seen_ids.contains(&neighbor.id.clone()),
                    id: neighbor.id.clone(),
                    depth: neighbor.depth + 1,
                    ..neighbor
                }));
                full_id_resolver.insert(neighbor.id.chars().take(6).collect(), neighbor.id);

                if !warning && !neighbor.is_explored {
                    current_node_mut.nodes.push(new_node.clone());
                    potential_explore.push(new_node);
                }
            }

            for (relation_name, hidden_path_nodes) in hidden_options {
                let expand_id: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(6)
                    .map(char::from)
                    .collect();

                let expansion_node = Arc::new(Mutex::new(PathNode {
                    id: expand_id.clone(),
                    relation_name: relation_name.clone(),
                    entity_name: format!(
                        "Expand {} more {relation_name}...",
                        hidden_path_nodes.len()
                    ),
                    nodes: Vec::new(),
                    is_explored: false,
                    is_inbound: false,
                    is_hidden: false,
                    depth: 0,
                }));
                if !warning {
                    current_node_mut.nodes.push(expansion_node.clone());
                }

                for neighbor in hidden_path_nodes {
                    let new_node = Arc::new(Mutex::new(PathNode {
                        nodes: Vec::new(),
                        is_explored: seen_ids.contains(&neighbor.id.clone()),
                        id: neighbor.id.clone(),
                        is_hidden: true,
                        ..neighbor
                    }));
                    full_id_resolver.insert(neighbor.id.chars().take(6).collect(), neighbor.id);
                    expansions
                        .entry(expand_id.clone())
                        .or_default()
                        .push(new_node.clone());

                    if !warning && !neighbor.is_explored {
                        current_node_mut.nodes.push(new_node.clone());
                        potential_explore.push(new_node);
                    }
                }
            }

            drop(current_node_mut);

            let named_path: String = {
                let snapshot_of_path = path_root.lock().await;
                self.create_path(&snapshot_of_path, 0).await
            };

            let warning_message = if warning {
                "YOU HAVE ALREADY SEEN THIS NODE! YOU WON'T BE ABLE TO SEE IT AGAIN! SEEING IT AGAIN WON'T GIVE YOU MORE INFORMATION."
            } else {
                ""
            };

            let agent_prompt = format!(
                "{warning_message}\nThe current explored node is:\n{data}\nThe original question is:\n{question}\nThe full exploration is:{named_path}"
            );

            let prompt_answer = self
                .fast_agent
                .prompt(agent_prompt)
                .await
                .unwrap_or("Agent error".to_string());

            tracing::info!("Agent answer: {prompt_answer}");

            for explore in explore_regex.captures_iter(&prompt_answer) {
                if let Some(full_id) = full_id_resolver.get(&explore[1]) {
                    explores.push(full_id.clone());
                }
                if let Some(hidden_nodes) = expansions.get(&explore[1]) {
                    let mut first = true;
                    for node in hidden_nodes {
                        let mut current_node_mut = node.lock().await;
                        current_node_mut.is_hidden = false;
                        if first {
                            explores.push(current_node_mut.id.clone());
                            first = false;
                        }
                        drop(current_node_mut);
                    }
                }
            }

            if let Some(answer) = answer_regex.captures(&prompt_answer) {
                partial_answers.push(answer[1].to_string());
                stack.push(current.clone());
            }

            if end_regex.captures(&prompt_answer).is_some() {
                break;
            }

            for potential_exploration_node in &potential_explore {
                let node = potential_exploration_node.lock().await;

                let insert_node = explores.contains(&node.id);
                explores.retain(|item| item != &node.id);
                drop(node);
                if insert_node {
                    // BFS
                    stack.insert(0, potential_exploration_node.clone());
                }
            }
        }

        let named_path: String = {
            let snapshot_of_path = path_root.lock().await;
            self.create_path(&snapshot_of_path, 0).await
        };

        let agent_prompt = format!(
            "From the search that was done can you provide a final answer? Here's the original question: {question}\nThe partial_answers are:[{}]\nThe full path of nodes:{named_path}",
            partial_answers.join("]\n[")
        );

        self.thinking_agent
            .prompt(agent_prompt)
            .await
            .unwrap_or("Agent error".to_string())
    }

    async fn expand_entity(
        &self,
        base_id: String,
        concepts_embeddings: Vec<Vec<f64>>,
    ) -> Result<(Vec<PathNode>, HashMap<String, Vec<PathNode>>), Error> {
        const IMPORTANCE_FACTOR: f64 = 0.9;
        const MAX_SAME_RELATION: usize = 4;
        const MAX_DIPLAYED_NEIGHBORS: usize = 15;
        const EMBEDDING_DISTANCE_THRESHOLD: f64 = 1.5;

        let mut relation_count: HashMap<String, usize> = HashMap::new();
        let mut hidden_options: HashMap<String, Vec<PathNode>> = HashMap::new();

        let mut relations = self
            .extract_path_nodes_to_neighbors(base_id.clone(), 0, false)
            .await
            .unwrap_or_default();

        relations.extend(
            self.extract_path_nodes_to_neighbors(base_id.clone(), 0, true)
                .await
                .unwrap_or_default(),
        );

        let mut relations_with_scores: Vec<(f64, PathNode)> = relations
            .into_iter()
            .map(|path_node| {
                let name_embedding = self.create_embedding(vec![&path_node.entity_name]);
                let relation_embedding = self.create_embedding(vec![&path_node.relation_name]);

                // Compute all cosine distances to concept embeddings and take the best (lowest)
                let min_name_distance = concepts_embeddings
                    .iter()
                    .map(|concept| self.cosine_distance(concept, &name_embedding))
                    .fold(f64::INFINITY, |a, b| a.min(b));

                // Compute all cosine distances to concept embeddings and take the best (lowest)
                let min_rel_distance = concepts_embeddings
                    .iter()
                    .map(|concept| self.cosine_distance(concept, &relation_embedding))
                    .fold(f64::INFINITY, |a, b| a.min(b));

                // Many times, the same relation name is used. It now differentiate on the best entity name
                let distance_score = if min_name_distance < min_rel_distance {
                    min_name_distance * IMPORTANCE_FACTOR
                        + min_rel_distance * (1.0 - IMPORTANCE_FACTOR)
                } else {
                    min_name_distance * (1.0 - IMPORTANCE_FACTOR)
                        + min_rel_distance * IMPORTANCE_FACTOR
                };

                (distance_score, path_node)
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
                    hidden_options
                        .entry(path_node.relation_name.clone())
                        .or_default()
                        .push(path_node);
                    None
                }
            })
            .take(MAX_DIPLAYED_NEIGHBORS)
            .collect();

        Ok((relations, hidden_options))
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
            let id = if is_inbound {
                result.from.id
            } else {
                result.to.id
            };

            let entity_name = self
                .get_name_of_id(id.to_string())
                .await
                .unwrap_or("No name".to_string());

            let relation_name = self
                .get_name_of_id(result.relation_type.clone())
                .await
                .unwrap_or("No name".to_string());

            PathNode {
                id: id.to_string(),
                relation_name,
                entity_name,
                nodes: Vec::new(),
                is_inbound,
                is_explored: false,
                is_hidden: false,
                depth: depth + 1,
            }
        }))
        .await
        .to_vec();

        Ok(related_nodes)
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

    async fn get_description_of_id(&self, id: String) -> Result<String, StatusCode> {
        let entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, &id)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(entity
            .attributes
            .description
            .unwrap_or("No description".to_string()))
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

    fn cosine_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        1.0 - (dot_product / (norm_a * norm_b))
    }
}
