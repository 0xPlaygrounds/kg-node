// ================================================================
// Ported from https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/core/ids/system.ts
// ================================================================
pub const TYPES: &str = "KEyred99SGesjDMcbB1oD2";
pub const ATTRIBUTE: &str = "LgKenoh2EfWrvqJqN6A7Ci";
pub const SCHEMA_TYPE: &str = "VdTsW1mGiy1XSooJaBBLc4";
pub const ATTRIBUTES: &str = "CBZs8pfSk5WHdujaAAKdD8";

pub const NAME: &str = "GG8Z4cSkjv8CywbkLqVU5M";
pub const DESCRIPTION: &str = "QoTdezDypdQrs7wq1tTRnb";
pub const AVATAR_ATTRIBUTE: &str = "399xP4sGWSoepxeEnp3UdR";
pub const COVER_ATTRIBUTE: &str = "DTEcNh4xFNvsqoX9bfF6qS";

// Value types
pub const VALUE_TYPE: &str = "JwYkkjY2i6uuR4wrgFScwt";
pub const CHECKBOX: &str = "NtWv5uGJ1d15Mfk4ZdXfmU";
pub const DATE: &str = "UGr1YqqZbE2BbEpJR9U88H";
pub const TEXT: &str = "JBYTdEigecQHj2xhL3NeHV";
pub const WEB_URL: &str = "5EQJAVKYDQWHZSfsawBtWa";
pub const NUMBER: &str = "SrhnoheQWdZMxvdgynGXiK";

// Relations are a data model that enable us to create references between some
// arbitrary id and a set of entity ids.
//
// They act similarly to Join Tables in a relational database, but are designed
// around the graph-based nature of the Geo data model.
//
// Relations are themselves entities, so can store any metadata about the relation
// as triples. Currently Relation entities cannot have their own relations. This is a
// technical limitation to avoid infinitely creating recursive relations.
//
// ┌─────────────────────┐       ┌────────────────────┐      ┌──────────────────────┐
// │                     │       │                    │      │                      │
// │      Entity         │◄──────┤     Relation       │─────►│        Entity        │
// │                     │       │                    │      │                      │
// └─────────────────────┘       └────────────────────┘      └──────────────────────┘

/// Relation type. This is the entity representing the Join between the
/// the Collection and the Entity
pub const RELATION: &str = "AKDxovGvZaPSWnmKnSoZJY";

/// Defines the relation value types for a relation. e.g., a Persons
/// attribute must only contain relations where the to entity is type
/// Person
pub const RELATION_VALUE_RELATIONSHIP_TYPE: &str = "LdAS7yWqF32E2J4doUDe5u";

pub const IMAGE: &str = "WpZ6MDcJZrfheC3XD7hyhh";
pub const IMAGE_FILE_TYPE_ATTRIBUTE: &str = "B3nyKkmERhFEcaVgoe6kAL";
pub const IMAGE_HEIGHT_ATTRIBUTE: &str = "GjaFuBBB8z63y9qr8dhaSP";
pub const IMAGE_URL_ATTRIBUTE: &str = "J6cw1v8xUHCFsEdPeuB1Uo";
pub const IMAGE_WIDTH_ATTRIBUTE: &str = "Xb3useEUkWV1Y9zYYkq4xp";

pub const BLOCKS: &str = "XbVubxtJCexLsmEhTUKPG";

// Data blocks
// @TODO: data block instead of TABLE_BLOCK
pub const DATA_BLOCK: &str = "PnQsGwnnztrLNRCm9mcKKY";
pub const DATA_SOURCE_ATTRIBUTE: &str = "J8nmVHZDeCLNhPxX7qyEZG";
pub const ALL_OF_GEO_DATA_SOURCE: &str = "XqDkiYjqEufsbjqegxkqZU";
pub const COLLECTION_DATA_SOURCE: &str = "3J6223VX6MkwTftWdzDfo4";
pub const QUERY_DATA_SOURCE: &str = "8HkP7HCufp2HcCFajuJFcq";

/**
 * Defines the filters applied to a data block. It applies whether
 * the data block is a COLLECTION or a QUERY.
 *
 * See the Filters spec to see the Filter format.
 */
