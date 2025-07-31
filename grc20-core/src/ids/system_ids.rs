use uuid::{uuid, Uuid};

// ================================================================
// Ported from https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/core/ids/system.ts
// ================================================================
pub const PROPERTY_TYPE: Uuid = uuid!("808a04ce-b21c-4d88-8ad1-2e240613e5ca");
pub const SCHEMA_TYPE: Uuid = uuid!("e7d737c5-3676-4c60-9fa1-6aa64a8c90ad");
pub const PROPERTIES: Uuid = uuid!("01412f83-8189-4ab1-8365-65c7fd358cc1");

pub const NAME_PROPERTY: Uuid = uuid!("a126ca53-0c8e-48d5-b888-82c734c38935");
pub const DESCRIPTION_PROPERTY: Uuid = uuid!("9b1f76ff-9711-404c-861e-59dc3fa7d037");
pub const COVER_ATTRIBUTE: Uuid = uuid!("34f53507-2e6b-42c5-a844-43981a77cfa2");
pub const TYPES_ATTRIBUTE: Uuid = uuid!("8f151ba4-de20-4e3c-9cb4-99ddf96f48f1");
pub const BLOCKS: Uuid = uuid!("beaba5cb-a677-41a8-b353-77030613fc70");

// Value types
pub const VALUE_TYPE_ATTRIBUTE: Uuid = uuid!("ee26ef23-f7f1-4eb6-b742-3b0fa38c1fd8");
pub const CHECKBOX: Uuid = uuid!("7aa4792e-eacd-4186-8272-fa7fc18298ac");
pub const TIME: Uuid = uuid!("167664f6-68f8-40e1-976b-20bd16ed8d47");
pub const TEXT: Uuid = uuid!("9edb6fcc-e454-4aa5-8611-39d7f024c010");
pub const URL: Uuid = uuid!("283127c9-6142-4684-92ed-90b0ebc7f29a");
pub const NUMBER: Uuid = uuid!("9b597aae-c31c-46c8-8565-a370da0c2a65");
pub const POINT: Uuid = uuid!("df250d17-e364-413d-9779-2ddaae841e34");
pub const IMAGE: Uuid = uuid!("f3f790c4-c74e-4d23-a0a9-1e8ef84e30d9");

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
pub const RELATION_SCHEMA_TYPE: Uuid = uuid!("4b6d9fc1-fbfe-474c-861c-83398e1b50d9");

pub const SPACE_TYPE: Uuid = uuid!("362c1dbd-dc64-44bb-a3c4-652f38a642d7");

/// Defines the relation value types for a relation. e.g., a Persons
/// attribute must only contain relations where the to entity is type
/// Person
pub const RELATION_VALUE_RELATIONSHIP_TYPE: Uuid = uuid!("9eea393f-17dd-4971-a62e-a603e8bfec20");

pub const IMAGE_TYPE: Uuid = uuid!("ba4e4146-0010-499d-a0a3-caaa7f579d0e");
pub const IMAGE_FILE_TYPE_ATTRIBUTE: Uuid = uuid!("515f346f-e0fb-40c7-8ea9-5339787eecc1");
pub const IMAGE_HEIGHT_ATTRIBUTE: Uuid = uuid!("7f6ad043-3e21-4257-a6d4-8bdad36b1d84");
pub const IMAGE_URL_ATTRIBUTE: Uuid = uuid!("8a743832-c094-4a62-b665-0c3cc2f9c7bc");
pub const IMAGE_WIDTH_ATTRIBUTE: Uuid = uuid!("f7b33e08-b76d-4190-aada-cadaa9f561e1");

// Data blocks
// @TODO: data block instead of TABLE_BLOCK
pub const DATA_BLOCK: Uuid = uuid!("b8803a86-65de-412b-bb35-7e0c84adf473");
pub const DATA_SOURCE_ATTRIBUTE: Uuid = uuid!("8ac1c4bf-453b-44b7-9eda-5d1b527e5ea3");
pub const ALL_OF_GEO_DATA_SOURCE: Uuid = uuid!("f9adb874-52b9-4982-8f55-aa40792751e3");
pub const COLLECTION_DATA_SOURCE: Uuid = uuid!("1295037a-5d9c-4d09-b27c-5502654b9177");
pub const QUERY_DATA_SOURCE: Uuid = uuid!("3b069b04-adbe-4728-917d-1283fd4ac27e");

/**
 * Defines the filters applied to a data block. It applies whether
 * the data block is a COLLECTION or a QUERY.
 *
 * See the Filters spec to see the Filter format.
 */
