pub mod delete_many;
pub mod delete_one;
pub mod find_many;
pub mod find_one;
pub mod find_path;
pub mod insert_many;
pub mod insert_one;
pub mod models;
pub mod semantic_search;
pub mod utils;

pub use delete_one::DeleteOneQuery;
pub use find_many::FindManyQuery;
pub use find_one::FindOneQuery;
pub use find_path::FindPathQuery;
pub use insert_one::InsertOneQuery;
pub use models::{Entity, EntityNode, EntityNodeRef, SystemProperties};
pub use semantic_search::SemanticSearchQuery;
pub use utils::{EntityFilter, EntityRelationFilter, TypesFilter};

use crate::block::BlockMetadata;

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j,
        block,
        entity_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

/// Creates a query to find a single entity by its ID if it exists. Supports optional
/// filtering by space ID and version.
/// ```rust
/// use grc20_core::mapping::entity;
///
/// // Get current entity
/// let maybe_entity = entity::find_one::<EntityNode>(&neo4j, "entity_id")
///    .send()
///    .await?;
///
/// // Get entity in a specific space and version
/// let maybe_entity = entity::find_one::<EntityNode>(&neo4j, "entity_id")
///     .space_id("space_id")
///     .space_version("space_version")
///     .send()
///     .await?;
/// ```
pub fn find_one<T>(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FindOneQuery<T> {
    FindOneQuery::new(neo4j, id.into())
}

/// Creates a query to find multiple entities. Supports filtering by relations and
/// properties as well as ordering and pagination. See [`EntityFilter`] for more details
/// on filtering options.
///
/// ```rust
/// use grc20_core::mapping::entity;
/// use grc20_core::mapping::query_utils::order_by;
///
/// // Find entities with a specific attribute, order them by a property and
/// // return the first 10.
/// let entities = entity::find_many::<EntityNode>(&neo4j)
///     .filter(entity::EntityFilter::default()
///         // Filter by "SOME_ATTRIBUTE" attribute with value "some_value"
///         .attribute(AttributeFilter::new("SOME_ATTRIBUTE").value("some_value")))
///     .order_by(order_by::asc("some_property"))
///     .limit(10)
///     .send()
///     .await?;
///
/// // Find entities with a specific relation, in this case entities that have a
/// // `Parent` relation to an entity with ID "Alice".
/// let entities = entity::find_many::<EntityNode>(&neo4j)
///     .filter(entity::EntityFilter::default()
///         // Filter by relations
///         .relations(entity::EntityRelationFilter::default()
///             // Filter by `Parent` relation to entity with ID "Alice"
///             .relation_type("Parent".to_string())
///             .to_id("Alice".to_string())))
///     .send()
///     .await?;
///
/// // Find entities with a specific type (note: `TypesFilter` is a shorthand
/// // for `EntityRelationFilter`. It is converted to a relation filter internally).
/// let entities = entity::find_many::<EntityNode>(&neo4j)
///     .filter(entity::EntityFilter::default()
///         // Filter by `Types` relations pointing to `EntityType`
///         .relations(TypesFilter::default().r#type("EntityType".to_string())))
///     .send()
///     .await?;
/// ```
pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

/// Create a query to search for entities using semantic search based on a vector. The query
/// supports the same filtering options as `find_many`, allowing you to filter results by
/// attributes, relations, and other properties.
///
/// Important: The search uses *approximate* nearest neighbor search, which means that
/// the results with filtering applied after the search, which may lead to some results
/// that contain fewer than the desired quantity `limit`.
/// ```rust
/// use grc20_core::mapping::entity;
///
/// let search_vector = embedding::embed("my search query");
///
/// // Search for entities similar to the provided vector.
/// let results = entity::search::<EntityNode>(&neo4j, search_vector)
///     .send()
///
/// // Search for types (i.e.: entities that have `Types`` relation to `SchemaType``) of
/// // entities similar to the provided vector.
/// let results = entity::search::<EntityNode>(&neo4j, search_vector)
///     .filter(entity::EntityFilter::default()
///         // Filter by `Types` relations pointing to `SchemaType`
///         .relations(entity::TypesFilter::default().r#type(system_ids::SCHEMA_TYPE)))
///     .send()
///     .await?;
/// ```
pub fn search<T>(neo4j: &neo4rs::Graph, vector: Vec<f64>) -> SemanticSearchQuery<T> {
    SemanticSearchQuery::new(neo4j, vector)
}

// TODO: add docs for use via GraphQL
pub fn find_path<T>(neo4j: &neo4rs::Graph, id1: String, id2: String) -> FindPathQuery<T> {
    FindPathQuery::new(neo4j, id1, id2)
}

pub fn insert_one<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity: T,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertOneQuery<T> {
    InsertOneQuery::new(
        neo4j.clone(),
        block.clone(),
        entity,
        space_id.into(),
        space_version.into(),
    )
}
