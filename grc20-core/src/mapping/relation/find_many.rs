use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    error::DatabaseError,
    mapping::{
        query_utils::{
            query_builder::{MatchQuery, QueryBuilder, Subquery},
            VersionFilter,
        },
        AttributeNode, EntityNode, EntityNodeRef, FromAttributes, PropFilter, QueryStream,
    },
};

use super::{utils::RelationFilter, FindManyToQuery, Relation, RelationEdge};

pub struct FindManyQuery<T> {
    neo4j: neo4rs::Graph,
    filter: RelationFilter,

    space_id: Option<PropFilter<String>>,
    version: VersionFilter,

    limit: usize,
    skip: Option<usize>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            filter: RelationFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn select_to<U>(self) -> FindManyToQuery<U> {
        FindManyToQuery {
            neo4j: self.neo4j,
            filter: self.filter,
            space_id: self.space_id,
            version: self.version,
            limit: self.limit,
            skip: self.skip,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn filter(mut self, filter: RelationFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn version(mut self, space_version: Option<String>) -> Self {
        if let Some(space_version) = space_version {
            self.version.version_mut(space_version);
        }
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    fn relation_edge_subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(
                MatchQuery::new("(from:Entity) -[r:RELATION]-> (to:Entity)")
                    // Apply edge id filter
                    .where_opt(
                        self.filter
                            .id
                            .as_ref()
                            .map(|id| id.subquery("r", "id", None)),
                    )
                    // Apply from.id filter
                    .where_opt(
                        self.filter
                            .from_
                            .as_ref()
                            .and_then(|from_filter| from_filter.id.clone())
                            .map(|from_id| from_id.subquery("from", "id", None)),
                    )
                    // Apply to.id filter
                    .where_opt(
                        self.filter
                            .to_
                            .as_ref()
                            .and_then(|to_filter| to_filter.id.clone())
                            .map(|to_id| to_id.subquery("to", "id", None)),
                    )
                    // Apply edge relation_type filter
                    .where_opt(
                        self.filter
                            .relation_type
                            .as_ref()
                            .and_then(|rt| rt.id.clone())
                            .map(|rt_id| rt_id.subquery("r", "relation_type", None)),
                    )
                    // Apply edge space_id filter
                    .where_opt(
                        self.space_id
                            .as_ref()
                            .map(|space_id| space_id.subquery("r", "space_id", None)),
                    )
                    // Apply edge version filter
                    .r#where(self.version.subquery("r")),
            )
            .subquery(self.filter.subquery("r", "from", "to"))
            .limit(self.limit)
            .subquery("ORDER BY r.index")
            .skip_opt(self.skip)
    }

    fn full_relation_subquery(&self) -> QueryBuilder {
        self.relation_edge_subquery().with(
            vec!["r".to_string(), "from".to_string(), "to".to_string()],
            {
                QueryBuilder::default()
                    .subquery(MatchQuery::new("(r_e:Entity {id: r.id})"))
                    .subquery(
                        MatchQuery::new_optional("(r_e) -[r_attr:ATTRIBUTE]-> (n:Attribute)")
                            .where_opt(
                                self.space_id
                                    .as_ref()
                                    .map(|space_id| space_id.subquery("r_attr", "space_id", None)),
                            )
                            .r#where(self.version.subquery("r_attr")),
                    )
            },
        )
    }
}

impl QueryStream<RelationEdge<EntityNodeRef>> for FindManyQuery<RelationEdge<EntityNodeRef>> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<RelationEdge<EntityNodeRef>, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();
        let query = self
            .relation_edge_subquery()
            .r#return("r{.*, from: from.id, to: to.id} as r");

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "relation_node::FindManyQuery::<RelationEdge<EntityNodeRef>>:\n{}",
                query.compile()
            );
        };

        Ok(neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RelationEdge<EntityNodeRef>>()
            .map_err(DatabaseError::from))
    }
}

impl QueryStream<RelationEdge<EntityNode>> for FindManyQuery<RelationEdge<EntityNode>> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<RelationEdge<EntityNode>, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();
        let query = self
            .relation_edge_subquery()
            .r#return("r{.*, from: from, to: to} as r");

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "relation_node::FindManyQuery::<RelationEdge<EntityNode>>:\n{}",
                query.compile()
            );
        };

        Ok(neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RelationEdge<EntityNode>>()
            .map_err(DatabaseError::from))
    }
}

