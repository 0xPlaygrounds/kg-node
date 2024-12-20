use sdk::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        Ok(models::Proposal::set_status(
            &self.kg.neo4j,
            block,
            &proposal_executed.proposal_id,
            models::proposal::ProposalStatus::Executed,
        )
        .await?)
    }
}
