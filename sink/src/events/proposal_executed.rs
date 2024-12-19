use sdk::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        Ok(self.kg.run(
            models::Proposal::set_status_query(
                block,
                &proposal_executed.proposal_id, 
                models::proposal::ProposalStatus::Executed,
            )
        )
        .await?)
    }
}
