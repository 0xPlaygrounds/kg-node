// ================================================================
// Ported from https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/core/ids/system.ts
// ================================================================
pub const PROPERTY_TYPE: &str = "GscJ2GELQjmLoaVrYyR3xm";
pub const SCHEMA_TYPE: &str = "VdTsW1mGiy1XSooJaBBLc4";
pub const PROPERTIES: &str = "9zBADaYzyfzyFJn4GU1cC";

pub const NAME_ATTRIBUTE: &str = "LuBWqZAu6pz54eiJS5mLv8";
pub const DESCRIPTION_ATTRIBUTE: &str = "LA1DqP5v6QAdsgLPXGF3YA";
pub const COVER_ATTRIBUTE: &str = "7YHk6qYkNDaAtNb8GwmysF";
pub const TYPES_ATTRIBUTE: &str = "Jfmby78N4BCseZinBmdVov";
pub const BLOCKS: &str = "QYbjCM6NT9xmh2hFGsqpQX";

// Value types
pub const VALUE_TYPE_ATTRIBUTE: &str = "WQfdWjboZWFuTseDhG5Cw1";
pub const CHECKBOX: &str = "G9NpD4c7GB7nH5YU9Tesgf";
pub const TIME: &str = "3mswMrL91GuYTfBq29EuNE";
pub const TEXT: &str = "LckSTmjBrYAJaFcDs89am5";
pub const URL: &str = "5xroh3gbWYbWY4oR3nFXzy";
pub const NUMBER: &str = "LBdMpTNyycNffsF51t2eSp";
pub const POINT: &str = "UZBZNbA7Uhx1f8ebLi1Qj5";
pub const IMAGE: &str = "X8KB1uF84RYppghBSVvhqr";

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
pub const RELATION_SCHEMA_TYPE: &str = "AKDxovGvZaPSWnmKnSoZJY";

pub const SPACE_TYPE: &str = "7gzF671tq5JTZ13naG4tnr";

/// Defines the relation value types for a relation. e.g., a Persons
/// attribute must only contain relations where the to entity is type
/// Person
pub const RELATION_VALUE_RELATIONSHIP_TYPE: &str = "LdAS7yWqF32E2J4doUDe5u";

pub const IMAGE_TYPE: &str = "Q1LaZhnzj8AtCzx8T1HRMf";
pub const IMAGE_FILE_TYPE_ATTRIBUTE: &str = "B3nyKkmERhFEcaVgoe6kAL";
pub const IMAGE_HEIGHT_ATTRIBUTE: &str = "GjaFuBBB8z63y9qr8dhaSP";
pub const IMAGE_URL_ATTRIBUTE: &str = "J6cw1v8xUHCFsEdPeuB1Uo";
pub const IMAGE_WIDTH_ATTRIBUTE: &str = "Xb3useEUkWV1Y9zYYkq4xp";

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
pub const ENTITY_FILTER: &str = "TWrUhTe6E8tKCJr9vfCzxT";

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
pub const RELATION_TYPE: &str = "QtC4Ay8HNLwSd1kSARgcDE";
pub const RELATION_FROM_ATTRIBUTE: &str = "RERshk4JoYoMC17r1qAo9J";
pub const RELATION_TO_ATTRIBUTE: &str = "Qx8dASiTNsxxP3rJbd4Lzd";
pub const RELATION_TYPE_ATTRIBUTE: &str = "3WxYoAVreE4qFhkDUs5J3q";

/*
 * Relations can be ordered using fractional indexing. By default we
 * add an index to every relation so that ordering can be added to
 * any relation at any point without having to add indexes to relations
 * post-creation.
 */
