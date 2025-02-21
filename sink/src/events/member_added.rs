use sdk::{
    indexer_ids,
    mapping::query_utils::Query,
    models::{Account, BlockMetadata, Space, SpaceMember},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_added(
        &self,
        member_added: &geo::MemberAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        // match try_join!(
        //     Space::find_by_voting_plugin_address(
        //         &self.neo4j,
        //         &member_added.main_voting_plugin_address
        //     ),
        //     Space::find_by_personal_plugin_address(
        //         &self.neo4j,
        //         &member_added.main_voting_plugin_address
        //     )
        // )? {
        //     // Space found
        //     (Some(space), _) | (None, Some(space)) => {
        //         let member = Account::new(member_added.member_address.clone());
        //         let member_rel = SpaceMember::new(&member.id, &space.id);

        //         // Add geo account
        //         member.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;

        //         // Add space member relation
        //         member_rel
        //             .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;
        //     }
        //     // Space not found
        //     (None, None) => {
        //         tracing::warn!(
        //             "Block #{} ({}): Could not add members for unknown space with voting_plugin_address = {}",
        //             block.block_number,
        //             block.timestamp,
        //             member_added.main_voting_plugin_address
        //         );
        //     }
        // };

        if let Some(space) =
            Space::find_entity_by_dao_address(&self.neo4j, &member_added.dao_address).await?
        {
            let member = Account::new(member_added.member_address.clone());
            let member_rel = SpaceMember::new(&member.id, &space.id);

            // Add geo account
            member
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;

            // Add space member relation
            member_rel
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add members for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                member_added.dao_address
            );
        }

        Ok(())
    }
}