pub const FILTER: &str = "3YqoLJ7uAPmthXyXmXKoSa";
pub const SPACE_FILTER: &str = "JiFmyuFYeoiRSiY286m7A2";

/**
 * Defines that a relation type is COLLECTION_ITEM. This is used by
 * data blocks with the [COLLECTION_DATA_SOURCE] to denote that a relation
 * should be consumed as a collection item in the block.
 *
 * Collections enable data blocks to render arbitrarily selected entities
 * rather than the results of a query. A collection item defines what
 * these selected entities are.
 */
pub const COLLECTION_ITEM_RELATION_TYPE: &str = "Mwrn46KavwfWgNrFaWcB9j";

/**
 * Defines the view type for a data block, either a [TABLE_VIEW],
 * [GALLERY_VIEW] or a [LIST_VIEW]. Your own application
 * can define their own views with their own IDs.
 *
 * This view data lives on the relation pointing to the block. This enables
 * different consumers of the data block to define their own views.
 */
pub const VIEW_ATTRIBUTE: &str = "46GzPiTRPG36jX9dmNE9ic";

pub const VIEW_TYPE: &str = "52itq1wC2HciX6gd9HEZPN";
pub const TABLE_VIEW: &str = "S9T1TPras3iPkVvrS5CoKE";
pub const LIST_VIEW: &str = "GUKPGARFBFBMoET6NGQctJ";
pub const GALLERY_VIEW: &str = "SHBs5faKV8gDeZgsUoVUQF";

/**
 * Defines the columns to show on a data block when in [TABLE_VIEW].
 */
pub const SHOWN_COLUMNS: &str = "9AecPe8JTN7uJRaX1Mk1XV";

/**
 * Defines the type of data source for a data block, either a
 * {@link COLLECTION_DATA_SOURCE}, an {@link ALL_OF_GEO_DATA_SOURCE} or
 * a {@link QUERY_DATA_SOURCE}
 */
pub const DATA_SOURCE_TYPE_RELATION_TYPE: &str = "4sz7Kx91uq4KBW5sohjLkj";

pub const IMAGE_BLOCK: &str = "V6R8hWrKfLZmtyv4dQyyzo";
pub const TEXT_BLOCK: &str = "Fc836HBAyTaLaZgBzcTS2a";

pub const MARKDOWN_CONTENT: &str = "V9A2298ZHL135zFRH4qcRg";

/**
 * Relations define an entity which represents a relationship between
 * two entities. Modeling this relationship as a from -> to means that
 * we can add extra data about the relationship to the entity itself
 * rather than to either of the from or to entities.
 *
 * e.g., John and Jane are married and you want to add a marriage date
 * to represent when they were married. It does not make sense to add
 * the date to John and Jane directly, since the data is about the
 * marriage itself, and not John or Jane. This representation of the
 * marriage also only exists in the context of John and Jane.
 */
pub const RELATION_TYPE: &str = "XAeYjgogh9zKBz4g8pB9wG";
pub const RELATION_FROM_ATTRIBUTE: &str = "3ZZFJ1dDBk7zTvN5x3XRR3";
pub const RELATION_TO_ATTRIBUTE: &str = "NToMyNnNkCvFh1McQLm4Rm";
pub const RELATION_TYPE_ATTRIBUTE: &str = "DGKmqmiyVPZ7Tfe18VksjN";

/*
 * Relations can be ordered using fractional indexing. By default we
 * add an index to every relation so that ordering can be added to
 * any relation at any point without having to add indexes to relations
 * post-creation.
 */
pub const RELATION_INDEX: &str = "gEfvT3cW16tyPmFEGA9bp";

/**
 * Defines whether a relation has been "verified." Verification can
 * mean different things semantically for different spaces. This
 * flag provides a means for spaces to build UIs or tooling around
 * a semantically verified entity. It's possible for relations to
 * point to entities which aren't verified, and it's up to spaces
 * to decide what "verified" means for them.
 *
 * e.g.
 * a link to a Person might be verified in that the linked space
 * is the correct one to represent this Person from the perspective
 * of the current space.
 */
pub const VERIFIED_SOURCE_ATTRIBUTE: &str = "5jodArZNFzucsYzQaDVFBL";
pub const SOURCE_SPACE_ATTRIBUTE: &str = "GzkEQP3yedWjXE8QPFKEwV";