pub const RELATION_INDEX: &str = "WNopXUYxsSsE51gkJGWghe";

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
pub const ACADEMIC_FIELD_TYPE: &str = "ExCjm3rzYVfpMRwDchdrEe";
pub const COMPANY_TYPE: &str = "UhpHYoFEzAov9WwqtDwQk4";
pub const DAO_TYPE: &str = "Hh8hc47TMBvRsD4oqUNxP9";
pub const GOVERNMENT_ORG_TYPE: &str = "MokrHqV4jZhBfPN3mLPjM8";
pub const INDUSTRY_TYPE: &str = "YA7mhzaafD2vnjekmcnLER";
pub const INTEREST_GROUP_TYPE: &str = "6VSi54UDKnL34BWHBqdzee";
pub const NONPROFIT_TYPE: &str = "RemzN69c24othsp2rP7yMX";
pub const POST_TYPE: &str = "X7KuZJQewaCiCy9QV2vjyv";
pub const PROJECT_TYPE: &str = "9vk7Q3pz7US3s2KePFQrJT";
pub const PROTOCOL_TYPE: &str = "R9Xo87Q6oaxfSBrHvVQFdS";
pub const REGION_TYPE: &str = "Qu6vfQq68ecZ4PkihJ4nZN";
pub const ROOT_SPACE_TYPE: &str = "k7vbnMPxzdtGL2J3uaB6d";

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
pub const INDUSTRY_ABOUT_PAGE_TEMPLATE: &str = "P6tM8upNJ83frWtMtmHXve";
pub const INDUSTRY_EVENTS_PAGE_TEMPLATE: &str = "NVZnC2af2v3ptuNqjMC9Mu";
pub const INDUSTRY_JOBS_PAGE_TEMPLATE: &str = "RNtMm9U2zmevNCQTHHwbgb";
pub const INDUSTRY_NEWS_PAGE_TEMPLATE: &str = "2GXJSWAqkw5B8JtvWZ9hnX";
pub const INDUSTRY_ONTOLOGY_PAGE_TEMPLATE: &str = "AkJKSSB2AbQKXAWjqYV9Qh";
pub const INDUSTRY_OVERVIEW_PAGE_TEMPLATE: &str = "Tgm2ubmm9VjAZFZACJLTZC";
pub const INDUSTRY_PEOPLE_PAGE_TEMPLATE: &str = "JMbQ5nH8wWLxRki6eQYLTB";
pub const INDUSTRY_PROJECTS_PAGE_TEMPLATE: &str = "WQLU4rAJLwBBSHcLN4uHad";
pub const COMPANY_EVENTS_PAGE_TEMPLATE: &str = "4CTRE9hBWqb7CjiaxQx47C";
pub const COMPANY_JOBS_PAGE_TEMPLATE: &str = "DSANGC24exwsRWXrfikKb7";
pub const COMPANY_OVERVIEW_PAGE_TEMPLATE: &str = "QZwChwTixtbLDv3HSX5E6n";
pub const COMPANY_POSTS_PAGE_TEMPLATE: &str = "AHLknvNrbs7CBao2i58mo5";
pub const COMPANY_PRODUCTS_PAGE_TEMPLATE: &str = "7Dp2MBb1tjMk6igDaYTZtb";
pub const COMPANY_SERVICES_PAGE_TEMPLATE: &str = "NRLUry4uMctKx6yiC2GP9F";
pub const COMPANY_TEAM_PAGE_TEMPLATE: &str = "B59SUroy7uy9yCHF9AD9mP";
pub const PERSON_OVERVIEW_PAGE_TEMPLATE: &str = "EJuFuEz17wdVCk9ctEAkW7";
pub const PERSON_POSTS_PAGE_TEMPLATE: &str = "98wgvodwzidmVA4ryVzGX6";
pub const NONPROFIT_FINANCES_PAGE_TEMPLATE: &str = "G3PRyzNzRNWn4m7S4sESQG";
pub const NONPROFIT_ID_NUMBER_ATTRIBUTE: &str = "Qv1R7wDaem6uBTE5TYQihB";
pub const NONPROFIT_OVERVIEW_PAGE_TEMPLATE: &str = "HEuj9VYAF5z1KQ8x37Uzze";
pub const NONPROFIT_POSTS_PAGE_TEMPLATE: &str = "G8iePrDZk2SkqL9QEW6nCR";
pub const NONPROFIT_PROJECTS_PAGE_TEMPLATE: &str = "JkJDTY4f3Xc6APZKna5kGh";
pub const NONPROFIT_SERVICE_TYPE: &str = "GZao3GpaUjMrX14VB2LoNR";
pub const NONPROFIT_TEAM_PAGE_TEMPLATE: &str = "K51CbDqxW35osbjPo5ZF77";
pub const ONTOLOGY_PAGE_TEMPLATE: &str = "JjEDWDpLYS7tYBKi4vCdwc";
pub const EDUCATION_PAGE_TEMPLATE: &str = "RHuzjGkGothjuwGN6TLXDx";
pub const ABOUT_PAGE_TEMPLATE: &str = "78qghJm6whF5proE9G93pX";

// Defines the type of the page being copied when creating an entity
// from a template.
pub const ABOUT_PAGE: &str = "Xn4sgn16Peoe64NSoARRv4";
pub const EDUCATION_PAGE: &str = "Qh8AQ8hQhXiaAbe8HkgAJV";
pub const EVENTS_PAGE: &str = "K97FaTqrx54jdiM93vZ1Fc";
pub const FINANCES_PAGE: &str = "R6FDYEK9CCdEQuxjuRjA2U";
pub const JOBS_PAGE: &str = "PJzxY3isAL3hGx1bRkYdPf";
pub const NEWS_PAGE: &str = "Ss6NVRoX8HKaEyFTEYNdUv";
pub const ONTOLOGY_PAGE: &str = "Ee2Es1b9PkfnR7nsbY2ubE";
pub const PEOPLE_PAGE: &str = "5PatoErephYf1ZJ8U6rNz8";
pub const POSTS_PAGE: &str = "E3jboNrTeuopjKgJ45ykBd";
pub const PRODUCTS_PAGE: &str = "Cnf53HgY8T7Fwcq8choaRn";
pub const PROJECTS_PAGE: &str = "3scJVFciFuhmaXe852pT3F";
pub const SERVICES_PAGE: &str = "2V8pajmGDJt8egodkJeoPC";
pub const SPACES_PAGE: &str = "JAPV1HvzUBXH1advi47FWN";
pub const TEAM_PAGE: &str = "BWMHGbpR31xTbjvk4QZdQA";

