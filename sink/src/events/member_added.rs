use futures::join;
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
        match join!(
            Space::find_by_voting_plugin_address(
                &self.kg.neo4j,
                &member_added.main_voting_plugin_address
            ),
            Space::find_by_personal_plugin_address(
                &self.kg.neo4j,
                &member_added.main_voting_plugin_address
            )
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let member = GeoAccount::new(member_added.member_address.clone());

                // Add geo account
                self.kg
                    .upsert_entity(block, &member)
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                // Add space member relation
                self.kg
                    .upsert_relation(block, &SpaceMember::new(member.id(), space.id()))
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
            }
            // Space not found
            (Ok(None), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not add members for unknown space with voting_plugin_address = {}",
                    block.block_number,
                    block.timestamp,
                    member_added.main_voting_plugin_address
                );
            }
            // Errors
            (Err(e), _) | (_, Err(e)) => {
                return Err(HandlerError::Other(format!("{e:?}").into()));
            }
        };

        Ok(())
    }
}
