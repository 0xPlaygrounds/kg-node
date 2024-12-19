use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    error::DatabaseError,
    ids,
    mapping::{Entity, Relation},
    network_ids, system_ids,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Space {
    pub network: String,
    pub r#type: SpaceType,
    /// The address of the space's DAO contract.
    pub dao_contract_address: String,
    /// The address of the space plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_plugin_address: Option<String>,
    /// The address of the voting plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voting_plugin_address: Option<String>,
    /// The address of the member access plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_access_plugin: Option<String>,
    /// The address of the personal space admin plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_space_admin_plugin: Option<String>,
}

impl Space {
    pub fn new_id(network: &str, address: &str) -> String {
        ids::create_id_from_unique_string(&format!("{network}:{}", checksum_address(address, None)))
    }

    pub fn builder(id: &str, dao_contract_address: &str) -> SpaceBuilder {
        SpaceBuilder::new(id, dao_contract_address)
    }

    pub fn new(id: &str, space: Space) -> Entity<Self> {
        Entity::new(id, system_ids::INDEXER_SPACE_ID, space).with_type(system_ids::INDEXED_SPACE)
    }

    /// Find a space by its DAO contract address.
    pub async fn find_by_dao_address(
        neo4j: &neo4rs::Graph,
        dao_contract_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}` {{dao_contract_address: $dao_contract_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query = neo4rs::query(QUERY).param("dao_contract_address", dao_contract_address);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<ResultRow>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    /// Find a space by its space plugin address.
    pub async fn find_by_space_plugin_address(
        neo4j: &neo4rs::Graph,
        space_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}` {{space_plugin_address: $space_plugin_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query = neo4rs::query(QUERY).param("space_plugin_address", space_plugin_address);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<ResultRow>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    /// Find a space by its voting plugin address.
    pub async fn find_by_voting_plugin_address(
        neo4j: &neo4rs::Graph,
        voting_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}` {{voting_plugin_address: $voting_plugin_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query = neo4rs::query(QUERY).param("voting_plugin_address", voting_plugin_address);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<ResultRow>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    /// Find a space by its member access plugin address.
    pub async fn find_by_member_access_plugin(
        neo4j: &neo4rs::Graph,
        member_access_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}` {{member_access_plugin: $member_access_plugin}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query = neo4rs::query(QUERY).param("member_access_plugin", member_access_plugin);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<ResultRow>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    /// Find a space by its personal space admin plugin address.
    pub async fn find_by_personal_plugin_address(
        neo4j: &neo4rs::Graph,
        personal_space_admin_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}` {{personal_space_admin_plugin: $personal_space_admin_plugin}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query =
            neo4rs::query(QUERY).param("personal_space_admin_plugin", personal_space_admin_plugin);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<ResultRow>()?;
                row.n.try_into()
            })
            .transpose()?)
    }

    /// Returns all spaces
    pub async fn find_all(neo4j: &neo4rs::Graph) -> Result<Vec<Entity<Self>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            "MATCH (n:`{INDEXED_SPACE}`) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        );

        let query = neo4rs::query(QUERY);

        #[derive(Debug, Deserialize)]
        struct ResultRow {
            n: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<ResultRow>()
            .map_err(DatabaseError::from)
            .and_then(|neo4j_node| async move { Ok(neo4j_node.n.try_into()?) })
            .try_collect::<Vec<_>>()
            .await
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub enum SpaceType {
    #[default]
    Public,
    Personal,
}

pub struct SpaceBuilder {
    id: String,
    network: String,
    r#type: SpaceType,
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
            r#type: SpaceType::Public,
            dao_contract_address: checksum_address(dao_contract_address, None),
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

    pub fn r#type(mut self, r#type: SpaceType) -> Self {
        self.r#type = r#type;
        self
    }

    pub fn dao_contract_address(mut self, dao_contract_address: &str) -> Self {
        self.dao_contract_address = checksum_address(dao_contract_address, None);
        self
    }

    pub fn space_plugin_address(mut self, space_plugin_address: &str) -> Self {
        self.space_plugin_address = Some(checksum_address(space_plugin_address, None));
        self
    }

    pub fn voting_plugin_address(mut self, voting_plugin_address: &str) -> Self {
        self.voting_plugin_address = Some(checksum_address(voting_plugin_address, None));
        self
    }

    pub fn member_access_plugin(mut self, member_access_plugin: &str) -> Self {
        self.member_access_plugin = Some(checksum_address(member_access_plugin, None));
        self
    }

    pub fn personal_space_admin_plugin(mut self, personal_space_admin_plugin: &str) -> Self {
        self.personal_space_admin_plugin =
            Some(checksum_address(personal_space_admin_plugin, None));
        self
    }

    pub fn build(self) -> Entity<Space> {
        Entity::new(
            &self.id,
            system_ids::INDEXER_SPACE_ID,
            Space {
                network: self.network,
                r#type: self.r#type,
                dao_contract_address: self.dao_contract_address,
                space_plugin_address: self.space_plugin_address,
                voting_plugin_address: self.voting_plugin_address,
                member_access_plugin: self.member_access_plugin,
                personal_space_admin_plugin: self.personal_space_admin_plugin,
            },
        )
        .with_type(system_ids::INDEXED_SPACE)
    }
}

/// Parent space relation (for subspaces).
#[derive(Deserialize, Serialize)]
pub struct ParentSpace;

impl ParentSpace {
    pub fn new(space_id: &str, parent_space_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_geo_id(),
            system_ids::INDEXER_SPACE_ID,
            space_id,
            parent_space_id,
            Self,
        )
        .with_type(system_ids::PARENT_SPACE)
    }
}
