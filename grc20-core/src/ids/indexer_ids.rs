use uuid::{uuid, Uuid};

/// Special space for the indexer data
pub const INDEXER_SPACE_ID: Uuid = uuid!("fc04f58f-e2bd-4662-ade7-eb6846bec3d6");

// System attributes
pub const CREATED_AT_TIMESTAMP: Uuid = uuid!("38efae7c-898a-44ce-89a0-ab4bad67d227");
pub const CREATED_AT_BLOCK: Uuid = uuid!("218ca5b1-14c6-41ff-af00-58eb2b913729");
pub const UPDATED_AT_TIMESTAMP: Uuid = uuid!("234e4099-b794-4014-8005-9db2561c2c81");
pub const UPDATED_AT_BLOCK: Uuid = uuid!("3739663e-5815-4323-987e-9969635fbe75");

// Space attributes
/// Type of space, e.g., Public or Personal
pub const SPACE_GOVERNANCE_TYPE: Uuid = uuid!("631a6dc1-4084-498f-874f-1f3fe284b3ed");
pub const SPACE_DAO_ADDRESS: Uuid = uuid!("712f426c-b724-4315-a979-f870484da2ec");
pub const SPACE_PLUGIN_ADDRESS: Uuid = uuid!("3ccadec7-d838-4f08-879f-37b72c864a3b");
pub const SPACE_VOTING_PLUGIN_ADDRESS: Uuid = uuid!("7c642815-d095-4504-8e62-2a3b64090c05");
pub const SPACE_MEMBER_PLUGIN_ADDRESS: Uuid = uuid!("4b0f05c8-54d5-4056-9da0-39db5f2050bf");
pub const SPACE_PERSONAL_PLUGIN_ADDRESS: Uuid = uuid!("723965ce-3dc6-40b4-85dd-bcee75193c8f");

/// GEO_ACCOUNT > MEMBER_RELATION > INDEXED_SPACE
pub const MEMBER_RELATION: Uuid = uuid!("0e8f17ee-4156-4b1c-9b42-9b24c2690bd1");

/// GEO_ACCOUNT > EDITOR_RELATION > INDEXED_SPACE
pub const EDITOR_RELATION: Uuid = uuid!("0894a01e-956e-457c-8fbb-bca05e2c0b3b");

/// SPACE > PARENT_SPACE > SPACE
pub const PARENT_SPACE: Uuid = uuid!("1e34c040-63fb-4165-88cb-8e5eacbe5d7e");

// Cursor
/// Cursor type ID
pub const CURSOR_TYPE: Uuid = uuid!("3bacc212-be34-44ab-95a4-5bb694a2c9e4");
/// Cursor ID
pub const CURSOR_ID: Uuid = uuid!("43d42395-6373-409a-ad06-11710429a70b");
pub const CURSOR_ATTRIBUTE: Uuid = uuid!("2d8ef4e9-fb9b-4908-b3d1-8a714e16c7c6");
pub const BLOCK_NUMBER_ATTRIBUTE: Uuid = uuid!("3dc13be8-6cc9-2eee-cb41-f00dc956c7c6");
pub const BLOCK_TIMESTAMP_ATTRIBUTE: Uuid = uuid!("44d9a4ee-598f-2b59-a3e8-0650f6617653");
pub const VERSION_ATTRIBUTE: Uuid = uuid!("7e6478f2-964f-2426-bbd1-52373735a32b");
