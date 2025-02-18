use sdk::{
    indexer_ids,
    mapping::{entity_node, Query, Triple},
    models::{self, proposal::ProposalStatus, Proposal},
    pb::geo,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &models::BlockMetadata,
        _index: usize,
    ) -> Result<(), HandlerError> {
        let plugin_address = checksum_address(&proposal_executed.plugin_address);
        let proposal_id = Proposal::gen_id(
            &proposal_executed.plugin_address,
            &proposal_executed.proposal_id,
        );

        let maybe_proposal = entity_node::find_one(&self.neo4j, &proposal_id)
            .send()
            .await?;

        // Find space
        // let maybe_space_id = triple::find_many(&self.neo4j)
        //     .attribute_id(prop_filter::value_in(vec![
        //         indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS.into(),
        //         indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS.into(),
        //         indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS.into(),
        //     ]))
        //     .value(prop_filter::value(&plugin_address))
        //     .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
        //     .send()
        //     .await?
        //     .into_iter()
        //     .next();

        if let Some(proposal) = maybe_proposal {
            // Update proposal status
            Triple::new(
                &proposal.id,
                "status", // TODO: Change to GRC20 id
                ProposalStatus::Executed,
            )
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Proposal {proposal_id} not found for space plugin address {plugin_address}",
                block.block_number,
                block.timestamp,
            );
        }

        Ok(())
    }
}