// Core types
pub const COMPANY_TYPE: &str = "UhpHYoFEzAov9WwqtDwQk4";
pub const NONPROFIT_TYPE: &str = "RemzN69c24othsp2rP7yMX";
pub const POST_TYPE: &str = "X7KuZJQewaCiCy9QV2vjyv";
pub const PROJECT_TYPE: &str = "9vk7Q3pz7US3s2KePFQrJT";
pub const SPACE_CONFIGURATION: &str = "EXWTH2k6qquguZ8CCfMp9K";

// Templates
pub const TEMPLATE_ATTRIBUTE: &str = "Sb7ZvdGsCDm2r1mNZBA5ft";
pub const PAGE_TYPE: &str = "9u4zseS3EDXG9ZvwR9RmqU";

/**
 * Defines the page type for a template. e.g., an events page, a
 * finances page, a products page, etc.
 */
pub const PAGE_TYPE_ATTRIBUTE: &str = "DD9FKRZ3XezaKEGUszMB3r";

// These define the entity id to copy when creating an entity from
// a template.

pub const COMPANY_EVENTS_PAGE_TEMPLATE: &str = "4CTRE9hBWqb7CjiaxQx47C";
pub const COMPANY_JOBS_PAGE_TEMPLATE: &str = "DSANGC24exwsRWXrfikKb7";
pub const COMPANY_POSTS_PAGE_TEMPLATE: &str = "AHLknvNrbs7CBao2i58mo5";
pub const COMPANY_PRODUCTS_PAGE_TEMPLATE: &str = "7Dp2MBb1tjMk6igDaYTZtb";
pub const COMPANY_SERVICES_PAGE_TEMPLATE: &str = "NRLUry4uMctKx6yiC2GP9F";
pub const COMPANY_SPACE_CONFIGURATION_TEMPLATE: &str = "QZwChwTixtbLDv3HSX5E6n";
pub const COMPANY_TEAM_PAGE_TEMPLATE: &str = "B59SUroy7uy9yCHF9AD9mP";
pub const PERSON_SPACE_CONFIGURATION_TEMPLATE: &str = "EJuFuEz17wdVCk9ctEAkW7";
pub const PERSON_POSTS_PAGE_TEMPLATE: &str = "98wgvodwzidmVA4ryVzGX6";
pub const NONPROFIT_FINANCES_PAGE_TEMPLATE: &str = "G3PRyzNzRNWn4m7S4sESQG";
pub const NONPROFIT_ID_NUMBER_ATTRIBUTE: &str = "Qv1R7wDaem6uBTE5TYQihB";
pub const NONPROFIT_POSTS_PAGE_TEMPLATE: &str = "G8iePrDZk2SkqL9QEW6nCR";
pub const NONPROFIT_PROJECTS_PAGE_TEMPLATE: &str = "JkJDTY4f3Xc6APZKna5kGh";
pub const NONPROFIT_SERVICE_TYPE: &str = "GZao3GpaUjMrX14VB2LoNR";
pub const NONPROFIT_SPACE_CONFIGURATION_TEMPLATE: &str = "HEuj9VYAF5z1KQ8x37Uzze";
pub const NONPROFIT_TEAM_PAGE_TEMPLATE: &str = "K51CbDqxW35osbjPo5ZF77";

// Defines the type of the page being copied when creating an entity
// from a template.
pub const POSTS_PAGE: &str = "E3jboNrTeuopjKgJ45ykBd";
pub const PRODUCTS_PAGE: &str = "Cnf53HgY8T7Fwcq8choaRn";
pub const PROJECTS_PAGE: &str = "3scJVFciFuhmaXe852pT3F";
pub const SERVICES_PAGE: &str = "2V8pajmGDJt8egodkJeoPC";
pub const SPACES_PAGE: &str = "JAPV1HvzUBXH1advi47FWN";
pub const TEAM_PAGE: &str = "BWMHGbpR31xTbjvk4QZdQA";
pub const EVENTS_PAGE: &str = "K97FaTqrx54jdiM93vZ1Fc";
pub const FINANCES_PAGE: &str = "R6FDYEK9CCdEQuxjuRjA2U";
pub const JOBS_PAGE: &str = "PJzxY3isAL3hGx1bRkYdPf";

