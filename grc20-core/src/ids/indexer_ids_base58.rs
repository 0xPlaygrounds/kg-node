/// Special space for the indexer data
pub const INDEXER_SPACE_ID: &str = "Y7z7eeWdupgZayxG9Lvi2Z";

// System attributes
pub const CREATED_AT_TIMESTAMP: &str = "82nP7aFmHJLbaPFszj2nbx";
pub const CREATED_AT_BLOCK: &str = "59HTYnd2e4gBx2aA98JfNx";
pub const UPDATED_AT_TIMESTAMP: &str = "5Ms1pYq8v8G1RXC3wWb9ix";
pub const UPDATED_AT_BLOCK: &str = "7pXCVQDV9C7ozrXkpVg8RJ";

// Space attributes
/// Type of space, e.g., Public or Personal
pub const SPACE_GOVERNANCE_TYPE: &str = "DEnbDZp79wgfvUXMGJPXDW";
pub const SPACE_DAO_ADDRESS: &str = "Eye1MG1mjzRba4poijsttb";
pub const SPACE_PLUGIN_ADDRESS: &str = "8WQFgjEoqNYHPS9niQGcfC";
pub const SPACE_VOTING_PLUGIN_ADDRESS: &str = "GMuFbsoSsVmiMcCxc34zZA";
pub const SPACE_MEMBER_PLUGIN_ADDRESS: &str = "AGaTTZWAbEaSrmZYTinQuc";
pub const SPACE_PERSONAL_PLUGIN_ADDRESS: &str = "F75rm9StiixRKWTRiHGgvS";

/// GEO_ACCOUNT > MEMBER_RELATION > INDEXED_SPACE
pub const MEMBER_RELATION: &str = "2oGooh2PEUo8pbdMPqcBrQ";

/// GEO_ACCOUNT > EDITOR_RELATION > INDEXED_SPACE
pub const EDITOR_RELATION: &str = "24TNsjdHUhuC5qtsN13stv";

/// SPACE > PARENT_SPACE > SPACE
pub const PARENT_SPACE: &str = "4jLdUCbpRzdjnpjhViDm2d";

// Voting
/// GEO_ACCOUNT > VOTE_CAST > PROPOSAL
pub const VOTE_CAST_TYPE: &str = "PfgzdxPYwDUTBCzkXCT9ga";
pub const VOTE_TYPE_ATTRIBUTE: &str = "UrTNUMukLGV2y4yWEnBKQv";

// Proposal
pub const PROPOSAL_TYPE: &str = "9No6qfEutiKg1WLeXDv73x";
pub const PROPOSAL_STATUS_ATTRIBUTE: &str = "status";
pub const ADD_MEMBER_PROPOSAL: &str = "6dJ23LRTHRdwqoWhtivRrM";
pub const REMOVE_MEMBER_PROPOSAL: &str = "8dJ23LRTHRdwqoWhtivRrM";
pub const ADD_EDITOR_PROPOSAL: &str = "7W7SE2UTj5YTsQvqSmCfLN";
pub const REMOVE_EDITOR_PROPOSAL: &str = "9W7SE2UTj5YTsQvqSmCfLN";
pub const ADD_SUBSPACE_PROPOSAL: &str = "DcEZrRpmAuwxfw7C5G7gjC";
pub const REMOVE_SUBSPACE_PROPOSAL: &str = "FcEZrRpmAuwxfw7C5G7gjC";
pub const EDIT_PROPOSAL: &str = "GcEZrRpmAuwxfw7C5G7gjC";

/// MEMBERSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
/// EDITORSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
pub const PROPOSED_ACCOUNT: &str = "N95e2Jvfe3VX5d6jc9owY6";

/// SUBSPACE_PROPOSAL_TYPE > PROPOSED_SUBSPACE > INDEXED_SPACE
pub const PROPOSED_SUBSPACE: &str = "5ZVrZv7S3Mk2ATV9LAZAha";

/// PROPOSAL > CREATOR > ACCOUNT
pub const PROPOSAL_CREATOR: &str = "FUS7nfaWo8Ugzv9AxsMPNX";

/// INDEXED_SPACE > PROPOSALS > PROPOSAL
pub const PROPOSALS: &str = "3gmeTonVCB6B11p3YF8mj5";

// Edits
/// Edit type ID
pub const EDIT_TYPE: &str = "Q2dfub76oMdzJpyR8z4GZv";
pub const EDIT_INDEX_ATTRIBUTE: &str = "edit_index";
pub const EDIT_CONTENT_URI_ATTRIBUTE: &str = "content_uri";

/// Proposal > PROPOSED_EDIT > Edit
pub const PROPOSED_EDIT: &str = "8NPzSYo8fXBeFHs7WvGtif";
/// Space > EDITS > Edit
pub const EDITS: &str = "QRkn8QWyKjo1sKmpVKsoUJ";

// Cursor
/// Cursor type ID
pub const CURSOR_TYPE: &str = "CURSOR_TYPE"; // TODO: Replace by GRC20 ID
/// Cursor ID
pub const CURSOR_ID: &str = "grc20-cursor"; // TODO: Replace by GRC20 ID
pub const CURSOR_ATTRIBUTE: &str = "cursor-attribute";
pub const BLOCK_NUMBER_ATTRIBUTE: &str = "block-number-attribute";
pub const BLOCK_TIMESTAMP_ATTRIBUTE: &str = "block-timestamp-attribute";
pub const VERSION_ATTRIBUTE: &str = "version-attribute";
