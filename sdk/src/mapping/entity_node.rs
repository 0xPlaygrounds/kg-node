use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;

use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata, system_ids};

use super::{
    attributes,
    query_utils::{
        edge_filter::EdgeFilter, prop_filter, AttributeFilter, PropFilter, Query, QueryPart,
    },
    relation_node, triple, AttributeNode, Triple,
};

/// Neo4j model of an Entity
#[derive(Debug, Deserialize, PartialEq)]
pub struct EntityNode {
    pub id: String,

    /// System properties
    #[serde(flatten)]
    pub system_properties: SystemProperties,
}

impl EntityNode {
    pub fn delete(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> DeleteOneQuery {
        DeleteOneQuery::new(neo4j, block, self.id, space_id.into(), space_version.into())
    }

    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.into(), space_version)
    }

    pub fn get_outbound_relations(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation_node::FindManyQuery {
        relation_node::FindManyQuery::new(neo4j)
            .from_id(prop_filter::value(self.id.clone()))
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn get_inbound_relations(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation_node::FindManyQuery {
        relation_node::FindManyQuery::new(neo4j)
            .to_id(prop_filter::value(self.id.clone()))
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.into(),
            space_version.into(),
            Triple {
                entity: self.id.clone(),
                attribute: attribute.id,
                value: attribute.value,
            },
        )
    }

    pub fn set_attributes<T>(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attributes: T,
    ) -> attributes::InsertOneQuery<T> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.into(),
            space_version.into(),
            attributes,
        )
    }

    /// Get all the versions that have been applied to this entity
    pub async fn versions(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
    ) -> Result<Vec<EntityVersion>, DatabaseError> {
        const QUERY: &str = r#"
            MATCH (:Entity {id: $id}) -[r:ATTRIBUTE]-> (:Attribute)
            WHERE r.space_id = $space_id
            WITH COLLECT(DISTINCT r.min_version) AS versions
            UNWIND versions AS version
            MATCH (e:Entity) -[:ATTRIBUTE]-> ({id: $EDIT_INDEX_ATTR, value: version})
            RETURN {entity_id: $id, id: e.id, index: version}
        "#;

        let query = neo4rs::query(QUERY)
            .param("id", self.id.clone())
            .param("space_id", space_id.into())
            .param("EDIT_INDEX_ATTR", indexer_ids::EDIT_INDEX_ATTRIBUTE);

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<EntityVersion>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row) })
            .try_collect::<Vec<_>>()
            .await?)
    }
}

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

pub fn find_one(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FindOneQuery {
    FindOneQuery::new(neo4j, id.into())
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SystemProperties {
    #[serde(rename = "82nP7aFmHJLbaPFszj2nbx")] // CREATED_AT_TIMESTAMP
    pub created_at: DateTime<Utc>,
    #[serde(rename = "59HTYnd2e4gBx2aA98JfNx")] // CREATED_AT_BLOCK
    pub created_at_block: String,
    #[serde(rename = "5Ms1pYq8v8G1RXC3wWb9ix")] // UPDATED_AT_TIMESTAMP
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "7pXCVQDV9C7ozrXkpVg8RJ")] // UPDATED_AT_BLOCK
    pub updated_at_block: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EntityVersion {
    pub entity_id: String,
    pub id: String,
    pub index: String,
}

impl Default for SystemProperties {
    fn default() -> Self {
        Self {
            created_at: Default::default(),
            created_at_block: "0".to_string(),
            updated_at: Default::default(),
            updated_at_block: "0".to_string(),
        }
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    id: String,
}

impl FindOneQuery {
    pub fn new(neo4j: &neo4rs::Graph, id: String) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
        }
    }
}

impl Query<Option<EntityNode>> for FindOneQuery {
    async fn send(self) -> Result<Option<EntityNode>, DatabaseError> {
        const QUERY: &str = r#"
            MATCH (e:Entity {id: $id})
            RETURN e
        "#;

        let query = neo4rs::query(QUERY).param("id", self.id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.e)
            })
            .transpose()?)
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    filter: EntityFilter,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            filter: EntityFilter::default(),
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.filter.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.filter.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.filter.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.filter.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.filter.attributes.extend(attributes);
    }

    /// Overwrite the current filter with a new one
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause("(e:Entity)")
            .return_clause("e");

        query_part.merge_mut(self.filter.into_query_part("e"));

        query_part
    }
}

impl Query<Vec<EntityNode>> for FindManyQuery {
    async fn send(self) -> Result<Vec<EntityNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        // let part = self.into_query_part();
        // println!("FindManyQuery: {}", part.query());
        // let query = part.build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.e) })
            .try_collect::<Vec<_>>()
            .await?)
    }
}

// TODO: Add types filter
#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    id: Option<PropFilter<String>>,
    attributes: Vec<AttributeFilter>,
    relations: Option<EntityRelationFilter>,
}

impl EntityFilter {
    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.attributes.extend(attributes);
    }

    pub fn relations(mut self, relations: EntityRelationFilter) -> Self {
        self.relations = Some(relations);
        self
    }

    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let mut query_part = QueryPart::default();

        if let Some(id) = self.id {
            query_part.merge_mut(id.into_query_part(&node_var, "id"));
        }

        for attribute in self.attributes {
            query_part.merge_mut(attribute.into_query_part(&node_var));
        }

        if let Some(relations) = self.relations {
            query_part.merge_mut(relations.into_query_part(node_var));
        }

        query_part
    }
}

/// Filter used to:
/// - Filter the relations outgoing from the entity
/// - Filter an entity by its outgoing relations
#[derive(Clone, Debug, Default)]
pub struct EntityRelationFilter {
    relation_type: Option<EdgeFilter>,
    to_id: Option<EdgeFilter>,
    space_version: Option<String>,
}

impl EntityRelationFilter {
    pub fn relation_type(mut self, relation_type: EdgeFilter) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn to_id(mut self, to_id: EdgeFilter) -> Self {
        self.to_id = Some(to_id);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.space_version = Some(version.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type.is_none() && self.to_id.is_none()
    }

    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let rel_node_var = format!("r_{node_var}");
        let mut query_part = QueryPart::default();

        if !self.is_empty() {
            query_part = query_part
                .match_clause(format!(
                    "({node_var}) <-[:`{FROM_ENTITY}`]- ({rel_node_var})",
                    FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE
                ));
            
            if let Some(relation_type) = self.relation_type {
                query_part.merge_mut(relation_type.into_query_part(
                    &rel_node_var,
                    system_ids::RELATION_TYPE_ATTRIBUTE,
                    self.space_version.clone(),
                ));
            }
    
            if let Some(to_id) = self.to_id {
                query_part.merge_mut(to_id.into_query_part(
                    &rel_node_var,
                    system_ids::RELATION_TO_ATTRIBUTE,
                    self.space_version,
                ));
            }
        }

        query_part
    }
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    id: String,
    space_id: String,
    space_version: String,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        id: String,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e:Entity {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> (:Attribute)
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
            .param("entity_id", self.id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[tokio::test]
    async fn test_find_by_id() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .insert(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let entity = find_one(&neo4j, "abc")
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(
            entity,
            EntityNode {
                id: "abc".to_string(),
                system_properties: SystemProperties::default(),
            }
        );
    }
}