pub const FINANCE_OVERVIEW_TYPE: &str = "5LHixDnR2vBTx26kmbnyih";
pub const FINANCE_SUMMMARY_TYPE: &str = "8zrMWkTeDkfxbGn1U1MjLx";

// Identity related ids
pub const ACCOUNT_TYPE: &str = "S7rX6suDMmU75yjbAD5WsP";
pub const ACCOUNTS_ATTRIBUTE: &str = "VA5i7mm1v3QMjUChMT5dPs";
pub const ADDRESS_ATTRIBUTE: &str = "HXLpAZyQkcy6Di4YJu4xzU";
pub const NETWORK_ATTRIBUTE: &str = "MuMLDVbHAmRjZQjhyk3HGx";
pub const PERSON_TYPE: &str = "GfN9BK2oicLiBHrUavteS8";
pub const NETWORK_TYPE: &str = "YCLXoVZho6C4S51g4AbF3C";

pub const CLAIM_TYPE: &str = "KeG9eTM8NUYFMAjnsvF4Dg";
pub const BROADER_CLAIMS_ATTRIBUTE: &str = "RWkXuBRdVqDAiHKQisTZZ4";
pub const BROADER_GOALS_ATTRIBUTE: &str = "EtNH2yJmrEK1Mcv7S5MjPN";
pub const BROADER_SPACES: &str = "CHwmK8bk4KMCqBNiV2waL9";
pub const BROADER_TOPICS_ATTRIBUTE: &str = "P9apCagMDXQVdjgAZftxU5";
pub const CLAIMS_FROM_ATTRIBUTE: &str = "JdNBawSt1fp9EdozJdmThR";
pub const DEFINITIONS_ATTRIBUTE: &str = "256myJaotY6FB6wGiC5mtk";
pub const EMAIL_ATTRIBUTE: &str = "2QafYRmRHP2Hd18W3Tj9zu";
pub const FOREIGN_TYPES: &str = "R32hqus9ojU3Twsz3HDuxf";
pub const GOALS_ATTRIBUTE: &str = "WNcdorfdj7ZprmwvmRiRtG";
pub const GOAL_TYPE: &str = "2y44qmFiLjZWmgkZ64EM7c";
pub const MEMBERSHIP_CONTRACT_ADDRESS: &str = "DDkwCoB8p1mHzXTedShcFv";
pub const MISSION_ATTRIBUTE: &str = "VGbyCo12NC8yTUhnhMHu1z";
pub const NONPROFIT_CATEGORIES_ATTRIBUTE: &str = "64uVL5vKHmfqBC94hwNzHZ";
pub const OPPOSING_ARGUMENTS_ATTRIBUTE: &str = "Agk2hbiBWsgHVXxpFAc7z5";
pub const PERSON_ATTRIBUTE: &str = "W2aFZPy5nnU3DgdkWJCNVn";
pub const PHONE_NUMBER_ATTRIBUTE: &str = "3zhuyrcqFjeaVgC5oHHqTJ";
pub const PLACEHOLDER_IMAGE: &str = "ENYn2afpf2koqBfyff7CGE";
pub const PLACEHOLDER_TEXT: &str = "AuihGk1yXCkfCcpMSwhfho";
pub const QUOTES_ATTRIBUTE: &str = "XXAf2w4C5f4URDhhpH8nUG";
pub const REGION_ATTRIBUTE: &str = "CGC6KXy8wcqf7vpZv8HH4i";
pub const REGION_TYPE: &str = "Qu6vfQq68ecZ4PkihJ4nZN";
pub const RELATED_TOPICS_ATTRIBUTE: &str = "SDw38koZeFukda9FWU9bfW";
pub const RELEVANT_QUESTIONS_ATTRIBUTE: &str = "Po4uUtzinhjDwXJP5QNCMp";
pub const ROLE_ATTRIBUTE: &str = "VGKSRGzxCRvQxpJP7CB4wj";
pub const SOURCES_ATTRIBUTE: &str = "A7NJF2WPh8VhmvbfVWiyLo";
pub const SPEAKERS_ATTRIBUTE: &str = "9nZuGhssmkEBn9DtRca8Gm";
pub const STREET_ADDRESS_ATTRIBUTE: &str = "8kx7oQvdCZRXLfUksucwCv";
pub const SUBCLAIMS_ATTRIBUTE: &str = "2DFyYPbh5Yy2PnWTbi3uL5";
pub const SUBGOALS_ATTRIBUTE: &str = "WX9xtsWqFPmAXcTBF833cF";
pub const SUBTOPICS_ATTRIBUTE: &str = "89cuddDgriDAZJV6oy8zmt";
pub const SUPPORTING_ARGUMENTS_ATTRIBUTE: &str = "4gFz5SYHWkBJoAANfCt61v";
pub const TAB_TYPE: &str = "6ym81VzJiHx32nV8e5h52q";
pub const TAGS_ATTRIBUTE: &str = "5d9VVey3wusmk98Uv3v5LM";
pub const TAG_TYPE: &str = "UnP1LtXV3EhrhvRADFcMZK";
pub const TOPICS_ATTRIBUTE: &str = "9bCuX6B9KZDSaY8xtghNZo";
pub const TOPIC_TYPE: &str = "Cj7JSjWKbcdgmUjcLWNR4V";
pub const VALUES_ATTRIBUTE: &str = "3c5k2MpF9PRYAZ925qTKNi";
pub const VISION_ATTRIBUTE: &str = "AAMDNTaJtS2i4aWp59zEAk";
pub const WEB_URL_ATTRIBUTE: &str = "93stf6cgYvBsdPruRzq1KK";

