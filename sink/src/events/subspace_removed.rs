use sdk::{
    models::{self, space::ParentSpace},
    network_ids,
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_removed(
        &self,
        subspace_removed: &geo::SubspaceRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = models::Space::find_entity_by_space_plugin_address(
            &self.neo4j,
            &subspace_removed.plugin_address,
        )
        .await?;

        let subspace_id = models::Space::gen_id(network_ids::GEO, &subspace_removed.subspace);

        if let Some(space) = space {
            ParentSpace::remove(&self.neo4j, block, &subspace_id, &space.id).await?;

            tracing::info!(
                "Block #{} ({}): Removed subspace {} from space {}",
                block.block_number,
                block.timestamp,
                subspace_removed.subspace,
                space.id
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not remove subspace for unknown space with plugin address = {}",
                block.block_number,
                block.timestamp,
                subspace_removed.plugin_address
            );
        }

        Ok(())
    }
}
