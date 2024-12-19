//! This module contains models reserved for use by the KG Indexer.

use serde::{Deserialize, Serialize};

use crate::{
    ids, mapping::{Relation}, system_ids
};


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoteType {
    Accept,
    Reject,
}

impl TryFrom<u64> for VoteType {
    type Error = String;

    fn try_from(vote: u64) -> Result<Self, Self::Error> {
        match vote {
            2 => Ok(Self::Accept),
            3 => Ok(Self::Reject),
            _ => Err(format!("Invalid vote type: {}", vote)),
        }
    }
}

/// A vote cast by a user on a proposal.
/// 
/// `Person > VOTE_CAST > Proposal`
#[derive(Deserialize, Serialize)]
pub struct VoteCast {
    pub vote_type: VoteType,
}

impl VoteCast {
    pub fn new_id(
        account_id: &str,
        proposal_id: &str,

    ) -> String {
        ids::create_id_from_unique_string(&format!("{account_id}-{proposal_id}"))
    }

    /// Creates a new vote cast with the given vote type.
    pub fn new(
        account_id: &str,
        proposal_id: &str,
        vote_type: VoteType
    ) -> Relation<Self> {
        Relation::new(
            &Self::new_id(account_id, proposal_id),
            system_ids::INDEXER_SPACE_ID,
            account_id,
            proposal_id,
            Self {
                vote_type,
            },
        )
    }
}