pub const FILTER: Uuid = uuid!("14a46854-bfd1-4b18-8215-2785c2dab9f3");
pub const SPACE_FILTER: Uuid = uuid!("8f6df521-24fa-4576-887e-0442973e2f33");
pub const ENTITY_FILTER: Uuid = uuid!("d6b8aa86-2c73-41ca-bddb-63aa0732174c");

/**
 * Defines that a relation type is COLLECTION_ITEM. This is used by
 * data blocks with the [COLLECTION_DATA_SOURCE] to denote that a relation
 * should be consumed as a collection item in the block.
 *
 * Collections enable data blocks to render arbitrarily selected entities
 * rather than the results of a query. A collection item defines what
 * these selected entities are.
 */
pub const COLLECTION_ITEM_RELATION_TYPE: Uuid = uuid!("a99f9ce1-2ffa-4dac-8c61-f6310d46064a");

/**
 * Defines the view type for a data block, either a [TABLE_VIEW],
 * [GALLERY_VIEW] or a [LIST_VIEW]. Your own application
 * can define their own views with their own IDs.
 *
 * This view data lives on the relation pointing to the block. This enables
 * different consumers of the data block to define their own views.
 */
pub const VIEW_ATTRIBUTE: Uuid = uuid!("1907fd1c-8111-4a3c-a378-b1f353425b65");

pub const VIEW_TYPE: Uuid = uuid!("20a21dc2-7371-482f-a120-7b147f1dc319");
pub const TABLE_VIEW: Uuid = uuid!("cba271ce-f7c1-4033-9047-614d174c69f1");
pub const LIST_VIEW: Uuid = uuid!("7d497dba-09c2-49b8-968f-716bcf520473");
pub const GALLERY_VIEW: Uuid = uuid!("ccb70fc9-17f0-4a54-b86e-3b4d20cc7130");

/**
 * Defines the columns to show on a data block when in [TABLE_VIEW].
 */
pub const SHOWN_COLUMNS: Uuid = uuid!("4221fb36-dcab-4c68-b150-701aaba6c8e0");

/**
 * Defines the type of data source for a data block, either a
 * {@link COLLECTION_DATA_SOURCE}, an {@link ALL_OF_GEO_DATA_SOURCE} or
 * a {@link QUERY_DATA_SOURCE}
 */
pub const DATA_SOURCE_TYPE_RELATION_TYPE: Uuid = uuid!("1f69cc98-80d4-44ab-ad49-3df6a7b15ee4");

pub const IMAGE_BLOCK: Uuid = uuid!("e3817941-7409-4df1-b519-1f3f1a0721e8");
pub const TEXT_BLOCK: Uuid = uuid!("76474f2f-0089-4e77-a041-0b39fb17d0bf");

pub const MARKDOWN_CONTENT: Uuid = uuid!("e3e363d1-dd29-4ccb-8e6f-f3b76d99bc33");

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
pub const RELATION_TYPE: Uuid = uuid!("c167ef23-fb2a-4044-9ed9-45123ce7d2a9");
pub const RELATION_FROM_ATTRIBUTE: Uuid = uuid!("c43b537b-cff7-4271-8822-717fdf2c9c01");
pub const RELATION_TO_ATTRIBUTE: Uuid = uuid!("c1f4cb6f-ece4-4c3c-a447-ab005b756972");
pub const RELATION_TYPE_ATTRIBUTE: Uuid = uuid!("14611456-b466-4cab-920d-2245f59ce828");

/*
 * Relations can be ordered using fractional indexing. By default we
 * add an index to every relation so that ordering can be added to
 * any relation at any point without having to add indexes to relations
 * post-creation.
 */
pub const RELATION_INDEX: Uuid = uuid!("ede47e69-30b0-4499-8ea4-aafbda449609");

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
pub const VERIFIED_SOURCE_ATTRIBUTE: Uuid = uuid!("265e869b-3a27-40bd-b652-a7a77b0df24f");
pub const SOURCE_SPACE_ATTRIBUTE: Uuid = uuid!("81891dec-cb6c-427e-aa1f-b917292ec2dc");

