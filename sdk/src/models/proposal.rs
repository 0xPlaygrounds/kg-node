use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ids, mapping::{Entity, Query, Relation}, pb::{self, grc20}, system_ids};

use super::BlockMetadata;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalType {
    AddEdit,
    ImportSpace,
    AddSubspace,
    RemoveSubspace,
    AddEditor,
    RemoveEditor,
    AddMember,
    RemoveMember,
}

impl TryFrom<pb::ipfs::ActionType> for ProposalType {
    type Error = String;

    fn try_from(action_type: pb::ipfs::ActionType) -> Result<Self, Self::Error> {
        match action_type {
            pb::ipfs::ActionType::AddMember => Ok(Self::AddMember),
            pb::ipfs::ActionType::RemoveMember => Ok(Self::RemoveMember),
            pb::ipfs::ActionType::AddEditor => Ok(Self::AddEditor),
            pb::ipfs::ActionType::RemoveEditor => Ok(Self::RemoveEditor),
            pb::ipfs::ActionType::AddSubspace => Ok(Self::AddSubspace),
            pb::ipfs::ActionType::RemoveSubspace => Ok(Self::RemoveSubspace),
            pb::ipfs::ActionType::AddEdit => Ok(Self::AddEdit),
            pb::ipfs::ActionType::ImportSpace => Ok(Self::ImportSpace),
            _ => Err(format!("Invalid action type: {:?}", action_type)),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus {
    Proposed,
    Accepted,
    Rejected,
    Canceled,
    Executed,
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Proposed => write!(f, "PROPOSED"),
            ProposalStatus::Accepted => write!(f, "ACCEPTED"),
            ProposalStatus::Rejected => write!(f, "REJECTED"),
            ProposalStatus::Canceled => write!(f, "CANCELED"),
            ProposalStatus::Executed => write!(f, "EXECUTED"),
        }
    }
}

/// Common fields for all proposals
#[derive(Clone, Deserialize, Serialize)]
pub struct Proposal {
    pub onchain_proposal_id: String,
    pub status: ProposalStatus,
    pub plugin_address: String,
    pub start_time: String,
    pub end_time: String,
}

impl Proposal {
    pub fn new_id(proposal_id: &str) -> String {
        ids::create_id_from_unique_string(proposal_id)
    }

    pub fn find_by_id_and_address(
        proposal_id: &str,
        plugin_address: &str,
    ) -> Query<Self> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (n:`{PROPOSAL_TYPE}` {{onchain_proposal_id: $proposal_id, plugin_address: $plugin_address}})
            RETURN n
            "#,
            PROPOSAL_TYPE = system_ids::PROPOSAL_TYPE,
        );

        Query::new(QUERY)
            .param("proposal_id", proposal_id)
            .param("plugin_address", plugin_address)
    }

    /// Returns a query to set the status of a proposal given its ID
    pub fn set_status_query(block: &BlockMetadata, proposal_id: &str, status: ProposalStatus) -> Query<()> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (n:`{PROPOSAL_TYPE}` {{onchain_proposal_id: $proposal_id}})
            SET n.status = $status
            SET n += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            "#,
            PROPOSAL_TYPE = system_ids::PROPOSAL_TYPE,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        Query::new(QUERY)
            .param("proposal_id", proposal_id)
            .param("status", status.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string())
    }
}

// Relation for Space > PROPOSALS > Proposal
#[derive(Deserialize, Serialize)]
pub struct Proposals;

impl Proposals {
    pub fn new(
        space_id: &str,
        proposal_id: &str,
    ) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("{space_id}-{proposal_id}")),
            system_ids::INDEXER_SPACE_ID,
            space_id,
            proposal_id,
            Proposals {}
        )
        .with_type(system_ids::PROPOSALS)
    }
}

// Proposal > CREATOR > Account
#[derive(Deserialize, Serialize)]
pub struct Creator;

impl Creator {
    pub fn new(
        proposal_id: &str,
        account_id: &str,
    ) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("{proposal_id}-{account_id}")),
            system_ids::INDEXER_SPACE_ID,
            proposal_id,
            account_id,
            Creator {}
        )
        .with_type(system_ids::PROPOSAL_CREATOR)
    }
}

pub struct EditProposal {
    pub name: String,
    pub proposal_id: String,
    pub space: String,
    pub space_address: String,
    pub creator: String,
    pub ops: Vec<grc20::Op>,
}

#[derive(Deserialize, Serialize)]
pub struct AddMemberProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl AddMemberProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::ADD_MEMBER_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RemoveMemberProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl RemoveMemberProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::REMOVE_MEMBER_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct AddEditorProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl AddEditorProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::ADD_EDITOR_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RemoveEditorProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl RemoveEditorProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::REMOVE_EDITOR_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProposedAccount;

impl ProposedAccount {
    pub fn new(
        proposal_id: &str,
        account_id: &str,
    ) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("{}-{}", proposal_id, account_id)),
            system_ids::INDEXER_SPACE_ID,
            proposal_id,
            account_id,
            Self {},
        )
        .with_type(system_ids::PROPOSED_ACCOUNT)
    }
}

#[derive(Deserialize, Serialize)]
pub struct AddSubspaceProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl AddSubspaceProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::ADD_SUBSPACE_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RemoveSubspaceProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
}

impl RemoveSubspaceProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            &Proposal::new_id(&proposal.onchain_proposal_id),
            system_ids::INDEXER_SPACE_ID,
            Self {proposal}
        )
        .with_type(system_ids::PROPOSAL_TYPE)
        .with_type(system_ids::REMOVE_SUBSPACE_PROPOSAL)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProposedSubspace;

impl ProposedSubspace {
    pub fn new(
        subspace_proposal_id: &str,
        subspace_id: &str,
    ) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("{}-{}", subspace_proposal_id, subspace_id)),
            system_ids::INDEXER_SPACE_ID,
            subspace_proposal_id,
            subspace_id,
            Self {},
        )
        .with_type(system_ids::PROPOSED_SUBSPACE)
    }
}