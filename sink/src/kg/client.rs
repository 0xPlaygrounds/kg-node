use futures::TryStreamExt;
use serde::Deserialize;

use crate::bootstrap::{self, constants};
// use web3_utils::checksum_address;

use sdk::{
    error::DatabaseError,
    mapping::{self, Entity, Relation},
    models::{self, BlockMetadata},
    pb, system_ids,
};

#[derive(Clone)]
pub struct Client {
    pub neo4j: neo4rs::Graph,
}

impl Client {
    pub async fn new(uri: &str, user: &str, pass: &str) -> anyhow::Result<Self> {
        let neo4j = neo4rs::Graph::new(uri, user, pass).await?;
        Ok(Self { neo4j })
    }

    /// Bootstrap the database with the initial data
    pub async fn bootstrap(&self, _rollup: bool) -> Result<(), DatabaseError> {
        // let bootstrap_ops = if rollup {
        //     conversions::batch_ops(bootstrap::bootstrap())
        // } else {
        //     bootstrap::bootstrap().map(Op::from).collect()
        // };

        // stream::iter(bootstrap_ops)
        //     .map(Ok) // Convert to Result to be able to use try_for_each
        //     .try_for_each(|op| async move { op.apply_op(self, ROOT_SPACE_ID).await })
        //     .await?;
        self.upsert_entity(
            &BlockMetadata::default(),
            &models::Space::builder(constants::ROOT_SPACE_ID, constants::ROOT_SPACE_DAO_ADDRESS)
                .space_plugin_address(constants::ROOT_SPACE_PLUGIN_ADDRESS)
                .voting_plugin_address(constants::ROOT_SPACE_MAIN_VOTING_ADDRESS)
                .member_access_plugin(constants::ROOT_SPACE_MEMBER_ACCESS_ADDRESS)
                .build(),
        )
        .await?;

        self.process_ops(
            &BlockMetadata::default(),
            constants::ROOT_SPACE_ID,
            bootstrap::bootstrap(),
        )
        .await
    }

    /// Reset the database by deleting all nodes and relations and re-bootstrapping it
    pub async fn reset_db(&self, rollup: bool) -> anyhow::Result<()> {
        // Delete all nodes and relations
        let mut txn = self.neo4j.start_txn().await?;
        txn.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await?;
        txn.commit().await?;

        // Re-bootstrap the database
        self.bootstrap(rollup).await?;

        Ok(())
    }

    pub async fn upsert_relation<T: serde::Serialize>(
        &self,
        block: &models::BlockMetadata,
        relation: &Relation<T>,
    ) -> Result<(), DatabaseError> {
        self.run(relation.upsert_query(block)?).await?;

        Ok(())
    }

    pub async fn upsert_entity<T: serde::Serialize>(
        &self,
        block: &models::BlockMetadata,
        entity: &Entity<T>,
    ) -> Result<(), DatabaseError> {
        entity.upsert_query(&self.neo4j, block).await?;

        Ok(())
    }

    pub async fn run(&self, query: mapping::Query<()>) -> Result<(), DatabaseError> {
        self.neo4j.run(query.query).await?;
        Ok(())
    }

    // pub async fn find_node_by_id<T: for<'a> Deserialize<'a> + Send>(
    //     &self,
    //     id: &str,
    // ) -> Result<Option<Entity<T>>, DatabaseError> {
    //     let query = Entity::<T>::find_by_id_query(id);
    //     self.find_node(query).await
    // }

    pub async fn find_node<T: for<'a> Deserialize<'a> + Send>(
        &self,
        query: mapping::Query<T>,
    ) -> Result<Option<Entity<T>>, DatabaseError> {
        self.neo4j
            .execute(query.query)
            .await?
            .next()
            .await?
            .map(|row| Ok::<_, DatabaseError>(Entity::<T>::try_from(row.to::<neo4rs::Node>()?)?))
            .transpose()
    }

    pub async fn find_nodes<T: for<'a> Deserialize<'a> + Send>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Vec<Entity<T>>, DatabaseError> {
        self.neo4j
            .execute(query)
            .await?
            .into_stream_as::<neo4rs::Node>()
            .map_err(DatabaseError::from)
            .and_then(|neo4j_node| async move { Ok(Entity::<T>::try_from(neo4j_node)?) })
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn find_node_from_relation<T: for<'a> Deserialize<'a> + Send>(
        &self,
        relation_id: &str,
    ) -> Result<Option<Entity<T>>, DatabaseError> {
        let query =
            mapping::Query::new("MATCH (n) -[r {id: $id}]-> () RETURN n").param("id", relation_id);
        self.find_node::<T>(query).await
    }

    pub async fn find_node_to_relation<T: for<'a> Deserialize<'a> + Send>(
        &self,
        relation_id: &str,
    ) -> Result<Option<Entity<T>>, DatabaseError> {
        let query =
            mapping::Query::new("MATCH () -[r {id: $id}]-> (n) RETURN n").param("id", relation_id);
        self.find_node::<T>(query).await
    }

    pub async fn find_types<T: for<'a> Deserialize<'a> + Send>(
        &self,
    ) -> Result<Vec<Entity<T>>, DatabaseError> {
        let query = neo4rs::query(&format!("MATCH (t:`{}`) RETURN t", system_ids::SCHEMA_TYPE));
        self.find_nodes::<T>(query).await
    }

    pub async fn process_ops(
        &self,
        block: &models::BlockMetadata,
        space_id: &str,
        ops: impl IntoIterator<Item = pb::grc20::Op>,
    ) -> Result<(), DatabaseError> {
        for op in ops {
            match (op.r#type(), op.triple) {
                (
                    pb::grc20::OpType::SetTriple,
                    Some(pb::grc20::Triple {
                        entity,
                        attribute,
                        value: Some(value),
                    }),
                ) => {
                    tracing::info!("SetTriple: {}, {}, {:?}", entity, attribute, value,);

                    Entity::<()>::set_triple(
                        &self.neo4j,
                        block,
                        space_id,
                        &entity,
                        &attribute,
                        &value,
                    )
                    .await?
                }
                (pb::grc20::OpType::DeleteTriple, Some(triple)) => {
                    tracing::info!(
                        "DeleteTriple: {}, {}, {:?}",
                        triple.entity,
                        triple.attribute,
                        triple.value,
                    );

                    Entity::<()>::delete_triple(&self.neo4j, block, space_id, triple).await?
                }
                (typ, maybe_triple) => {
                    tracing::warn!("Unhandled case: {:?} {:?}", typ, maybe_triple);
                }
            }
        }

        Ok(())
    }
}
