use crate::mapping;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Neo4j error: {0}")]
    Neo4jError(#[from] neo4rs::Error),
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] neo4rs::DeError),
    #[error("Serialization Error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("SetTripleError: {0}")]
    SetTripleError(#[from] mapping::entity::SetTripleError),
}
