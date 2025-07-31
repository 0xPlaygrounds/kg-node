use chrono::DateTime;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{
    block::BlockMetadata,
    entity::UpdateEntity,
    error::DatabaseError,
    ids::{create_geo_id, indexer_ids},
    neo4rs,
    pb::chain::GeoOutput,
    value::Value,
};
use ipfs::IpfsClient;
use prost::Message;
use substreams_utils::pb::sf::substreams::rpc::v2::BlockScopedData;
use uuid::Uuid;

use crate::{
    blacklist, metrics,
    preprocess::{self, EventData},
};
use cache::KgCache;
use std::sync::Arc;

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("IPFS error: {0}")]
    IpfsError(#[from] ipfs::Error),

    #[error("prost error: {0}")]
    Prost(#[from] prost::DecodeError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Neo4j error: {0}")]
    Neo4jError(#[from] neo4rs::Error),

    #[error("Cache error: {0}")]
    CacheError(#[from] cache::CacheError),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    // #[error("KG error: {0}")]
    // KgError(#[from] kg::Error),
    #[error("Error processing event: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct EventHandler {
    pub(crate) ipfs: IpfsClient,
    pub(crate) neo4j: neo4rs::Graph,
    #[allow(dead_code)]
    pub(crate) cache: Option<Arc<KgCache>>,
    pub(crate) spaces_blacklist: Vec<Uuid>,
    pub(crate) embedding_model: fastembed::TextEmbedding,
    pub(crate) embedding_model_dim: usize,

    // Handler config
    pub(crate) versioning: bool,
    pub(crate) governance: bool,
}

impl EventHandler {
    pub fn neo4j(&self) -> &neo4rs::Graph {
        &self.neo4j
    }

    pub fn embedding_dim(&self) -> usize {
        self.embedding_model_dim
    }

    pub fn new(neo4j: neo4rs::Graph, cache: Option<Arc<KgCache>>) -> Result<Self, HandlerError> {
        Self::new_with_ipfs(
            neo4j,
            IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            cache,
        )
    }

    pub fn new_with_ipfs(
        neo4j: neo4rs::Graph,
        ipfs: IpfsClient,
        cache: Option<Arc<KgCache>>,
    ) -> Result<Self, HandlerError> {
        Ok(Self {
            ipfs,
            neo4j,
            cache,
            spaces_blacklist: match blacklist::load() {
                Ok(Some(blacklist)) => {
                    tracing::info!(
                        "Blacklisting spaces: {}",
                        blacklist
                            .spaces
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    blacklist.spaces
                }
                Ok(None) => {
                    tracing::info!("No blacklist found");
                    vec![]
                }
                Err(e) => {
                    tracing::warn!("Error loading blacklist, skipping: {:?}", e);
                    vec![]
                }
            },
            embedding_model: TextEmbedding::try_new(
                InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
            )
            .map_err(|e| {
                tracing::error!("Error initializing embedding model: {e:?}");
                HandlerError::Other(format!("Error initializing embedding model: {e:?}").into())
            })?,
            embedding_model_dim: TextEmbedding::get_model_info(&EMBEDDING_MODEL)
                .map_err(|e| {
                    tracing::error!("Error getting embedding model info: {e:?}");
                    HandlerError::Other(format!("Error getting embedding model info: {e:?}").into())
                })?
                .dim,
            versioning: false,
            governance: false,
        })
    }

    pub fn versioning(mut self, versioning: bool) -> Self {
        self.versioning = versioning;
        self
    }

    pub fn governance(mut self, governance: bool) -> Self {
        self.governance = governance;
        self
    }
}

fn get_block_metadata(block: &BlockScopedData) -> anyhow::Result<BlockMetadata> {
    let clock = block.clock.as_ref().unwrap();
    let timestamp = DateTime::from_timestamp(
        clock.timestamp.as_ref().unwrap().seconds,
        clock.timestamp.as_ref().unwrap().nanos as u32,
    )
    .ok_or(anyhow::anyhow!("get_block_metadata: Invalid timestamp"))?;

    Ok(BlockMetadata {
        cursor: block.cursor.clone(),
        block_number: clock.number,
        timestamp,
        request_id: create_geo_id(),
    })
}

impl substreams_utils::Sink<preprocess::EventData> for EventHandler {
    type Error = HandlerError;

    async fn preprocess_block_scoped_data(
        &self,
        raw_block: &BlockScopedData,
    ) -> Result<preprocess::EventData, Self::Error> {
        let output = raw_block
            .output
            .as_ref()
            .unwrap()
            .map_output
            .as_ref()
            .unwrap();

        let block = get_block_metadata(raw_block)
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        let data = GeoOutput::decode(output.value.as_slice())?;

        let prefetched_edits = stream::iter(data.edits_published)
            .then(|edit_event| async {
                let edit = self
                    .fetch_edit(&edit_event)
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
                Result::<_, Self::Error>::Ok((edit_event, edit))
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(EventData {
            block,
            edits_published: prefetched_edits,
        })
    }

    async fn process_block_scoped_data(
        &self,
        _raw_block: &BlockScopedData,
        data: preprocess::EventData,
    ) -> Result<(), Self::Error> {
        let _timer = metrics::BLOCK_PROCESSING_TIME.start_timer();

        let drift = chrono::Utc::now().timestamp() - data.block.timestamp.timestamp();
        metrics::HEAD_BLOCK_TIME_DRIFT.set(drift as f64);
        metrics::HEAD_BLOCK_NUMBER.set(data.block.block_number as f64);
        metrics::HEAD_BLOCK_TIMESTAMP.set(data.block.timestamp.timestamp() as f64);

        // Handle edits published
        if !data.edits_published.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} edits published events",
                data.block.block_number,
                data.block.timestamp,
                data.edits_published.len()
            );
        }
        self.handle_edits_published(data.edits_published, &vec![], &data.block)
            .await?;

        // Persist block number and timestamp
        grc20_core::entity::update_one(
            &self.neo4j,
            UpdateEntity::new(indexer_ids::CURSOR_ID)
                .value(Value::new(
                    indexer_ids::BLOCK_NUMBER_ATTRIBUTE,
                    data.block.block_number.to_string(),
                ))
                .value(Value::new(
                    indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE,
                    data.block.timestamp.to_string(),
                )),
            indexer_ids::INDEXER_SPACE_ID,
        )
        .send()
        .await?;

        Ok(())
    }

    async fn load_persisted_cursor(&self) -> Result<Option<String>, Self::Error> {
        let cursor = grc20_core::value::find_one(
            &self.neo4j,
            indexer_ids::INDEXER_SPACE_ID,
            indexer_ids::CURSOR_ID,
            indexer_ids::CURSOR_ATTRIBUTE,
        )
        .send()
        .await?;

        Ok(cursor.map(|c| c.value))
    }

    async fn persist_cursor(&self, cursor: String) -> Result<(), Self::Error> {
        grc20_core::entity::update_one(
            &self.neo4j,
            UpdateEntity::new(indexer_ids::CURSOR_ID)
                .value(Value::new(indexer_ids::CURSOR_ATTRIBUTE, cursor)),
            indexer_ids::INDEXER_SPACE_ID,
        )
        .send()
        .await?;

        Ok(())
    }
}
