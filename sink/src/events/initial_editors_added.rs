use futures::{stream, StreamExt, TryStreamExt};
use sdk::{
    models::{self, GeoAccount, Space, SpaceEditor},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_initial_space_editors_added(
        &self,
        initial_editor_added: &geo::InitialEditorAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = Space::find_by_voting_plugin_address(
            &self.kg.neo4j,
            &initial_editor_added.plugin_address,
        )
        .await?;

        if let Some(space) = &space {
            stream::iter(&initial_editor_added.addresses)
                .map(Result::<_, HandlerError>::Ok)
                .try_for_each(|editor| async move {
                    let editor = GeoAccount::new(editor.clone(), block);

                    // Add geo account
                    editor.upsert(&self.kg.neo4j).await?;

                    // Add space editor relation
                    SpaceEditor::new(editor.id(), space.id(), block)
                        .upsert(&self.kg.neo4j)
                        .await?;

                    Ok(())
                })
                .await?;

            tracing::info!(
                "Block #{} ({}): Added {} initial editors to space {}",
                block.block_number,
                block.timestamp,
                initial_editor_added.addresses.len(),
                space.id()
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add initial editors for unknown space with plugin_address = {}",
                block.block_number,
                block.timestamp,
                initial_editor_added.plugin_address
            );
        }

        Ok(())
    }
}