// Core types
pub const ACADEMIC_FIELD_TYPE: Uuid = uuid!("70fbf179-3da8-4bb0-80b8-60a7bc2f61fb");
pub const COMPANY_TYPE: Uuid = uuid!("e059a29e-6f6b-437b-bc15-c7983d078c0d");
pub const DAO_TYPE: Uuid = uuid!("872cb6f6-926d-4bb3-9d63-ccede27232b8");
pub const GOVERNMENT_ORG_TYPE: Uuid = uuid!("a87e0291-ec36-4572-8af2-1301b3099e97");
pub const INDUSTRY_TYPE: Uuid = uuid!("fc512a40-8b55-44dc-85b8-5aae88b51fae");
pub const INTEREST_GROUP_TYPE: Uuid = uuid!("2c765cae-c1b6-4cc3-a65d-693d0a67eaeb");
pub const NONPROFIT_TYPE: Uuid = uuid!("c7a192a3-3909-4572-a848-a56b64dc4636");
pub const POST_TYPE: Uuid = uuid!("f3d44614-86b7-4d25-83d8-9709c9d84f65");
pub const PROJECT_TYPE: Uuid = uuid!("484a18c5-030a-499c-b0f2-ef588ff16d50");
pub const PROTOCOL_TYPE: Uuid = uuid!("c38c4198-10c2-4cf2-9dd5-8f194033fc31");
pub const REGION_TYPE: Uuid = uuid!("c188844a-7224-42ab-b476-2991c9c913f1");
pub const ROOT_SPACE_TYPE: Uuid = uuid!("06053fcf-6443-4dc6-80ca-8a3b173a6016");

// Templates
pub const TEMPLATE_ATTRIBUTE: Uuid = uuid!("cf37cd59-840c-4dac-a22b-9d9dde536ea7");
pub const PAGE_TYPE: Uuid = uuid!("480e3fc2-67f3-4993-85fb-acdf4ddeaa6b");

/**
 * Defines the page type for a template. e.g., an events page, a
 * finances page, a products page, etc.
 */
pub const PAGE_TYPE_ATTRIBUTE: Uuid = uuid!("62dfabe5-282d-44a7-ba93-f2e80d20743d");

// These define the entity id to copy when creating an entity from
// a template.
pub const INDUSTRY_ABOUT_PAGE_TEMPLATE: Uuid = uuid!("b2fb69c7-0822-4999-b874-77e6f557a9ff");
pub const INDUSTRY_EVENTS_PAGE_TEMPLATE: Uuid = uuid!("ae0cf033-49c7-44d2-a5e9-d8a4b7140594");
pub const INDUSTRY_JOBS_PAGE_TEMPLATE: Uuid = uuid!("c56998f3-e275-4ba3-bac6-83cb61a1b430");
pub const INDUSTRY_NEWS_PAGE_TEMPLATE: Uuid = uuid!("0a43f7fa-c782-4ad3-92a8-02cfb5be5010");
pub const INDUSTRY_ONTOLOGY_PAGE_TEMPLATE: Uuid = uuid!("4eede710-07a1-41c4-b17d-61eae5dd79ce");
pub const INDUSTRY_OVERVIEW_PAGE_TEMPLATE: Uuid = uuid!("d81abf98-18d9-4259-b968-e538c60c841b");
pub const INDUSTRY_PEOPLE_PAGE_TEMPLATE: Uuid = uuid!("8c8b6e5e-20bb-41f5-98e0-7443bf23a3c2");
pub const INDUSTRY_PROJECTS_PAGE_TEMPLATE: Uuid = uuid!("ee1b1fe6-29ff-4f9f-a710-0fae36a72f8e");
pub const COMPANY_EVENTS_PAGE_TEMPLATE: Uuid = uuid!("19e4e0c0-8071-420d-a5b1-af1de1bda5fb");
pub const COMPANY_JOBS_PAGE_TEMPLATE: Uuid = uuid!("64b107d0-fc26-4077-ab93-a134ed3ee88a");
pub const COMPANY_OVERVIEW_PAGE_TEMPLATE: Uuid = uuid!("bedb6493-dc5b-47b4-b1a5-c68bfbbb2b43");
pub const COMPANY_POSTS_PAGE_TEMPLATE: Uuid = uuid!("4b2a529e-6806-442c-8018-e9e92ad6d8d8");
pub const COMPANY_PRODUCTS_PAGE_TEMPLATE: Uuid = uuid!("3260bc7e-4680-42e3-820b-9d9b44ca1250");
pub const COMPANY_SERVICES_PAGE_TEMPLATE: Uuid = uuid!("ad75c47d-cf3a-4880-8df5-228f6f673d7e");
pub const COMPANY_TEAM_PAGE_TEMPLATE: Uuid = uuid!("518f9006-1a62-4878-87b7-e694c28f2a1e");
pub const PERSON_OVERVIEW_PAGE_TEMPLATE: Uuid = uuid!("6bc6a6b0-b9eb-441d-b898-14f7a726877c");
pub const PERSON_POSTS_PAGE_TEMPLATE: Uuid = uuid!("41e504c2-e347-450b-bce3-f838bad0fa55");
pub const NONPROFIT_FINANCES_PAGE_TEMPLATE: Uuid = uuid!("79ce6410-1c8f-4e8d-ac71-08015e7ab111");
pub const NONPROFIT_ID_NUMBER_ATTRIBUTE: Uuid = uuid!("c1a8dd82-48b7-4cba-a22c-09441d479c26");
pub const NONPROFIT_OVERVIEW_PAGE_TEMPLATE: Uuid = uuid!("838361ab-f358-4044-9f13-1586a3266a2b");
pub const NONPROFIT_POSTS_PAGE_TEMPLATE: Uuid = uuid!("7a8cf365-2906-4786-ad6c-72a716518362");
pub const NONPROFIT_PROJECTS_PAGE_TEMPLATE: Uuid = uuid!("8fb6f2c4-f47b-42ad-b525-701b6db6c9ea");
pub const NONPROFIT_SERVICE_TYPE: Uuid = uuid!("7e05b4ea-09e4-41b2-816d-73709cbcf61a");
pub const NONPROFIT_TEAM_PAGE_TEMPLATE: Uuid = uuid!("92539817-7989-49da-2c89-6bf00f07e5da");
pub const ONTOLOGY_PAGE_TEMPLATE: Uuid = uuid!("8f90bd6d-8147-4eb8-b67f-d34fa2af1397");
pub const EDUCATION_PAGE_TEMPLATE: Uuid = uuid!("c4b7e33f-939a-4871-b9e2-2711b6d3f49f");
pub const ABOUT_PAGE_TEMPLATE: Uuid = uuid!("31af0a8a-6bc1-4981-b2ac-c45f2ba4d27c");

