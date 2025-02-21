use futures::{stream, StreamExt, TryStreamExt};
use ipfs::deserialize;
use sdk::{
    error::DatabaseError,
    indexer_ids,
    mapping::{self, entity_node, query_utils::Query, relation_node, triple, Entity},
    models::{
        self,
        edit::{Edits, ProposedEdit},
        Proposal, Space,
    },
    pb::{self, geo},
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

pub struct Edit {
    pub name: String,
    pub proposal_id: String,
    pub space_id: String,
    pub space_plugin_address: String,
    pub creator: String,
    pub content_uri: String,
    pub ops: Vec<pb::ipfs::Op>,
}

impl EventHandler {
    pub async fn handle_edits_published(
        &self,
        edits_published: &[geo::EditPublished],
        _created_space_ids: &[String],
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposals = stream::iter(edits_published)
            .then(|proposal| async {
                let edits = self.fetch_edit(proposal).await?;
                anyhow::Ok(edits)
            })
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))? // TODO: Convert anyhow::Error to HandlerError properly
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // let space_id = Space::new_id(network_ids::GEO, address)

        // TODO: Create "synthetic" proposals for newly created spaces and
        // personal spaces

        stream::iter(proposals)
            .enumerate()
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(|(idx, proposal)| async move {
                tracing::info!(
                    "Block #{} ({}): Processing {} ops for proposal {}",
                    block.block_number,
                    block.timestamp,
                    proposal.ops.len(),
                    proposal.proposal_id
                );

                self.process_edit(block, proposal, idx).await
            })
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        Ok(())
    }

    async fn fetch_edit(
        &self,
        edit_published: &geo::EditPublished,
    ) -> Result<Vec<Edit>, HandlerError> {
        // TODO: (optimization) Check if need to fetch entire space
        let space = if let Some(space) =
            Space::find_by_dao_address(&self.neo4j, &edit_published.dao_address)
                .await
                .map_err(|e| {
                    HandlerError::Other(
                        format!(
                            "Error querying space with plugin address {} {e:?}",
                            checksum_address(&edit_published.plugin_address)
                        )
                        .into(),
                    )
                })? {
            space
        } else {
            tracing::warn!(
                "Matching space in edit not found for plugin address {}",
                edit_published.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&edit_published.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = if let Ok(metadata) = deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
            metadata
        } else {
            tracing::warn!("Invalid metadata for edit {}", edit_published.content_uri);
            return Ok(vec![]);
        };

        match metadata.r#type() {
            pb::ipfs::ActionType::AddEdit => {
                let edit = deserialize::<pb::ipfs::Edit>(&bytes)?;
                Ok(vec![Edit {
                    name: edit.name,
                    content_uri: edit_published.content_uri.clone(),
                    proposal_id: edit.id,
                    space_id: space.id.to_string(),
                    space_plugin_address: space
                        .attributes
                        .space_plugin_address
                        .clone()
                        .expect("Space plugin address not found"),
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            pb::ipfs::ActionType::ImportSpace => {
                let import = deserialize::<pb::ipfs::Import>(&bytes)?;
                stream::iter(import.edits)
                    .map(|edit_uri| {
                        let space_id = space.id.clone();
                        let space_plugin_address = space
                            .attributes
                            .space_plugin_address
                            .clone()
                            .expect("Space plugin address not found");

                        async move {
                            let hash = edit_uri.replace("ipfs://", "");
                            let edit = self.ipfs.get::<pb::ipfs::ImportEdit>(&hash, true).await?;

                            Ok(Edit {
                                name: edit.name,
                                content_uri: edit_uri,
                                proposal_id: edit.id,
                                space_id,
                                space_plugin_address,
                                creator: edit.authors[0].clone(),
                                ops: edit.ops,
                            })
                        }
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn process_edit(
        &self,
        block: &models::BlockMetadata,
        edit: Edit,
        index: usize,
    ) -> Result<(), DatabaseError> {
        // TODO: Store edit metadata
        // 1. Check if edit exists (i.e.: was created via edit proposal)
        // 2. If exists, update edit metadata
        // 3. If not, create edit metadata

        let version_index = mapping::new_version_index(block.block_number, index);
        let edit_medatata =
            models::Edit::new(edit.name, edit.content_uri, Some(version_index.clone()));
        let proposal_id = Proposal::gen_id(&edit.space_plugin_address, &edit.proposal_id);
        self.create_edit_relations(block, edit_medatata, &edit.space_id, &proposal_id)
            .await?;

        // Group ops by type
        let op_groups = OpGroups::from_ops(edit.ops);

        // Handle SET_TRIPLE ops
        triple::insert_many(&self.neo4j, block, &edit.space_id, &version_index)
            .triples(
                op_groups
                    .set_triples
                    .into_iter()
                    .map(|triple| triple.try_into().expect("Failed to convert triple")),
            )
            .send()
            .await?;

        // Handle DELETE_TRIPLE ops
        triple::delete_many(&self.neo4j, block, &edit.space_id, &version_index)
            .triples(
                op_groups
                    .delete_triples
                    .into_iter()
                    .map(|triple| (triple.entity, triple.attribute)),
            )
            .send()
            .await?;

        // Handle CREATE_RELATION ops
        relation_node::insert_many(&self.neo4j, block, &edit.space_id, &version_index)
            .relations(
                op_groups
                    .create_relations
                    .into_iter()
                    .map(|relation| relation.into()),
            )
            .send()
            .await?;

        // Handle DELETE_RELATION ops
        relation_node::delete_many(&self.neo4j, block, &edit.space_id, &version_index)
            .relations(
                op_groups
                    .delete_relations
                    .into_iter()
                    .map(|relation| relation.id),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn create_edit_relations(
        &self,
        block: &models::BlockMetadata,
        edit: Entity<models::Edit>,
        space_id: &str,
        proposal_id: &str,
    ) -> Result<(), DatabaseError> {
        let edit_id = edit.id.clone();

        // Insert edit
        edit.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        // Create relation between proposal and edit
        if let Some(proposal) = entity_node::find_one(&self.neo4j, proposal_id)
            .send()
            .await?
        {
            ProposedEdit::new(proposal.id, &edit_id)
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Proposal {proposal_id} not found for edit {edit_id}",
                block.block_number,
                block.timestamp,
            );
        }

        // Create relation between space and edit
        Edits::new(space_id, edit_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        Ok(())
    }
}

// Ops are grouped by type
#[derive(Debug, Default)]
pub struct OpGroups {
    set_triples: Vec<pb::ipfs::Triple>,
    delete_triples: Vec<pb::ipfs::Triple>,
    create_relations: Vec<pb::ipfs::Relation>,
    delete_relations: Vec<pb::ipfs::Relation>,
}

impl OpGroups {
    pub fn from_ops(ops: impl IntoIterator<Item = pb::ipfs::Op>) -> Self {
        let mut op_groups = Self::default();

        for op in ops {
            match (op.r#type(), op) {
                (
                    pb::ipfs::OpType::SetTriple,
                    pb::ipfs::Op {
                        triple: Some(triple),
                        ..
                    },
                ) => {
                    op_groups.set_triples.push(triple);
                }
                (
                    pb::ipfs::OpType::DeleteTriple,
                    pb::ipfs::Op {
                        triple: Some(triple),
                        ..
                    },
                ) => {
                    op_groups.delete_triples.push(triple);
                }

                (
                    pb::ipfs::OpType::CreateRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    op_groups.create_relations.push(relation);
                }
                (
                    pb::ipfs::OpType::DeleteRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    op_groups.delete_relations.push(relation);
                }

                (typ, maybe_triple) => {
                    tracing::warn!("Unhandled case: {:?} {:?}", typ, maybe_triple);
                }
            }
        }

        op_groups
    }
}
