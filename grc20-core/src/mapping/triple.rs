use std::collections::HashMap;

use futures::{Stream, TryStreamExt};
use neo4rs::{BoltMap, BoltType};
use serde::Deserialize;

use crate::{
    block::BlockMetadata,
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{query_utils::query_builder::Subquery, EFFECTIVE_SEARCH_RATIO},
    pb,
};

use super::{
    aggregation::AggregationDirection,
    query_utils::{
        query_builder::{MatchQuery, QueryBuilder},
        PropFilter, Query, QueryStream, VersionFilter,
    },
    Pluralism, Value,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Triple {
    pub entity: String,

    #[serde(alias = "id")]
    pub attribute: String,

    #[serde(flatten)]
    pub value: Value,

    pub embedding: Option<Vec<f64>>,
}

impl Triple {
    pub fn new(
        entity: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            entity: entity.into(),
            attribute: attribute.into(),
            value: value.into(),
            embedding: None,
        }
    }

    pub fn with_embedding(
        entity: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<Value>,
        embedding: Vec<f64>,
    ) -> Self {
        Self {
            entity: entity.into(),
            attribute: attribute.into(),
            value: value.into(),
            embedding: Some(embedding),
        }
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version.into(), self)
    }
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    attribute_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j,
        block,
        attribute_id.into(),
        entity_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

pub fn delete_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn insert_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
    triple: Triple,
) -> InsertOneQuery {
    InsertOneQuery::new(neo4j, block, space_id.into(), space_version.into(), triple)
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    attribute_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneQuery {
    FindOneQuery::new(
        neo4j,
        attribute_id.into(),
        entity_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

pub fn search(neo4j: &neo4rs::Graph, vector: Vec<f64>) -> SemanticSearchQuery {
    SemanticSearchQuery::new(neo4j, vector)
}

impl TryFrom<pb::ipfs::Triple> for Triple {
    type Error = String;

    fn try_from(triple: pb::ipfs::Triple) -> Result<Self, Self::Error> {
        if let Some(value) = triple.value {
            Ok(Triple {
                entity: triple.entity,
                attribute: triple.attribute,
                value: value.try_into()?,
                embedding: None,
            })
        } else {
            Err("Triple value is required".to_string())
        }
    }
}

impl TryFrom<(pb::ipfs::Triple, Vec<f64>)> for Triple {
    type Error = String;

    fn try_from(triple_and_embedding: (pb::ipfs::Triple, Vec<f64>)) -> Result<Self, Self::Error> {
        let (triple, embedding) = triple_and_embedding;
        if let Some(value) = triple.value {
            Ok(Triple {
                entity: triple.entity,
                attribute: triple.attribute,
                value: value.try_into()?,
                embedding: Some(embedding),
            })
        } else {
            Err("Triple value is required".to_string())
        }
    }
}

impl From<Triple> for BoltType {
    fn from(triple: Triple) -> Self {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "attr_labels".into(),
            },
            if ids::indexed(&triple.attribute) {
                vec!["Attribute", "Indexed"].into()
            } else {
                "Attribute".into()
            },
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "entity".into(),
            },
            triple.entity.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "attribute".into(),
            },
            triple.attribute.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            triple.value.into(),
        );

        if let Some(embedding) = triple.embedding {
            triple_bolt_map.insert(
                neo4rs::BoltString {
                    value: "embedding".into(),
                },
                embedding.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap {
            value: triple_bolt_map,
        })
    }
}

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triple: Triple,
}

impl InsertOneQuery {
    pub(crate) fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
        triple: Triple,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triple,
        }
    }
}

impl Query<()> for InsertOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e:Entity {{id: $triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            CALL (e) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:$($triple.attr_labels) {{id: $triple.attribute}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e) {{
                MERGE (e) -[r:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:$($triple.attr_labels) {{id: $triple.attribute}})
                SET m += $triple.value
                SET m.embedding = $triple.embedding
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triple", self.triple)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct InsertManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triples: Vec<Triple>,
}

impl InsertManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triples: vec![],
        }
    }

    pub fn triple(mut self, triple: Triple) -> Self {
        self.triples.push(triple);
        self
    }

