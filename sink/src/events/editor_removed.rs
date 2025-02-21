use sdk::{
    models::{self, Account, Space, SpaceEditor},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_removed(
        &self,
        editor_removed: &geo::EditorRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space =
            Space::find_entity_by_dao_address(&self.neo4j, &editor_removed.dao_address).await?;

        if let Some(space) = space {
            SpaceEditor::remove(
                &self.neo4j,
                block,
                &Account::gen_id(&editor_removed.editor_address),
                &space.id,
            )
            .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not remove editor for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                editor_removed.dao_address
            );
        }

        Ok(())
    }
}
