use futures::join;
use sdk::{
    models::{self, Space, SpaceEditor},
    pb::geo,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_added(
        &self,
        editor_added: &geo::EditorAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            Space::find_by_voting_plugin_address(
                &self.kg.neo4j,
                &editor_added.main_voting_plugin_address,
            ),
            Space::find_by_personal_plugin_address(
                &self.kg.neo4j,
                &editor_added.main_voting_plugin_address
            )
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let editor = models::GeoAccount::new(editor_added.editor_address.clone());

                // Add geo account
                self.kg
                    .upsert_entity(block, &editor)
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                // Add space editor relation
                self.kg
                    .upsert_relation(block, &SpaceEditor::new(editor.id(), space.id()))
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