// Defines the type of the page being copied when creating an entity
// from a template.
pub const ABOUT_PAGE: Uuid = uuid!("f93d044e-61f6-42a8-92eb-ef37a377a535");
pub const EDUCATION_PAGE: Uuid = uuid!("bfdc5a8f-4f6a-4955-bfbd-7bea921d8a42");
pub const EVENTS_PAGE: Uuid = uuid!("92e64c6e-36ad-453b-9533-9e4be1033cdf");
pub const FINANCES_PAGE: Uuid = uuid!("c316cec6-ddaa-436c-9c52-43b4179db529");
pub const JOBS_PAGE: Uuid = uuid!("b4ac6985-5e8e-46d8-8f73-3ed8a918d33a");
pub const NEWS_PAGE: Uuid = uuid!("d172f6f6-93a8-4c28-9577-e9d43259863b");
pub const ONTOLOGY_PAGE: Uuid = uuid!("6e7215ec-10ca-4904-8895-2d89070a9c69");
pub const PEOPLE_PAGE: Uuid = uuid!("238bce0f-1420-4df7-85a3-088aded159fd");
pub const POSTS_PAGE: Uuid = uuid!("69a88b15-9a8f-4d56-930e-d7fc84accb14");
pub const PRODUCTS_PAGE: Uuid = uuid!("5f7474c7-115e-44e4-9608-af93edcaf491");
pub const PROJECTS_PAGE: Uuid = uuid!("1743389c-dafb-49a6-b667-9d8bc46f3f52");
pub const SERVICES_PAGE: Uuid = uuid!("0c06c8a0-563e-420b-b8e5-3b9c911ffa37");
pub const SPACES_PAGE: Uuid = uuid!("8afae81d-33b5-40f7-b89c-ea7d49b10f9f");
pub const TEAM_PAGE: Uuid = uuid!("55147448-8894-4ff7-a163-2f96f7afa7df");

pub const FINANCE_OVERVIEW_TYPE: Uuid = uuid!("2315fe0c-6a4d-4f99-bb19-b59c8e7c563a");
pub const FINANCE_SUMMMARY_TYPE: Uuid = uuid!("40c3c7e1-e066-43df-8eea-9900514e96ed");

