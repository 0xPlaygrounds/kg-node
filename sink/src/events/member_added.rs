use futures::try_join;
use sdk::{
    models::{BlockMetadata, GeoAccount, Space, SpaceMember},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_added(
        &self,
        member_added: &geo::MemberAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        match try_join!(
            Space::find_by_voting_plugin_address(
                &self.kg.neo4j,
                &member_added.main_voting_plugin_address
            ),
            Space::find_by_personal_plugin_address(
                &self.kg.neo4j,
                &member_added.main_voting_plugin_address
            )
        )? {
            // Space found
            (Some(space), _) | (None, Some(space)) => {
                let member = GeoAccount::new(member_added.member_address.clone(), block);

                // Add geo account
                member.upsert(&self.kg.neo4j).await?;

                // Add space member relation
                SpaceMember::new(member.id(), space.id(), block)
                    .upsert(&self.kg.neo4j)
                    .await?;
            }
            // Space not found
            (None, None) => {
                tracing::warn!(
                    "Block #{} ({}): Could not add members for unknown space with voting_plugin_address = {}",
                    block.block_number,
                    block.timestamp,
                    member_added.main_voting_plugin_address
                );
            }
        };

        Ok(())
    }
}
