use futures::join;
use sdk::{
    ids, mapping::{Entity, Relation}, models::{self, Space}, pb::geo, system_ids::{self, INDEXER_SPACE_ID}
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
            self.kg
                .find_node(Space::find_by_voting_plugin_address(&vote.plugin_address)),
            self.kg
                .find_node(Space::find_by_member_access_plugin(&vote.plugin_address))
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let maybe_proposal = self.kg
                    .find_node(models::Proposal::find_by_id_and_address(&vote.onchain_proposal_id, &vote.plugin_address))
                    .await?;

                let account = self
                    .kg
                    .find_node(
                        Entity::<models::GeoAccount>::find_by_id_query(
                            &models::GeoAccount::new_id(&vote.voter),
                        ))
                        .await?;

                match (maybe_proposal, account) {
                    (Some(proposal), Some(account)) => {
                        let vote_cast = models::VoteCast {
                            vote_type: vote
                                .vote_option
                                .try_into()
                                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                        };

                        self.kg
                            .upsert_relation(
                                block,
                                &Relation::new(
                                    &ids::create_geo_id(),
                                    INDEXER_SPACE_ID,
                                    account.id(),
                                    proposal.id(),
                                    vote_cast,
                                ),
                            )
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