// Identity
pub const ACCOUNT_TYPE: Uuid = uuid!("cb69723f-7456-471a-a8ad-3e93ddc3edfe");
pub const ACCOUNTS_ATTRIBUTE: Uuid = uuid!("e4047a77-0043-4ed4-a410-72139bff7f7e");
pub const ADDRESS_ATTRIBUTE: Uuid = uuid!("85cebdf1-d84f-4afd-993b-35f182096b59");
pub const NETWORK_ATTRIBUTE: Uuid = uuid!("a945fa95-d15e-42bc-b70a-43d3933048dd");
pub const PERSON_TYPE: Uuid = uuid!("7ed45f2b-c48b-419e-8e46-64d5ff680b0d");
pub const NETWORK_TYPE: Uuid = uuid!("fca08431-1aa1-40f2-8a4d-0743c2a59df7");

pub const GOALS_ATTRIBUTE: Uuid = uuid!("eddd99d6-2033-4651-b046-2cecd1bd6ca5");
pub const GOAL_TYPE: Uuid = uuid!("0fecaded-7c58-4a71-9a02-e1cb49800e27");
pub const MEMBERSHIP_CONTRACT_ADDRESS: Uuid = uuid!("62f5aa2f-34ca-47c0-bcfc-a937396676dd");
pub const MISSION_ATTRIBUTE: Uuid = uuid!("e4ed96e6-92cf-42c4-967b-ab2cef56f889");
pub const PLACEHOLDER_IMAGE: Uuid = uuid!("6c49012e-21fd-4b35-b976-60210ea0ae0f");
pub const PLACEHOLDER_TEXT: Uuid = uuid!("503e9e78-8669-4243-9777-af8fb75cdc56");
pub const TAB_TYPE: Uuid = uuid!("306a88ec-1960-4328-8cdb-77c23de2785a");
pub const ROLE_ATTRIBUTE: Uuid = uuid!("e4e366e9-d555-4b68-92bf-7358e824afd2");

// Do we still need these?
pub const DEFAULT_TYPE: Uuid = uuid!("36ea5723-0851-4012-945e-90d05f0e54e9");
pub const BROADER_CLAIMS_ATTRIBUTE: Uuid = uuid!("c682b9a5-82b3-4345-abc9-1c31cd09a253");
pub const CLAIMS_FROM_ATTRIBUTE: Uuid = uuid!("8ebf2fe7-270b-4a7e-806d-e481c8c058d0");
pub const DEFINITIONS_ATTRIBUTE: Uuid = uuid!("08abac45-2e19-4cb6-afaa-5e11a31eea99");
pub const EMAIL_ATTRIBUTE: Uuid = uuid!("0b63fdea-d04d-4985-8f18-f97995926e6e");
pub const FOREIGN_TYPES: Uuid = uuid!("c2a3dd99-bd57-4593-a801-882e8280b94c");
pub const NONPROFIT_CATEGORIES_ATTRIBUTE: Uuid = uuid!("29094591-c312-43c4-9c4a-0c2eb5063e2c");
pub const PHONE_NUMBER_ATTRIBUTE: Uuid = uuid!("1840e2d2-487f-42a0-9265-f7e7a4752a75");
pub const QUOTES_ATTRIBUTE: Uuid = uuid!("f7286c68-3412-4673-bd58-c25450ab53f9");
pub const REGION_ATTRIBUTE: Uuid = uuid!("5b33846f-4742-49f9-86e5-7009978019a7");
pub const RELATED_TOPICS_ATTRIBUTE: Uuid = uuid!("cc42b16e-956e-4451-a304-5f27e1c3ed11");
pub const RELEVANT_QUESTIONS_ATTRIBUTE: Uuid = uuid!("b897ab9e-6409-473c-9071-0a34f3da537b");
pub const SPEAKERS_ATTRIBUTE: Uuid = uuid!("4725dae3-1163-4c87-8e87-dcf159e385a6");
pub const STREET_ADDRESS_ATTRIBUTE: Uuid = uuid!("3ed2eb81-20b1-488c-90f5-c84f58535083");
pub const SUBCLAIMS_ATTRIBUTE: Uuid = uuid!("09cf4adf-d8e8-4b2e-8cd1-48a3d9a24ac2");
pub const VALUES_ATTRIBUTE: Uuid = uuid!("15183b43-6f73-46b2-812d-edbe1de78343");
pub const VISION_ATTRIBUTE: Uuid = uuid!("4a306614-0c75-416c-a347-d36b3cc9c031");

pub const ROOT_SPACE_ID: Uuid = uuid!("08c4f093-7858-4b7c-9b94-b82e448abcff");

// Added by me
pub const AGGREGATION_DIRECTION: Uuid = uuid!("30895c62-de91-421c-87fc-897bd824d9a6");