    pub fn triple_mut(&mut self, triple: Triple) {
        self.triples.push(triple);
    }

    pub fn triples(mut self, triples: impl IntoIterator<Item = Triple>) -> Self {
        self.triples.extend(triples);
        self
    }

    pub fn triples_mut(&mut self, triples: impl IntoIterator<Item = Triple>) {
        self.triples.extend(triples);
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $triples as triple
            MERGE (e:Entity {{id: triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e, triple
            CALL (e, triple) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:$(triple.attr_labels) {{id: triple.attribute}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, triple) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:$(triple.attr_labels) {{id: triple.attribute}})
                SET m += triple.value
                SET m.embedding = triple.embedding
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triples", self.triples)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    version: VersionFilter,
    pluralism: Pluralism,
}

impl FindOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        version: Option<String>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id,
            entity_id,
            space_id,
            version: VersionFilter::new(version),
            pluralism: Pluralism::None,
        }
    }

    pub fn pluralism(mut self, pluralism_config: Pluralism) -> Self {
        self.pluralism = pluralism_config;
        self
    }

    fn subquery(&self) -> impl Subquery {
        match &self.pluralism {
            Pluralism::None => {
                QueryBuilder::default()
                    .subquery(
                        MatchQuery::new("(e:Entity {id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (attr:Attribute {id: $attribute_id})")
                            .r#where(self.version.subquery("r"))
                    )
                    .params("attribute_id", self.attribute_id.clone())
                    .params("entity_id", self.entity_id.clone())
                    .params("space_id", self.space_id.clone())
                    .r#return("attr{.*, entity: e.id} AS triple")
            }
            Pluralism::Direction(AggregationDirection::Up) => {
                QueryBuilder::default()
                    .subquery(format!(
                        r#"MATCH (start:Entity {{id: $space_id}}) (() <-[r:RELATION {{relation_type: "{}", space_id: "{}"}}]- (s:Entity)){{,}}"#,
                        indexer_ids::PARENT_SPACE,
                        indexer_ids::INDEXER_SPACE_ID,
                    ))
                    .subquery("WHERE size(s) = size(COLLECT { WITH s UNWIND s AS _ RETURN DISTINCT _ })")
                    .subquery("WITH COLLECT({space_id: LAST([start] + s).id, depth: SIZE(s)}) AS subspaces")
                    .subquery("UNWIND subspaces AS subspace")
                    .subquery(r#"MATCH (e:Entity {id: $entity_id}) -[r_attr:ATTRIBUTE {space_id: subspace.space_id}]-> (attr:Attribute {id: $attribute_id})"#)
                    .subquery(self.version.subquery("r_attr"))
                    .subquery("ORDER BY subspace.depth")
                    .limit(1)
                    .params("attribute_id", self.attribute_id.clone())
                    .params("entity_id", self.entity_id.clone())
                    .params("space_id", self.space_id.clone())
                    .r#return("attr{.*, entity: e.id} AS triple")
            }
            Pluralism::Direction(AggregationDirection::Down) => {
                QueryBuilder::default()
                    .subquery(format!(
                        r#"MATCH (start:Entity {{id: $space_id}}) (() -[r:RELATION {{relation_type: "{}", space_id: "{}"}}]-> (s:Entity)){{,}}"#,
                        indexer_ids::PARENT_SPACE,
                        indexer_ids::INDEXER_SPACE_ID,
                    ))
                    .subquery("WHERE size(s) = size(COLLECT { WITH s UNWIND s AS _ RETURN DISTINCT _ })")
                    .subquery("WITH COLLECT({space_id: LAST([start] + s).id, depth: SIZE(s)}) AS parent_spaces")
                    .subquery("UNWIND parent_spaces AS parent_space")
                    .subquery(r#"MATCH (e:Entity {id: $entity_id}) -[r_attr:ATTRIBUTE {space_id: parent_space.space_id}]-> (attr:Attribute {id: $attribute_id})"#)
                    .subquery(self.version.subquery("r_attr"))
                    .subquery("ORDER BY parent_space.depth")
                    .limit(1)
                    .params("attribute_id", self.attribute_id.clone())
                    .params("entity_id", self.entity_id.clone())
                    .params("space_id", self.space_id.clone())
                    .r#return("attr{.*, entity: e.id} AS triple")
            }
            Pluralism::Direction(AggregationDirection::Bidirectional) => {
                tracing::warn!("Bidirectional aggregation direction is not implemented yet! Defaulting to None.");
                QueryBuilder::default()
                    .subquery(
                        MatchQuery::new("(e:Entity {id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (attr:Attribute {id: $attribute_id})")
                            .r#where(self.version.subquery("r"))
                    )
                    .params("attribute_id", self.attribute_id.clone())
                    .params("entity_id", self.entity_id.clone())
                    .params("space_id", self.space_id.clone())
                    .r#return("attr{.*, entity: e.id} AS triple")
            }
            Pluralism::Hierarchy(spaces) => {
                QueryBuilder::default()
                    .subquery("UNWIND $spaces AS space")
                    .subquery(r#"MATCH (e:Entity {id: $entity_id}) -[r_attr:ATTRIBUTE {space_id: space.space_id}]-> (attr:Attribute {id: $attribute_id})"#)
                    .subquery(self.version.subquery("r_attr"))
                    .subquery("ORDER BY space.depth")
                    .limit(1)
                    .params("attribute_id", self.attribute_id.clone())
                    .params("entity_id", self.entity_id.clone())
                    .params("spaces", spaces.clone())
                    .r#return("attr{.*, entity: e.id} AS triple")
            }
        }
    }
}

impl Query<Option<Triple>> for FindOneQuery {
    async fn send(self) -> Result<Option<Triple>, DatabaseError> {
        let query = self.subquery();

        if cfg!(debug_assertions) || cfg!(test) {
            println!("triple::FindOneQuery:\n{}", query.compile());
        }

        self.neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| {
                // NOTE: When returning a projection, you can deserialize directly to
                // the struct without an intermediate "RowResult" struct since neo4j
                // will not return the data as a "Node" but instead as raw JSON.
                // let row = row.to::<RowResult>()?;
                // Result::<_, DatabaseError>::Ok(row.triple)
                row.to::<Triple>().map_err(DatabaseError::from)
            })
            .transpose()
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    attribute_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    entity_id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    limit: usize,
    space_version: VersionFilter,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id: None,
            value: None,
            value_type: None,
            entity_id: None,
            space_id: None,
            limit: 100,
            space_version: VersionFilter::default(),
        }
    }

    pub fn attribute_id(mut self, filter: PropFilter<String>) -> Self {
        self.attribute_id = Some(filter);
        self
    }

    pub fn value(mut self, filter: PropFilter<String>) -> Self {
        self.value = Some(filter);
        self
    }

    pub fn value_type(mut self, filter: PropFilter<String>) -> Self {
        self.value_type = Some(filter);
        self
    }

    pub fn entity_id(mut self, filter: PropFilter<String>) -> Self {
        self.entity_id = Some(filter);
        self
    }

    pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
        self.space_id = Some(filter);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn space_version(mut self, space_version: impl Into<String>) -> Self {
        self.space_version.version_mut(space_version.into());
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(
                MatchQuery::new("(e:Entity) -[r:ATTRIBUTE]-> (n:Attribute)")
                    .where_opt(self.entity_id.as_ref().map(|s| s.subquery("e", "id", None)))
                    .where_opt(
                        self.attribute_id
                            .as_ref()
                            .map(|s| s.subquery("n", "id", None)),
                    )
                    .where_opt(self.value.as_ref().map(|s| s.subquery("n", "value", None)))
                    .where_opt(
                        self.value_type
                            .as_ref()
                            .map(|s| s.subquery("n", "value_type", None)),
                    )
                    .where_opt(
                        self.space_id
                            .as_ref()
                            .map(|s| s.subquery("r", "space_id", None)),
                    )
                    .r#where(self.space_version.subquery("r")),
            )
            .limit(self.limit)
            .subquery("RETURN n{.*, entity: e.id}")
    }
}

impl QueryStream<Triple> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Triple, DatabaseError>>, DatabaseError> {
        let query = self.subquery();

