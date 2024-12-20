use futures::join;
use sdk::{
    mapping::Entity,
    models::{self, Space, VoteCast},
    pb::geo,
    system_ids,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_vote_cast(
        &self,
        vote: &geo::VoteCast,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            Space::find_by_voting_plugin_address(&self.kg.neo4j, &vote.plugin_address),
            Space::find_by_member_access_plugin(&self.kg.neo4j, &vote.plugin_address)
        ) {
            // Space found
            (Ok(Some(_space)), Ok(_)) | (Ok(None), Ok(Some(_space))) => {
                let maybe_proposal = models::Proposal::find_by_id_and_address(
                    &self.kg.neo4j,
                    &vote.onchain_proposal_id,
                    &vote.plugin_address,
                )
                .await?;

                let account = Entity::<models::GeoAccount>::find_by_id(
                    &self.kg.neo4j,
                    &models::GeoAccount::new_id(&vote.voter),
                    system_ids::INDEXER_SPACE_ID,
                )
                .await?;

                match (maybe_proposal, account) {
                    (Some(proposal), Some(account)) => {
                        VoteCast::new(
                            account.id(),
                            proposal.id(),
                            vote.vote_option
                                .try_into()
                                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                            block,
                        )
                        .upsert(&self.kg.neo4j)
                        .await?;
                    }
                    // Proposal or account not found
                    (Some(_), None) => {
                        tracing::warn!(
                            "Block #{} ({}): Matching account not found for vote cast",
                            block.block_number,
                            block.timestamp,
                        );
                    }
                    (None, _) => {
                        tracing::warn!(
                            "Block #{} ({}): Matching proposal not found for vote cast",
                            block.block_number,
                            block.timestamp,
                        );
                    }
                }
            }
            // Space not found
            (Ok(None), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Matching space in Proposal not found for plugin address = {}",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&vote.plugin_address, None),
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
