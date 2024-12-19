use futures::join;
use sdk::{models::{self, space::ParentSpace}, pb::geo};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_added(
        &self,
        subspace_added: &geo::SubspaceAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            self.kg
                .find_node(models::Space::find_by_space_plugin_address(&subspace_added.plugin_address)),
            self.kg
                .find_node(models::Space::find_by_dao_address_query(&subspace_added.subspace))
        ) {
            (Ok(Some(parent_space)), Ok(Some(subspace))) => {
                self.kg
                    .upsert_relation(block, &ParentSpace::new(
                        subspace.id(),
                        parent_space.id(),
                    ))
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly
            }
            (Ok(None), Ok(_)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace: parent space with plugin_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&subspace_added.plugin_address, None)
                );
            }
            (Ok(Some(_)), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace: space with dao_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&subspace_added.plugin_address, None)
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                Err(HandlerError::from(e))?;
            }
        };

        Ok(())
    }
}