        if cfg!(debug_assertions) || cfg!(test) {
            println!("triple::FindManyQuery:\n{}", query.compile());
        }

        Ok(self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<Triple>()
            .map_err(DatabaseError::from))
    }
}

pub struct SemanticSearchQuery {
    neo4j: neo4rs::Graph,
    vector: Vec<f64>,
    // space_id: Option<PropFilter<String>>,
    // space_version: VersionFilter,
    limit: usize,
    skip: Option<usize>,
}

impl SemanticSearchQuery {
    pub fn new(neo4j: &neo4rs::Graph, vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            vector,
            // space_id: None,
            // space_version: VersionFilter::default(),
            limit: 100,
            skip: None,
        }
    }

    // pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
    //     self.space_id = Some(filter);
    //     self
    // }

    // pub fn space_version(mut self, space_version: impl Into<String>) -> Self {
    //     self.space_version.version_mut(space_version.into());
    //     self
    // }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn limit_opt(mut self, limit: Option<usize>) -> Self {
        if let Some(limit) = limit {
            self.limit = limit;
        }
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn skip_opt(mut self, skip: Option<usize>) -> Self {
        self.skip = skip;
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SemanticSearchResult {
    #[serde(flatten)]
    pub triple: Triple,
    pub score: f64,
    pub space_id: String,
    pub space_version: String,
}

impl QueryStream<SemanticSearchResult> for SemanticSearchQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<SemanticSearchResult, DatabaseError>>, DatabaseError>
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            CALL db.index.vector.queryNodes('vector_index', $effective_search_ratio, $vector)
            YIELD node AS n, score AS score
            ORDER BY score DESC
            LIMIT $limit
            MATCH (e:Entity) -[r:ATTRIBUTE]-> (n)
            RETURN n{{.*, entity: e.id, space_version: r.min_version, space_id: r.space_id, score: score}}
            "#
        );
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH (e:Entity) -[r:ATTRIBUTE]-> (a:Attribute:Indexed)
        //     WHERE r.max_version IS null
        //     AND a.embedding IS NOT NULL
        //     WITH e, a, r, vector.similarity.cosine(a.embedding, $vector) AS score
        //     ORDER BY score DESC
        //     WHERE score IS NOT null
        //     LIMIT $limit
        //     RETURN a{{.*, entity: e.id, space_version: r.min_version, space_id: r.space_id, score: score}}
        //     "#,
        // );

        let query = neo4rs::query(QUERY)
            .param("vector", self.vector)
            .param("limit", self.limit as i64)
            .param("effective_search_ratio", EFFECTIVE_SEARCH_RATIO);

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<SemanticSearchResult>()
            .map_err(DatabaseError::from))
    }
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: String,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            attribute_id,
            entity_id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e:Entity {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: $attribute_id}})
            WHERE r.max_version IS null
            SET r.max_version = $space_version
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("attribute_id", self.attribute_id)
            .param("entity_id", self.entity_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct DeleteManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    triples: Vec<(String, String)>,
}

