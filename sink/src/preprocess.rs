use crate::events::Edit;
use grc20_core::{block::BlockMetadata, pb::chain};

pub struct EventData {
    pub block: BlockMetadata,

    // pub spaces_created: Vec<geo::GeoSpaceCreated>,
    // pub governance_plugins_created: Vec<geo::GeoGovernancePluginCreated>,
    // pub initial_editors_added: Vec<geo::InitialEditorAdded>,
    // pub votes_cast: Vec<geo::VoteCast>,
    pub edits_published: Vec<(chain::EditPublished, Vec<Edit>)>,
    // pub successor_spaces_created: Vec<geo::SuccessorSpaceCreated>,
    // pub subspaces_added: Vec<geo::SubspaceAdded>,
    // pub subspaces_removed: Vec<geo::SubspaceRemoved>,
    // pub executed_proposals: Vec<geo::ProposalExecuted>,
    // pub members_added: Vec<geo::MemberAdded>,
    // pub editors_added: Vec<geo::EditorAdded>,
    // pub personal_plugins_created: Vec<geo::GeoPersonalSpaceAdminPluginCreated>,
    // pub members_removed: Vec<geo::MemberRemoved>,
    // pub editors_removed: Vec<geo::EditorRemoved>,
    // pub edits: Vec<geo::PublishEditProposalCreated>,
    // pub proposed_added_members: Vec<geo::AddMemberProposalCreated>,
    // pub proposed_removed_members: Vec<geo::RemoveMemberProposalCreated>,
    // pub proposed_added_editors: Vec<geo::AddEditorProposalCreated>,
    // pub proposed_removed_editors: Vec<geo::RemoveEditorProposalCreated>,
    // pub proposed_added_subspaces: Vec<geo::AddSubspaceProposalCreated>,
    // pub proposed_removed_subspaces: Vec<geo::RemoveSubspaceProposalCreated>,
}
