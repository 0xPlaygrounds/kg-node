use sdk::{
    models::{self, GeoAccount, Space, SpaceEditor},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_removed(
        &self,
        editor_removed: &geo::EditorRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = Space::find_by_dao_address(&self.kg.neo4j, &editor_removed.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        if let Some(space) = space {
            SpaceEditor::remove(
                &self.kg.neo4j,
                &GeoAccount::new_id(&editor_removed.editor_address),
                space.id(),
            )
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
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