impl DeleteManyQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triples: vec![],
        }
    }

    pub fn triple(mut self, entity_id: impl Into<String>, attribute_id: impl Into<String>) -> Self {
        self.triples.push((entity_id.into(), attribute_id.into()));
        self
    }

    pub fn triple_mut(&mut self, entity_id: impl Into<String>, attribute_id: impl Into<String>) {
        self.triples.push((entity_id.into(), attribute_id.into()));
    }

    pub fn triples(mut self, triples: impl IntoIterator<Item = (String, String)>) -> Self {
        self.triples.extend(triples);
        self
    }

    pub fn triples_mut(&mut self, triples: impl IntoIterator<Item = (String, String)>) {
        self.triples.extend(triples);
    }
}

impl Query<()> for DeleteManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $triples as triple
            MATCH (e:Entity {{id: triple.entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: triple.attribute_id}})
            WHERE r.max_version IS null
            SET r.max_version = $space_version
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param(
                "triples",
                self.triples
                    .into_iter()
                    .map(|(entity_id, attribute_id)| {
                        BoltType::Map(BoltMap {
                            value: HashMap::from([
                                (
                                    neo4rs::BoltString {
                                        value: "entity_id".into(),
                                    },
                                    entity_id.into(),
                                ),
                                (
                                    neo4rs::BoltString {
                                        value: "attribute_id".into(),
                                    },
                                    attribute_id.into(),
                                ),
                            ]),
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        neo4j.run(
            neo4rs::query(r#"CREATE (:Entity {id: "abc"}) -[:ATTRIBUTE {space_id: "ROOT", min_version: 0}]-> (:Attribute {id: "name", value: "Alice", value_type: "TEXT"})"#)
        )
        .await
        .expect("Failed to create test data");

        let triple = Triple::new("abc", "name", "Alice");

        let found_triple = find_one(&neo4j, "name", "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find triple")
            .expect("Triple not found");

        assert_eq!(triple, found_triple);
    }

    #[tokio::test]
    async fn test_insert_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple = Triple::new("abc", "name", "Alice");

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let found_triple = find_one(&neo4j, "name", "abc", "ROOT", Some("0".into()))
            .send()
            .await
            .expect("Failed to find triple")
            .expect("Triple not found");

        assert_eq!(triple, found_triple);
    }

    #[tokio::test]
    pub async fn test_insert_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple = Triple::new("abc", "name", "Alice");
        let other_triple = Triple::new("def", "name", "Bob");

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![triple.clone(), other_triple])
            .send()
            .await
            .expect("Failed to insert triples");

        let found_triples = FindManyQuery::new(&neo4j)
            .attribute_id(PropFilter::default().value("name"))
            .value(PropFilter::default().value("Alice"))
            .value_type(PropFilter::default().value("TEXT"))
            .entity_id(PropFilter::default().value("abc"))
            .space_id(PropFilter::default().value("ROOT"))
            .space_version("0")
            .send()
            .await
            .expect("Failed to find triples")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(vec![triple], found_triples);
    }

    #[tokio::test]
    pub async fn test_insert_find_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple = Triple::new("abc", "name", "Alice");

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let other_triple = Triple::new("def", "name", "Bob");

        other_triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let found_triples = FindManyQuery::new(&neo4j)
            .attribute_id(PropFilter::default().value("name"))
            .value(PropFilter::default().value("Alice"))
            .value_type(PropFilter::default().value("TEXT"))
            .entity_id(PropFilter::default().value("abc"))
            .space_id(PropFilter::default().value("ROOT"))
            .space_version("0")
            .send()
            .await
            .expect("Failed to find triples")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(vec![triple], found_triples);
    }

    #[tokio::test]
    async fn test_versioning() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple_v1 = Triple::new("abc", "name", "Alice");

        triple_v1
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_v2 = Triple::new("abc", "name", "NotAlice");

        triple_v2
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "1")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_latest = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, triple_latest);
        assert_eq!(triple_latest.value.value, "NotAlice".to_string());

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v1, found_triple_v1);
        assert_eq!(found_triple_v1.value.value, "Alice".to_string());
    }

    #[tokio::test]
    async fn test_update_no_versioning() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple_v1 = Triple::new("abc", "name", "Alice");

        triple_v1
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_v2 = Triple::new("abc", "name", "NotAlice");

        triple_v2
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let triple_latest = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, triple_latest);
        assert_eq!(triple_latest.value.value, "NotAlice".to_string());

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple_v2, found_triple_v1);
        assert_eq!(found_triple_v1.value.value, "NotAlice".to_string());
    }

    #[tokio::test]
    async fn test_delete() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple = Triple::new("abc", "name", "Alice");

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        delete_one(
            &neo4j,
            &BlockMetadata::default(),
            "name",
            "abc",
            "ROOT",
            "1",
        )
        .send()
        .await
        .expect("Failed to delete triple");

        let found_triple = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            None,
        )
        .send()
        .await
        .expect("Failed to find triple");

        assert_eq!(None, found_triple);

        let found_triple_v1 = find_one(
            &neo4j,
            "name".to_string(),
            "abc".to_string(),
            "ROOT".to_string(),
            Some("0".into()),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple, found_triple_v1);
    }
}
