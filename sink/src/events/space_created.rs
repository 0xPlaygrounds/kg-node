use sdk::{
    indexer_ids, mapping::query_utils::Query, models::{self, Account, Space, SpaceGovernanceType}, network_ids, pb::{self, geo}
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    /// Handles `GeoSpaceCreated` events.
    pub async fn handle_space_created(
        &self,
        space_created: &geo::GeoSpaceCreated,
        edits_published: &[geo::EditPublished],
        block: &models::BlockMetadata,
    ) -> Result<String, HandlerError> {
        let maybe_initial_proposal = edits_published.iter().find(|proposal| {
            checksum_address(&proposal.plugin_address)
                == checksum_address(&space_created.space_address)
        });

        let maybe_existing_space_id = match maybe_initial_proposal {
            Some(initial_proposal) => {
                let bytes = self
                    .ipfs
                    .get_bytes(&initial_proposal.content_uri.replace("ipfs://", ""), true)
                    .await?;

                if let Ok(metadata) = ipfs::deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
                    match metadata.r#type() {
                        pb::ipfs::ActionType::ImportSpace => {
                            let import = ipfs::deserialize::<pb::ipfs::Import>(&bytes)?;

                            tracing::info!(
                                "Block #{} ({}): Found import for space {} (derived id: {})",
                                block.block_number,
                                block.timestamp,
                                checksum_address(&space_created.space_address),
                                Space::generate_id(
                                    &import.previous_network,
                                    &import.previous_contract_address,
                                )
                            );

                            Some(Space::generate_id(
                                &import.previous_network,
                                &import.previous_contract_address,
                            ))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        let space_id = maybe_existing_space_id
            .unwrap_or_else(|| Space::generate_id(network_ids::GEO, &space_created.dao_address));

        tracing::info!(
            "Block #{} ({}): Creating space {}",
            block.block_number,
            block.timestamp,
            space_id
        );

        Space::builder(&space_id, &space_created.dao_address)
            .network(network_ids::GEO.to_string())
            .space_plugin_address(&space_created.space_address)
            .build()
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
            .send()
            .await?;

        // Create the spaces
        // let created_ids: Vec<_> = stream::iter(spaces_created)
        //     .then(|event| async {
        //         let space_id = space_ids
        //             .get(&event.space_address)
        //             .cloned()
        //             .unwrap_or(Space::new_id(network_ids::GEO, &event.dao_address));

        //         anyhow::Ok(space_id)
        //     })
        //     .try_collect()
        //     .await
        //     .map_err(|err| HandlerError::Other(format!("{err:?}").into()))?;

        Ok(space_id)
    }

    pub async fn handle_personal_space_created(
        &self,
        personal_space_created: &geo::GeoPersonalSpaceAdminPluginCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = Space::find_by_dao_address(&self.neo4j, &personal_space_created.dao_address)
            .await?;

        if let Some(space) = &space {
            Space::builder(&space.id, &space.attributes.dao_contract_address)
                .governance_type(SpaceGovernanceType::Personal)
                .personal_space_admin_plugin(&personal_space_created.personal_admin_address)
                .build()
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
                .send()
                .await?;

            // Add initial editors to the personal space
            let editor = Account::new(personal_space_created.initial_editor.clone());

            editor.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
                .send()
                .await?;

            tracing::info!(
                "Block #{} ({}): Created personal admin space plugin for space {} with initial editor {}",
                block.block_number,
                block.timestamp,
                space.id,
                Account::generate_id(&personal_space_created.initial_editor),
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create personal admin space plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&personal_space_created.dao_address)
            );
        }

        Ok(())
    }

    pub async fn handle_governance_plugin_created(
        &self,
        governance_plugin_created: &geo::GeoGovernancePluginCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space =
            Space::find_by_dao_address(&self.neo4j, &governance_plugin_created.dao_address).await?;

        if let Some(space) = space {
            tracing::info!(
                "Block #{} ({}): Creating governance plugin for space {}",
                block.block_number,
                block.timestamp,
                space.id
            );

            Space::builder(&space.id, &space.attributes.dao_contract_address)
                .voting_plugin_address(&governance_plugin_created.main_voting_address)
                .member_access_plugin(&governance_plugin_created.member_access_address)
                .build()
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
                .send()
                .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create governance plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&governance_plugin_created.dao_address)
            );
        }

        Ok(())
    }
}