pub const FINANCE_OVERVIEW_TYPE: &str = "5LHixDnR2vBTx26kmbnyih";
pub const FINANCE_SUMMMARY_TYPE: &str = "8zrMWkTeDkfxbGn1U1MjLx";

// Identity
pub const ACCOUNT_TYPE: &str = "S7rX6suDMmU75yjbAD5WsP";
pub const ACCOUNTS_ATTRIBUTE: &str = "VA5i7mm1v3QMjUChMT5dPs";
pub const ADDRESS_ATTRIBUTE: &str = "HXLpAZyQkcy6Di4YJu4xzU";
pub const NETWORK_ATTRIBUTE: &str = "MuMLDVbHAmRjZQjhyk3HGx";
pub const PERSON_TYPE: &str = "GfN9BK2oicLiBHrUavteS8";
pub const NETWORK_TYPE: &str = "YCLXoVZho6C4S51g4AbF3C";

pub const GOALS_ATTRIBUTE: &str = "WNcdorfdj7ZprmwvmRiRtG";
pub const GOAL_TYPE: &str = "2y44qmFiLjZWmgkZ64EM7c";
pub const MEMBERSHIP_CONTRACT_ADDRESS: &str = "DDkwCoB8p1mHzXTedShcFv";
pub const MISSION_ATTRIBUTE: &str = "VGbyCo12NC8yTUhnhMHu1z";
pub const PLACEHOLDER_IMAGE: &str = "ENYn2afpf2koqBfyff7CGE";
pub const PLACEHOLDER_TEXT: &str = "AuihGk1yXCkfCcpMSwhfho";
pub const TAB_TYPE: &str = "6ym81VzJiHx32nV8e5h52q";
pub const ROLE_ATTRIBUTE: &str = "VGKSRGzxCRvQxpJP7CB4wj";

// Do we still need these?
pub const DEFAULT_TYPE: &str = "7nJuuYkrKT62HCFxDygF1S";
pub const BROADER_CLAIMS_ATTRIBUTE: &str = "RWkXuBRdVqDAiHKQisTZZ4";
pub const CLAIMS_FROM_ATTRIBUTE: &str = "JdNBawSt1fp9EdozJdmThR";
pub const DEFINITIONS_ATTRIBUTE: &str = "256myJaotY6FB6wGiC5mtk";
pub const EMAIL_ATTRIBUTE: &str = "2QafYRmRHP2Hd18W3Tj9zu";
pub const FOREIGN_TYPES: &str = "R32hqus9ojU3Twsz3HDuxf";
pub const NONPROFIT_CATEGORIES_ATTRIBUTE: &str = "64uVL5vKHmfqBC94hwNzHZ";
pub const PHONE_NUMBER_ATTRIBUTE: &str = "3zhuyrcqFjeaVgC5oHHqTJ";
pub const QUOTES_ATTRIBUTE: &str = "XXAf2w4C5f4URDhhpH8nUG";
pub const REGION_ATTRIBUTE: &str = "CGC6KXy8wcqf7vpZv8HH4i";
pub const RELATED_TOPICS_ATTRIBUTE: &str = "SDw38koZeFukda9FWU9bfW";
pub const RELEVANT_QUESTIONS_ATTRIBUTE: &str = "Po4uUtzinhjDwXJP5QNCMp";
pub const SPEAKERS_ATTRIBUTE: &str = "9nZuGhssmkEBn9DtRca8Gm";
pub const STREET_ADDRESS_ATTRIBUTE: &str = "8kx7oQvdCZRXLfUksucwCv";
pub const SUBCLAIMS_ATTRIBUTE: &str = "2DFyYPbh5Yy2PnWTbi3uL5";
pub const VALUES_ATTRIBUTE: &str = "3c5k2MpF9PRYAZ925qTKNi";
pub const VISION_ATTRIBUTE: &str = "AAMDNTaJtS2i4aWp59zEAk";

pub const ROOT_SPACE_ID: &str = "25omwWh6HYgeRQKCaSpVpa";

// Added by me
pub const AGGREGATION_DIRECTION: &str = "6zd9BPJNdUpcKenuK7LjCh";
