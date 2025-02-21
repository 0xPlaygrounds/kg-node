use futures::{pin_mut, StreamExt};
use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{
        self,
        attributes::{FromAttributes, IntoAttributes},
        entity, entity_node, prop_filter,
        query_utils::{AttributeFilter, PropFilter, Query, QueryStream},
        relation, Attributes, Entity, EntityNode, Relation, TriplesConversionError, Value,
    },
    network_ids, system_ids,
};

use super::BlockMetadata;

#[derive(Clone)]
pub struct Space {
    pub network: String,
    pub governance_type: SpaceGovernanceType,
    /// The address of the space's DAO contract.
    pub dao_contract_address: String,
    /// The address of the space plugin contract.
    pub space_plugin_address: Option<String>,
    /// The address of the voting plugin contract.
    pub voting_plugin_address: Option<String>,
    /// The address of the member access plugin contract.
    pub member_access_plugin: Option<String>,
    /// The address of the personal space admin plugin contract.
    pub personal_space_admin_plugin: Option<String>,
}

impl Space {
    pub fn gen_id(network: &str, address: &str) -> String {
        ids::create_id_from_unique_string(format!("{network}:{}", checksum_address(address)))
    }

    pub fn builder(id: &str, dao_contract_address: &str) -> SpaceBuilder {
        SpaceBuilder::new(id, dao_contract_address)
    }

    /// Find a space by its DAO contract address.
    pub async fn find_by_dao_address(
        neo4j: &neo4rs::Graph,
        dao_contract_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        entity::find_one(
            neo4j,
            Space::gen_id(network_ids::GEO, dao_contract_address),
            indexer_ids::INDEXER_SPACE_ID,
            None,
        )
        .send()
        .await
    }

    pub async fn find_entity_by_dao_address(
        neo4j: &neo4rs::Graph,
        dao_contract_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        entity_node::find_one(neo4j, Space::gen_id(network_ids::GEO, dao_contract_address))
            .send()
            .await
    }

