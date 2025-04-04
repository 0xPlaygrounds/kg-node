use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, pb::geo};
use grc20_sdk::models::{account, space, SpaceEditor};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_initial_space_editors_added(
        &self,
        initial_editor_added: &geo::InitialEditorAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space =
            space::find_entity_by_dao_address(&self.neo4j, &initial_editor_added.dao_address)
                .await?;

        if let Some(space) = &space {
            stream::iter(&initial_editor_added.addresses)
                .map(Result::<_, HandlerError>::Ok)
                .try_for_each(|editor_address| async move {
                    // Create editor account and relation
                    let editor = account::new(editor_address.clone());
                    let editor_rel = SpaceEditor::new(editor.id(), &space.id);

                    // Insert editor account
                    editor
                        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                        .send()
                        .await?;

                    // Insert space editor relation
                    editor_rel
                        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                        .send()
                        .await?;

                    Ok(())
                })
                .await?;

            tracing::info!(
                "Block #{} ({}): Added {} initial editors to space {}",
                block.block_number,
                block.timestamp,
                initial_editor_added.addresses.len(),
                space.id,
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
