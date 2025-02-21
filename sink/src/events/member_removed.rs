use sdk::{
    models::{self, Account, Space, SpaceMember},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_removed(
        &self,
        member_removed: &geo::MemberRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space =
            Space::find_entity_by_dao_address(&self.neo4j, &member_removed.dao_address).await?;

        if let Some(space) = space {
            SpaceMember::remove(
                &self.neo4j,
                block,
                &Account::gen_id(&member_removed.member_address),
                &space.id,
            )
            .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not remove member for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                member_removed.dao_address
            );
        }

        Ok(())
    }
}