    /// Find a space by its space plugin address.
    pub async fn find_by_space_plugin_address(
        neo4j: &neo4rs::Graph,
        space_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(space_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    pub async fn find_entity_by_space_plugin_address(
        neo4j: &neo4rs::Graph,
        space_plugin_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        let stream = entity_node::find_many(neo4j)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS)
                    .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
                    .value(prop_filter::value(checksum_address(space_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its voting plugin address.
    pub async fn find_by_voting_plugin_address(
        neo4j: &neo4rs::Graph,
        voting_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(voting_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    pub async fn find_entity_by_voting_plugin_address(
        neo4j: &neo4rs::Graph,
        voting_plugin_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        let stream = entity_node::find_many(neo4j)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)
                    .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
                    .value(prop_filter::value(checksum_address(voting_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its member access plugin address.
    pub async fn find_by_member_access_plugin(
        neo4j: &neo4rs::Graph,
        member_access_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(member_access_plugin))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its personal space admin plugin address.
    pub async fn find_by_personal_plugin_address(
        neo4j: &neo4rs::Graph,
        personal_space_admin_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS).value(
                    PropFilter::default().value(checksum_address(personal_space_admin_plugin)),
                ),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Returns all spaces
    pub async fn find_all(_neo4j: &neo4rs::Graph) -> Result<Vec<Self>, DatabaseError> {
        // const QUERY: &str = const_format::formatcp!(
        //     "MATCH (n:`{INDEXED_SPACE}`) RETURN n",
        //     INDEXED_SPACE = system_ids::SPACE_TYPE,
        // );

        // let query = neo4rs::query(QUERY);

        // #[derive(Debug, Deserialize)]
        // struct ResultRow {
        //     n: neo4rs::Node,
        // }

        // neo4j
        //     .execute(query)
        //     .await?
        //     .into_stream_as::<ResultRow>()
        //     .map_err(DatabaseError::from)
        //     .and_then(|neo4j_node| async move { Ok(neo4j_node.n.try_into()?) })
        //     .try_collect::<Vec<_>>()
        //     .await
        todo!()
    }
}

impl IntoAttributes for Space {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        let mut attributes = Attributes::default()
            .attribute((system_ids::NETWORK_ATTRIBUTE, self.network))
            .attribute((indexer_ids::SPACE_GOVERNANCE_TYPE, self.governance_type))
            .attribute((indexer_ids::SPACE_DAO_ADDRESS, self.dao_contract_address));

        if let Some(space_plugin_address) = self.space_plugin_address {
            attributes.attribute_mut((indexer_ids::SPACE_PLUGIN_ADDRESS, space_plugin_address))
        }

        if let Some(voting_plugin_address) = self.voting_plugin_address {
            attributes.attribute_mut((
                indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS,
                voting_plugin_address,
            ))
        }

        if let Some(member_access_plugin) = self.member_access_plugin {
            attributes.attribute_mut((
                indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS,
                member_access_plugin,
            ))
        }

        if let Some(personal_space_admin_plugin) = self.personal_space_admin_plugin {
            attributes.attribute_mut((
                indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS,
                personal_space_admin_plugin,
            ))
        }

        Ok(attributes)
    }
}

impl FromAttributes for Space {
    fn from_attributes(mut attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(Self {
            network: attributes.pop(system_ids::NETWORK_ATTRIBUTE)?,
            governance_type: attributes.pop(indexer_ids::SPACE_GOVERNANCE_TYPE)?,
            dao_contract_address: attributes.pop(indexer_ids::SPACE_DAO_ADDRESS)?,
            space_plugin_address: attributes.pop_opt(indexer_ids::SPACE_PLUGIN_ADDRESS)?,
            voting_plugin_address: attributes.pop_opt(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)?,
            member_access_plugin: attributes.pop_opt(indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS)?,
            personal_space_admin_plugin: attributes
                .pop_opt(indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS)?,
        })
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub enum SpaceGovernanceType {
    #[default]
    Public,
    Personal,
}

impl From<SpaceGovernanceType> for Value {
    fn from(governance_type: SpaceGovernanceType) -> Self {
        match governance_type {
            SpaceGovernanceType::Public => Value::text("Public".to_string()),
            SpaceGovernanceType::Personal => Value::text("Personal".to_string()),
        }
    }
}

impl TryFrom<Value> for SpaceGovernanceType {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value.as_str() {
            "Public" => Ok(SpaceGovernanceType::Public),
            "Personal" => Ok(SpaceGovernanceType::Personal),
            _ => Err(format!(
                "Invalid SpaceGovernanceType value: {}",
                value.value
            )),
        }
    }
}

pub struct SpaceBuilder {
    id: String,
    network: String,
    governance_type: SpaceGovernanceType,
    dao_contract_address: String,
    space_plugin_address: Option<String>,
    voting_plugin_address: Option<String>,
    member_access_plugin: Option<String>,
    personal_space_admin_plugin: Option<String>,
}

impl SpaceBuilder {
    pub fn new(id: &str, dao_contract_address: &str) -> Self {
        Self {
            id: id.to_string(),
            network: network_ids::GEO.to_string(),
            governance_type: SpaceGovernanceType::Public,
            dao_contract_address: checksum_address(dao_contract_address),
            space_plugin_address: None,
            voting_plugin_address: None,
            member_access_plugin: None,
            personal_space_admin_plugin: None,
        }
    }

    pub fn network(mut self, network: String) -> Self {
        self.network = network;
        self
    }

    pub fn governance_type(mut self, governance_type: SpaceGovernanceType) -> Self {
        self.governance_type = governance_type;
        self
    }

    pub fn dao_contract_address(mut self, dao_contract_address: &str) -> Self {
        self.dao_contract_address = checksum_address(dao_contract_address);
        self
    }

    pub fn space_plugin_address(mut self, space_plugin_address: &str) -> Self {
        self.space_plugin_address = Some(checksum_address(space_plugin_address));
        self
    }

    pub fn voting_plugin_address(mut self, voting_plugin_address: &str) -> Self {
        self.voting_plugin_address = Some(checksum_address(voting_plugin_address));
        self
    }

    pub fn member_access_plugin(mut self, member_access_plugin: &str) -> Self {
        self.member_access_plugin = Some(checksum_address(member_access_plugin));
        self
    }

    pub fn personal_space_admin_plugin(mut self, personal_space_admin_plugin: &str) -> Self {
        self.personal_space_admin_plugin = Some(checksum_address(personal_space_admin_plugin));
        self
    }

    pub fn build(self) -> Entity<Space> {
        Entity::new(
            &self.id,
            Space {
                network: self.network,
                governance_type: self.governance_type,
                dao_contract_address: self.dao_contract_address,
                space_plugin_address: self.space_plugin_address,
                voting_plugin_address: self.voting_plugin_address,
                member_access_plugin: self.member_access_plugin,
                personal_space_admin_plugin: self.personal_space_admin_plugin,
            },
        )
        .with_type(system_ids::SPACE_TYPE)
    }
}

/// Parent space relation (for subspaces).
/// Space > PARENT_SPACE > Space
#[derive(Clone)]
pub struct ParentSpace;

impl ParentSpace {
    pub fn generate_id(space_id: &str, parent_space_id: &str) -> String {
        ids::create_id_from_unique_string(format!("PARENT_SPACE:{space_id}:{parent_space_id}"))
    }

    pub fn new(space_id: &str, parent_space_id: &str) -> Relation<Self> {
        Relation::new(
            Self::generate_id(space_id, parent_space_id),
            space_id,
            parent_space_id,
            indexer_ids::PARENT_SPACE,
            "0",
            Self,
        )
    }

    /// Delete a relation between a space and its parent space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        parent_space_id: &str,
    ) -> Result<(), DatabaseError> {
        relation::delete_one(
            neo4j,
            block,
            ParentSpace::generate_id(space_id, parent_space_id),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .send()
        .await
    }
}

impl mapping::IntoAttributes for ParentSpace {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default())
    }
}

impl FromAttributes for ParentSpace {
    fn from_attributes(
        _attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {})
    }
}
