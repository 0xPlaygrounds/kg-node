use sdk::{
    models::{self, GeoAccount, Space, SpaceType},
    network_ids,
    pb::geo,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    /// Handles `GeoSpaceCreated` events.
    pub async fn handle_space_created(
        &self,
        space_created: &geo::GeoSpaceCreated,
        // edits_published: &[geo::EditPublished],
        block: &models::BlockMetadata,
    ) -> Result<String, HandlerError> {
        // Match the space creation events with their corresponding initial proposal (if any)
        // let initial_proposals = spaces_created
        //     .iter()
        //     .filter_map(|event| {
        //         edits_published
        //             .iter()
        //             .find(|proposal| {
        //                 checksum_address(&proposal.plugin_address, None)
        //                     == checksum_address(&event.space_address, None)
        //             })
        //             .map(|proposal| (event.space_address.clone(), proposal))
        //     })
        //     .collect::<HashMap<_, _>>();

        // tracing::info!()

        // For spaces with an initial proposal, get the space ID from the import (if available)
        // let space_ids = stream::iter(initial_proposals)
        //     .filter_map(|(space_address, proposal_processed)| async move {
        //         let ipfs_hash = proposal_processed.content_uri.replace("ipfs://", "");
        //         self.ipfs
        //             .get::<grc20::Import>(&ipfs_hash, true)
        //             .await
        //             .ok()
        //             .map(|import| {
        //                 (
        //                     space_address,
        //                     Space::new_id(
        //                         &import.previous_network,
        //                         &import.previous_contract_address,
        //                     ),
        //                 )
        //             })
        //     })
        //     .collect::<HashMap<_, _>>()
        //     .await;
        let space_id = Space::new_id(network_ids::GEO, &space_created.dao_address);

        tracing::info!(
            "Block #{} ({}): Creating space {}",
            block.block_number,
            block.timestamp,
            space_id
        );

        self.kg
            .upsert_entity(
                block,
                &Space::builder(&space_id, &space_created.dao_address)
                    .network(network_ids::GEO.to_string())
                    .space_plugin_address(&space_created.space_address)
                    .build(),
            )
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
        let space = Space::find_by_dao_address(&self.kg.neo4j, &personal_space_created.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = &space {
            self.kg
                .upsert_entity(
                    block,
                    &Space::builder(space.id(), &space.attributes().dao_contract_address)
                        .r#type(SpaceType::Personal)
                        .personal_space_admin_plugin(&personal_space_created.personal_admin_address)
                        .build(),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            // Add initial editors to the personal space
            let editor = GeoAccount::new(personal_space_created.initial_editor.clone());

            self.kg
                .upsert_entity(block, &editor)
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            tracing::info!(
                "Block #{} ({}): Creating personal admin space plugin for space {} with initial editor {}",
                block.block_number,
                block.timestamp,
                space.id(),
                editor.id(),
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create personal admin space plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&personal_space_created.dao_address, None)
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
            Space::find_by_dao_address(&self.kg.neo4j, &governance_plugin_created.dao_address)
                .await
                .map_err(|e| {
                    HandlerError::Other(
                        format!(
                            "Error fetching space with dao address = {}: {e:?}",
                            checksum_address(&governance_plugin_created.dao_address, None)
                        )
                        .into(),
                    )
                })?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = space {
            tracing::info!(
                "Block #{} ({}): Creating governance plugin for space {}",
                block.block_number,
                block.timestamp,
                space.id()
            );

            self.kg
                .upsert_entity(
                    block,
                    &Space::builder(space.id(), &space.attributes().dao_contract_address)
                        .voting_plugin_address(&governance_plugin_created.main_voting_address)
                        .member_access_plugin(&governance_plugin_created.member_access_address)
                        .build(),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("Error updating space: {e:?}").into()))?;
        // TODO: Convert anyhow::Error to HandlerError properly
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create governance plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&governance_plugin_created.dao_address, None)
            );
        }

        Ok(())
    }
}