impl<T: FromAttributes> QueryStream<Relation<T, EntityNodeRef>>
    for FindManyQuery<Relation<T, EntityNodeRef>>
{
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation<T, EntityNodeRef>, DatabaseError>>, DatabaseError>
    {
        let query = self.full_relation_subquery().with(
            vec![
                "r".to_string(),
                "r_e".to_string(),
                "from".to_string(),
                "to".to_string(),
                "COLLECT(DISTINCT n{.*}) AS attrs".to_string(),
            ],
            "RETURN r{.*, from: from.id, to: to.id, attributes: attrs} as r",
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "relation_node::FindManyQuery::<Relation<T, EntityNodeRef>>:\n{}",
                query.compile()
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationEdge<EntityNodeRef>,
            attributes: Vec<AttributeNode>,
        }

        let stream = self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| {
                row_result.and_then(|row| {
                    T::from_attributes(row.attributes.into())
                        .map(|attributes| Relation {
                            relation: row.node,
                            attributes,
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}

impl<T: FromAttributes> QueryStream<Relation<T, EntityNode>>
    for FindManyQuery<Relation<T, EntityNode>>
{
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation<T, EntityNode>, DatabaseError>>, DatabaseError>
    {
        let query = self.full_relation_subquery().with(
            vec![
                "r".to_string(),
                "r_e".to_string(),
                "from".to_string(),
                "to".to_string(),
                "COLLECT(DISTINCT n{.*}) AS attrs".to_string(),
            ],
            "RETURN r{.*, from: from, to: to, attributes: attrs} as r",
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "relation_node::FindManyQuery::<Relation<T, EntityNodeRef>>:\n{}",
                query.compile()
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationEdge<EntityNode>,
            attributes: Vec<AttributeNode>,
        }

        let stream = self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| {
                row_result.and_then(|row| {
                    T::from_attributes(row.attributes.into())
                        .map(|attributes| Relation {
                            relation: row.node,
                            attributes,
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use futures::{pin_mut, StreamExt, TryStreamExt};

    use crate::{
        block::BlockMetadata,
        indexer_ids,
        mapping::{
            self, prop_filter,
            relation::{find_many, insert_many, RelationEdge},
            triple, EntityFilter, EntityNode, EntityNodeRef, Query, QueryStream, Relation, Triple,
        },
        relation::utils::RelationFilter,
        system_ids,
    };

    #[derive(Clone, Debug, PartialEq)]
    struct Foo {
        name: String,
        bar: u64,
    }

    impl mapping::IntoAttributes for Foo {
        fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
            Ok(mapping::Attributes::default()
                .attribute(("name", self.name))
                .attribute(("bar", self.bar)))
        }
    }

    impl mapping::FromAttributes for Foo {
        fn from_attributes(
            mut attributes: mapping::Attributes,
        ) -> Result<Self, mapping::TriplesConversionError> {
            Ok(Self {
                name: attributes.pop("name")?,
                bar: attributes.pop("bar")?,
            })
        }
    }

    #[tokio::test]
    async fn test_find_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let block = &BlockMetadata::default();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice:Entity {{id: "alice"}})
            CREATE (bob:Entity {{id: "bob"}})
            CREATE (charlie:Entity {{id: "charlie"}})
            CREATE (knows:Entity {{id: "knows"}})
            CREATE (alice) -[r1:RELATION {{id: "abc", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "0"}}]-> (bob)
            SET r1 += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            CREATE (alice) -[r2:RELATION {{id: "dev", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "1"}}]-> (charlie)
            SET r2 += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        ))
            .param("block_timestamp", block.timestamp.to_rfc3339())
            .param("block_number", block.block_number.to_string())
        )
            .await
            .expect("Failed to insert data");

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        let found_relations = find_many::<RelationEdge<EntityNodeRef>>(&neo4j)
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("knows")))
                    .from_(EntityFilter::default().id(prop_filter::value("alice"))),
            )
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(found_relations, relation_nodes);
    }

    #[tokio::test]
    async fn test_find_many_node() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
                Triple::new("charlie", "name", "Charlie"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .relations(relation_nodes.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many::<RelationEdge<EntityNode>>(&neo4j)
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("knows")))
                    .from_(EntityFilter::default().id(prop_filter::value("alice"))),
            )
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(
            found_relations,
            vec![
                RelationEdge {
                    id: "abc".to_string(),
                    from: EntityNode {
                        id: "alice".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "bob".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "knows".to_string(),
                    index: "0".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                RelationEdge {
                    id: "dev".to_string(),
                    from: EntityNode {
                        id: "alice".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "charlie".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "knows".to_string(),
                    index: "1".to_string(),
                    system_properties: BlockMetadata::default().into(),
                }
            ],
        );
    }

    #[tokio::test]
    async fn test_insert_find_many_relations() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let stream = find_many::<Relation<Foo, EntityNodeRef>>(&neo4j)
            .space_id(prop_filter::value::<String>("ROOT"))
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("relation_type"))),
            )
            .limit(1)
            .send()
            .await
            .expect("Failed to find relations");

        pin_mut!(stream);

        let found_relation: Relation<Foo, EntityNodeRef> = stream
            .next()
            .await
            .expect("Failed to get next relation")
            .expect("Relation not found");

        assert_eq!(found_relation.relation.id, relation.relation.id);
        assert_eq!(found_relation.attributes, relation.attributes);
    }

    #[tokio::test]
    async fn test_insert_find_many_relations_node() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let stream = find_many::<Relation<Foo, EntityNode>>(&neo4j)
            .space_id(prop_filter::value::<String>("ROOT"))
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("relation_type"))),
            )
            .limit(1)
            .send()
            .await
            .expect("Failed to find relations");

        pin_mut!(stream);

        let found_relation: Relation<Foo, EntityNode> = stream
            .next()
            .await
            .expect("Failed to get next relation")
            .expect("Relation not found");

        assert_eq!(found_relation.relation.id, relation.relation.id);
        assert_eq!(found_relation.attributes, relation.attributes);
    }
}
