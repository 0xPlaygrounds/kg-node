use sdk::{
    error::DatabaseError,
    indexer_ids,
    mapping::{attributes::IntoAttributes, query_utils::Query, Entity},
    models::{
        proposal::{ProposalStatus, ProposedAccount, ProposedSubspace},
        Account, AddEditorProposal, AddMemberProposal, AddSubspaceProposal, BlockMetadata,
        EditProposal, Proposal, ProposalCreator, Proposals, RemoveEditorProposal,
        RemoveMemberProposal, RemoveSubspaceProposal, Space,
    },
    network_ids,
    pb::geo,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_add_member_proposal_created(
        &self,
        add_member_proposal: &geo::AddMemberProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &add_member_proposal.dao_address);
        let creator_id = Account::gen_id(&add_member_proposal.creator);
        let proposed_account_id = Account::gen_id(&add_member_proposal.member);

        // Create proposal
        let proposal = AddMemberProposal::new(Proposal {
            onchain_proposal_id: add_member_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&add_member_proposal.plugin_address),
            start_time: add_member_proposal.start_time.clone(),
            end_time: add_member_proposal.end_time.clone(),
        });

        self.handle_account_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_account_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_remove_member_proposal_created(
        &self,
        remove_member_proposal: &geo::RemoveMemberProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &remove_member_proposal.dao_address);
        let creator_id = Account::gen_id(&remove_member_proposal.creator);
        let proposed_account_id = Account::gen_id(&remove_member_proposal.member);

        // Create proposal
        let proposal = RemoveMemberProposal::new(Proposal {
            onchain_proposal_id: remove_member_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&remove_member_proposal.plugin_address),
            start_time: remove_member_proposal.start_time.clone(),
            end_time: remove_member_proposal.end_time.clone(),
        });

        self.handle_account_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_account_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_add_editor_proposal_created(
        &self,
        add_editor_proposal: &geo::AddEditorProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &add_editor_proposal.dao_address);
        let creator_id = Account::gen_id(&add_editor_proposal.creator);
        let proposed_account_id = Account::gen_id(&add_editor_proposal.editor);

        // Create proposal
        let proposal = AddEditorProposal::new(Proposal {
            onchain_proposal_id: add_editor_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&add_editor_proposal.plugin_address),
            start_time: add_editor_proposal.start_time.clone(),
            end_time: add_editor_proposal.end_time.clone(),
        });

        self.handle_account_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_account_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_remove_editor_proposal_created(
        &self,
        remove_editor_proposal: &geo::RemoveEditorProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &remove_editor_proposal.dao_address);
        let creator_id = Account::gen_id(&remove_editor_proposal.creator);
        let proposed_account_id = Account::gen_id(&remove_editor_proposal.editor);

        // Create proposal
        let proposal = RemoveEditorProposal::new(Proposal {
            onchain_proposal_id: remove_editor_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&remove_editor_proposal.plugin_address),
            start_time: remove_editor_proposal.start_time.clone(),
            end_time: remove_editor_proposal.end_time.clone(),
        });

        self.handle_account_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_account_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_add_subspace_proposal_created(
        &self,
        add_subspace_proposal: &geo::AddSubspaceProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &add_subspace_proposal.dao_address);
        let creator_id = Account::gen_id(&add_subspace_proposal.creator);
        let proposed_subspace_id = Space::gen_id(network_ids::GEO, &add_subspace_proposal.subspace);

        // Create proposal
        let proposal = AddSubspaceProposal::new(Proposal {
            onchain_proposal_id: add_subspace_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&add_subspace_proposal.plugin_address),
            start_time: add_subspace_proposal.start_time.clone(),
            end_time: add_subspace_proposal.end_time.clone(),
        });

        self.handle_subspace_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_subspace_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_remove_subspace_proposal_created(
        &self,
        remove_subspace_proposal: &geo::RemoveSubspaceProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &remove_subspace_proposal.dao_address);
        let creator_id = Account::gen_id(&remove_subspace_proposal.creator);
        let proposed_subspace_id =
            Space::gen_id(network_ids::GEO, &remove_subspace_proposal.subspace);

        // Create proposal
        let proposal = RemoveSubspaceProposal::new(Proposal {
            onchain_proposal_id: remove_subspace_proposal.proposal_id.clone(),
            status: ProposalStatus::Proposed,
            plugin_address: checksum_address(&remove_subspace_proposal.plugin_address),
            start_time: remove_subspace_proposal.start_time.clone(),
            end_time: remove_subspace_proposal.end_time.clone(),
        });

        self.handle_subspace_related_proposals(
            block,
            proposal,
            &space_id,
            &creator_id,
            &proposed_subspace_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_publish_edit_proposal_created(
        &self,
        publish_edit_proposal: &geo::PublishEditProposalCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = Space::gen_id(network_ids::GEO, &publish_edit_proposal.dao_address);
        let creator_id = Account::gen_id(&publish_edit_proposal.creator);

        let proposal = EditProposal::new(
            Proposal {
                onchain_proposal_id: publish_edit_proposal.proposal_id.clone(),
                status: ProposalStatus::Proposed,
                plugin_address: checksum_address(&publish_edit_proposal.plugin_address),
                start_time: publish_edit_proposal.start_time.clone(),
                end_time: publish_edit_proposal.end_time.clone(),
            },
            publish_edit_proposal.content_uri.clone(),
        );

        let proposal_id = proposal.id.clone();

        // Insert Proposal
        proposal
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        self.create_proposal_relations(block, &space_id, &proposal_id, &creator_id)
            .await?;

        Ok(())
    }

    /// Handle account-related proposals (AddMemberProposal, RemoveMemberProposal, AddEditorProposal, RemoveEditorProposal)
    ///
    /// Create the followind entities:
    /// - Proposal (proposal_id)
    /// - Account (creator_id)
    /// - Account (proposed_account_id)
    ///
    /// Create the following relations:
    /// - (space_id) > PROPOSALS > (proposal_id)
    /// - (proposal_id) > PROPOSAL_CREATOR > (creator_id)
    /// - (proposal_id) > PROPOSED_ACCOUNT > (proposed_account_id)
    async fn handle_account_related_proposals<T: IntoAttributes>(
        &self,
        block: &BlockMetadata,
        proposal: Entity<T>,
        space_id: &str,
        creator_id: &str,
        proposed_account_id: &str,
    ) -> Result<(), DatabaseError> {
        let proposal_id = proposal.id.clone();

        // Insert Proposal
        proposal
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        self.create_proposal_relations(block, space_id, &proposal_id, creator_id)
            .await?;

        self.create_proposed_account_relation(block, &proposal_id, proposed_account_id)
            .await?;

        Ok(())
    }

    /// Handle subspace-related proposals (AddSubspaceProposal, RemoveSubspaceProposal)
    ///
    /// Create the followind entities:
    /// - Proposal (proposal_id)
    /// - Account (creator_id)
    ///
    /// Create the following relations:
    /// - (space_id) > PROPOSALS > (proposal_id)
    /// - (proposal_id) > PROPOSAL_CREATOR > (creator_id)
    /// - (proposal_id) > PROPOSED_SUBSPACE > (proposed_subspace_id)
    async fn handle_subspace_related_proposals<T: IntoAttributes>(
        &self,
        block: &BlockMetadata,
        proposal: Entity<T>,
        space_id: &str,
        creator_id: &str,
        proposed_subspace_id: &str,
    ) -> Result<(), DatabaseError> {
        let proposal_id = proposal.id.clone();

        // Insert Proposal
        proposal
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        self.create_proposal_relations(block, space_id, &proposal_id, creator_id)
            .await?;

        self.create_proposed_subspace_relation(block, &proposal_id, proposed_subspace_id)
            .await?;

        Ok(())
    }

    /// Create the following relations:
    /// - (space_id) > PROPOSALS > (proposal_id)
    /// - (proposal_id) > PROPOSAL_CREATOR > (creator_id)
    async fn create_proposal_relations(
        &self,
        block: &BlockMetadata,
        space_id: &str,
        proposal_id: &str,
        creator_id: &str,
    ) -> Result<(), DatabaseError> {
        // Create Space > PROPOSALS > Proposal relation
        Proposals::new(space_id, proposal_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        // Create Proposal > PROPOSAL_CREATOR > Account relation
        ProposalCreator::new(proposal_id, creator_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        Ok(())
    }

    /// Create the following relations:
    /// - (proposal_id) > PROPOSED_ACCOUNT > (proposed_account_id)
    async fn create_proposed_account_relation(
        &self,
        block: &BlockMetadata,
        proposal_id: &str,
        proposed_account_id: &str,
    ) -> Result<(), DatabaseError> {
        ProposedAccount::new(proposal_id, proposed_account_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await
    }

    /// Create the following relations:
    /// - (proposal_id) > PROPOSED_SUBSPACE > (proposed_subspace_id)
    async fn create_proposed_subspace_relation(
        &self,
        block: &BlockMetadata,
        proposal_id: &str,
        proposed_subspace_id: &str,
    ) -> Result<(), DatabaseError> {
        ProposedSubspace::new(proposal_id, proposed_subspace_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await
    }
}
