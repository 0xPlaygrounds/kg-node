use sdk::{models, network_ids, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_add_member_proposal_created(
        &self,
        add_member_proposal: &geo::AddMemberProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::AddMemberProposal::new(models::Proposal {
                    onchain_proposal_id: add_member_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: add_member_proposal.plugin_address.clone(),
                    start_time: add_member_proposal.start_time.clone(),
                    end_time: add_member_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &add_member_proposal.dao_address),
                    &models::Proposal::new_id(&add_member_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&add_member_proposal.proposal_id),
                    &add_member_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_remove_member_proposal_created(
        &self,
        remove_member_proposal: &geo::RemoveMemberProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::RemoveMemberProposal::new(models::Proposal {
                    onchain_proposal_id: remove_member_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: remove_member_proposal.plugin_address.clone(),
                    start_time: remove_member_proposal.start_time.clone(),
                    end_time: remove_member_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &remove_member_proposal.dao_address),
                    &models::Proposal::new_id(&remove_member_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&remove_member_proposal.proposal_id),
                    &remove_member_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_add_editor_proposal_created(
        &self,
        add_editor_proposal: &geo::AddEditorProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::AddEditorProposal::new(models::Proposal {
                    onchain_proposal_id: add_editor_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: add_editor_proposal.plugin_address.clone(),
                    start_time: add_editor_proposal.start_time.clone(),
                    end_time: add_editor_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &add_editor_proposal.dao_address),
                    &models::Proposal::new_id(&add_editor_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&add_editor_proposal.proposal_id),
                    &add_editor_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_remove_editor_proposal_created(
        &self,
        remove_editor_proposal: &geo::RemoveEditorProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::RemoveEditorProposal::new(models::Proposal {
                    onchain_proposal_id: remove_editor_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: remove_editor_proposal.plugin_address.clone(),
                    start_time: remove_editor_proposal.start_time.clone(),
                    end_time: remove_editor_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &remove_editor_proposal.dao_address),
                    &models::Proposal::new_id(&remove_editor_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&remove_editor_proposal.proposal_id),
                    &remove_editor_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_add_subspace_proposal_created(
        &self,
        add_subspace_proposal: &geo::AddSubspaceProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::AddSubspaceProposal::new(models::Proposal {
                    onchain_proposal_id: add_subspace_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: add_subspace_proposal.plugin_address.clone(),
                    start_time: add_subspace_proposal.start_time.clone(),
                    end_time: add_subspace_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &add_subspace_proposal.dao_address),
                    &models::Proposal::new_id(&add_subspace_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&add_subspace_proposal.proposal_id),
                    &add_subspace_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_remove_subspace_proposal_created(
        &self,
        remove_subspace_proposal: &geo::RemoveSubspaceProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // Create proposal
        self.kg
            .upsert_entity(
                block,
                &models::RemoveSubspaceProposal::new(models::Proposal {
                    onchain_proposal_id: remove_subspace_proposal.proposal_id.clone(),
                    status: sdk::models::proposal::ProposalStatus::Proposed,
                    plugin_address: remove_subspace_proposal.plugin_address.clone(),
                    start_time: remove_subspace_proposal.start_time.clone(),
                    end_time: remove_subspace_proposal.end_time.clone(),
                }),
            )
            .await?;

        // Create Space > PROPOSALS > Proposal relation
        self.kg
            .upsert_relation(
                block,
                &models::Proposals::new(
                    &models::Space::new_id(network_ids::GEO, &remove_subspace_proposal.dao_address),
                    &models::Proposal::new_id(&remove_subspace_proposal.proposal_id),
                ),
            )
            .await?;

        // Create Proposal > CREATOR > Account relation
        self.kg
            .upsert_relation(
                block,
                &models::Creator::new(
                    &models::Proposal::new_id(&remove_subspace_proposal.proposal_id),
                    &remove_subspace_proposal.creator,
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn handle_publish_edit_proposal_created(
        &self,
        _publish_edit_proposal: &geo::PublishEditProposalCreated,
        _block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        todo!()
    }
}
