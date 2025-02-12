use futures::{stream, StreamExt, TryStreamExt};
use ipfs::deserialize;
use sdk::{
    error::DatabaseError,
    mapping::{Entity, Relation},
    models::{self, EditProposal, Space},
    pb::{self, geo},
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

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
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(|proposal| async move {
                tracing::info!(
                    "Block #{} ({}): Processing ops for proposal {}",
                    block.block_number,
                    block.timestamp,
                    proposal.proposal_id
                );

                self.process_ops(block, &proposal.space, proposal.ops).await
            })
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        Ok(())
    }

    async fn fetch_edit(
        &self,
        edit: &geo::EditPublished,
    ) -> Result<Vec<EditProposal>, HandlerError> {
        let space = if let Some(space) =
            Space::find_by_space_plugin_address(&self.neo4j, &edit.plugin_address)
                .await
                .map_err(|e| {
                    HandlerError::Other(
                        format!(
                            "Error querying space with plugin address {} {e:?}",
                            checksum_address(&edit.plugin_address)
                        )
                        .into(),
                    )
                })? {
            space
        } else {
            tracing::warn!(
                "Matching space in edit not found for plugin address {}",
                edit.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&edit.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = if let Ok(metadata) = deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
            metadata
        } else {
            tracing::warn!("Invalid metadata for edit {}", edit.content_uri);
            return Ok(vec![]);
        };

        match metadata.r#type() {
            pb::ipfs::ActionType::AddEdit => {
                let edit = deserialize::<pb::ipfs::Edit>(&bytes)?;
                Ok(vec![EditProposal {
                    name: edit.name,
                    proposal_id: edit.id,
                    space: space.id().to_string(),
                    space_address: space
                        .attributes()
                        .space_plugin_address
                        .clone()
                        .expect("Space plugin address not found"),
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            pb::ipfs::ActionType::ImportSpace => {
                let import = deserialize::<pb::ipfs::Import>(&bytes)?;
                let edits = stream::iter(import.edits)
                    .map(|edit| async move {
                        let hash = edit.replace("ipfs://", "");
                        self.ipfs.get::<pb::ipfs::ImportEdit>(&hash, true).await
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await?;

                Ok(edits
                    .into_iter()
                    .map(|edit| EditProposal {
                        name: edit.name,
                        proposal_id: edit.id,
                        space: space.id().to_string(),
                        space_address: space
                            .attributes()
                            .space_plugin_address
                            .clone()
                            .expect("Space plugin address not found"),
                        creator: edit.authors[0].clone(),
                        ops: edit.ops,
                    })
                    .collect())
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn process_ops(
        &self,
        block: &models::BlockMetadata,
        space_id: &str,
        ops: impl IntoIterator<Item = pb::ipfs::Op>,
    ) -> Result<(), DatabaseError> {
        for op in ops {
            match (op.r#type(), op) {
                (
                    pb::ipfs::OpType::SetTriple,
                    pb::ipfs::Op {
                        triple:
                            Some(pb::ipfs::Triple {
                                entity,
                                attribute,
                                value: Some(value),
                            }),
                        ..
                    },
                ) => {
                    tracing::info!("SetTriple: {}, {}, {:?}", entity, attribute, value,);

                    Entity::<()>::set_triple(
                        &self.neo4j,
                        block,
                        space_id,
                        &entity,
                        &attribute,
                        &value,
                    )
                    .await?
                }
                (
                    pb::ipfs::OpType::DeleteTriple,
                    pb::ipfs::Op {
                        triple: Some(triple),
                        ..
                    },
                ) => {
                    tracing::info!(
                        "DeleteTriple: {}, {}, {:?}",
                        triple.entity,
                        triple.attribute,
                        triple.value,
                    );

                    Entity::<()>::delete_triple(&self.neo4j, block, space_id, triple).await?
                }
                // TODO: Handle these cases
                // (pb::ipfs::OpType::SetTripleBatch, op) => {
                // }
                // (pb::ipfs::OpType::DeleteEntity, op) => {
                // }
                (
                    pb::ipfs::OpType::CreateRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    tracing::info!(
                        "CreateRelation: {}, {}, {}, {}",
                        relation.id,
                        relation.r#type,
                        relation.from_entity,
                        relation.to_entity,
                    );

                    Relation::<()>::new(
                        &relation.id,
                        space_id,
                        &relation.r#type,
                        &relation.from_entity,
                        &relation.to_entity,
                        block,
                        (),
                    )
                    .upsert(&self.neo4j)
                    .await?
                }
                (
                    pb::ipfs::OpType::DeleteRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    tracing::info!("DeleteRelation: {}", relation.id);
                    Entity::<()>::delete(&self.neo4j, block, &relation.id, space_id).await?
                }
                (typ, maybe_triple) => {
                    tracing::warn!("Unhandled case: {:?} {:?}", typ, maybe_triple);
                }
            }
        }

        Ok(())
    }
}
