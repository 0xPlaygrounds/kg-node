use sdk::{models, pb::geo, system_ids};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_removed(
        &self,
        subspace_removed: &geo::SubspaceRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .find_node(models::Space::find_by_space_plugin_address(&subspace_removed.plugin_address))
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = space {
            self.kg.neo4j
                .run(neo4rs::query(&format!(
                    "MATCH (subspace:`{INDEXED_SPACE}` {{parent_space: $space_id}}) DELETE subspace",
                    INDEXED_SPACE = system_ids::INDEXED_SPACE,
                )).param("space_id", space.id()))
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            tracing::info!(
                "Block #{} ({}): Removed subspace {} from space {}",
                block.block_number,
                block.timestamp,
                subspace_removed.subspace,
                space.id()
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