// Not sure what this is
pub const DEFAULT_TYPE: &str = "7nJuuYkrKT62HCFxDygF1S";

// ================================================================
// Indexer Space
// ================================================================
// System attributes
pub const CREATED_AT_TIMESTAMP: &str = "82nP7aFmHJLbaPFszj2nbx";
pub const CREATED_AT_BLOCK: &str = "59HTYnd2e4gBx2aA98JfNx";
pub const UPDATED_AT_TIMESTAMP: &str = "5Ms1pYq8v8G1RXC3wWb9ix";
pub const UPDATED_AT_BLOCK: &str = "7pXCVQDV9C7ozrXkpVg8RJ";

/// Geo Account
pub const GEO_ACCOUNT: &str = "FUS7nfaWo8Ugzv9AxsMPNX";

// Space types and attributes
pub const INDEXED_SPACE: &str = "XN8KqVDUoFvvBkL4tFX17u";
pub const SPACE: &str = "HrFMSEDZ9kFAeJhhkHdsCQ";

pub const SPACE_PLUGIN_ADDRESS: &str = "8WQFgjEoqNYHPS9niQGcfC";
pub const SPACE_VOTING_PLUGIN_ADDRESS: &str = "GMuFbsoSsVmiMcCxc34zZA";
pub const SPACE_MEMBER_PLUGIN_ADDRESS: &str = "AGaTTZWAbEaSrmZYTinQuc";
/// Type of space, e.g., Public or Personal
pub const SPACE_KIND: &str = "DEnbDZp79wgfvUXMGJPXDW";

/// GEO_ACCOUNT > MEMBER_RELATION > INDEXED_SPACE
pub const MEMBER_RELATION: &str = "2oGooh2PEUo8pbdMPqcBrQ";

/// GEO_ACCOUNT > EDITOR_RELATION > INDEXED_SPACE
pub const EDITOR_RELATION: &str = "24TNsjdHUhuC5qtsN13stv";

/// SPACE > PARENT_SPACE > SPACE
pub const PARENT_SPACE: &str = "4jLdUCbpRzdjnpjhViDm2d";

/// Special space for the indexer data
pub const INDEXER_SPACE_ID: &str = "Y7z7eeWdupgZayxG9Lvi2Z";

// Voting
/// GEO_ACCOUNT > VOTE_CAST > PROPOSAL
pub const VOTE_CAST: &str = "PfgzdxPYwDUTBCzkXCT9ga";

// Proposal
pub const PROPOSAL_TYPE: &str = "9No6qfEutiKg1WLeXDv73x";
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

/// INDEXED_SPACE > PROPOSALS > PROPOSAL
pub const PROPOSALS: &str = "3gmeTonVCB6B11p3YF8mj5";

/// PROPOSAL > CREATOR > ACCOUNT
pub const PROPOSAL_CREATOR: &str = "213";
