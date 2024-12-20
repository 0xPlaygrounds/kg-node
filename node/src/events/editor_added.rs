use futures::join;
use kg_core::{models, pb::geo};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_added(
        &self,
        editor_added: &geo::EditorAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            self.kg
                .get_space_by_voting_plugin_address(&editor_added.main_voting_plugin_address),
            self.kg
                .get_space_by_personal_plugin_address(&editor_added.main_voting_plugin_address)
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let editor = models::GeoAccount::new(editor_added.editor_address.clone());

                self.kg
                    .add_editor(&space.id, &editor, &models::SpaceEditor, block)
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
            }
            // Space not found
            (Ok(None), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not add editor for unknown space with voting_plugin_address = {}",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&editor_added.main_voting_plugin_address, None)
                );
            }
            // Errors
            (Err(e), _) | (_, Err(e)) => {
                return Err(HandlerError::Other(format!("{e:?}").into()));
            }
        }

        Ok(())
    }
}
