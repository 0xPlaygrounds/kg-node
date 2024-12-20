use futures::try_join;
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
        match try_join!(
            Space::find_by_voting_plugin_address(
                &self.kg.neo4j,
                &editor_added.main_voting_plugin_address,
            ),
            Space::find_by_personal_plugin_address(
                &self.kg.neo4j,
                &editor_added.main_voting_plugin_address
            )
        )? {
            // Space found
            (Some(space), _) | (None, Some(space)) => {
                let editor = models::GeoAccount::new(editor_added.editor_address.clone(), block);

                // Add geo account
                editor.upsert(&self.kg.neo4j).await?;

                // Add space editor relation
                SpaceEditor::new(editor.id(), space.id(), block)
                    .upsert(&self.kg.neo4j)
                    .await?;
            }
            // Space not found
            (None, None) => {
                tracing::warn!(
                    "Block #{} ({}): Could not add editor for unknown space with voting_plugin_address = {}",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&editor_added.main_voting_plugin_address, None)
                );
            }
        }

        Ok(())
    }
}
