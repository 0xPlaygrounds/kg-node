use clap::{Args, Parser, Subcommand};
use futures::{stream, StreamExt, TryStreamExt};
use ipfs::IpfsClient;
use sdk::{ids, pb::grc20};
use sink::{
    kg::{self, mapping::DefaultAttributes},
    ops::conversions,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// const ROOT_SPACE_IMPORTS: &str = "bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy";
// const CONSTRUCTION_SPACE_IMPORTS: &str = "bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq";
// const CONSTRUCTION_SPACE_IMPORTS: &str =
//     "bafkreiadpdybqrlieaql57fjpcwhy25ut3s742qkhuxz4i6meahhrpvnf4";
// const CONSTRUCTION_SPACE_IMPORTS: &str =
//     "bafkreidgyievktbezgsaxnnuylyn7acgy3kmaderzy4t4lwnfenhrggice";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level();
    init_tracing();
    let args = AppArgs::parse();

    let kg_client = kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;
    let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    match args.command {
        Command::FindTriples { id: _ } => {
            // let ops = find_triples(bootstrap_ops, &id)
            //     .into_iter()
            //     .chain(find_triples(root_space_ops, &id));

            // for (op_type, triple) in ops {
            //     match op_type {
            //         grc20::OpType::SetTriple => println!("SetTriple: {:?}", triple),
            //         grc20::OpType::DeleteTriple => println!("DeleteTriple: {:?}", triple),
            //         _ => (),
            //     }
            // }
            unimplemented!()
        }
        Command::Describe { id } => {
            let entity = kg_client
                .find_node_by_id::<DefaultAttributes>(&id)
                .await?
                .expect("Entity not found");

            println!("Entity: {}", entity.name_or_id());

            let attributes = kg_client
                .attribute_nodes::<DefaultAttributes>(entity.id())
                .await?;

            for attribute in attributes {
                println!("\tAttribute: {}", attribute.name_or_id());
                if let Some(value_type) = kg_client
                    .value_type_node::<DefaultAttributes>(attribute.id())
                    .await?
                {
                    println!("\t\tValue type: {}", value_type.name_or_id());
                }
            }
        }
        Command::Codegen => {
            let code = codegen::codegen(&kg_client).await?;
            std::fs::write("./src/space.ts", code)?;
            println!("Generated code has been written to ./src/space.ts");
        }
        Command::ResetDb => {
            kg_client.reset_db(true).await?;
        }
        Command::ImportSpace {
            ipfs_hash,
            space_id,
        } => {
            let ops = import_space(&ipfs_client, &ipfs_hash).await?;
            let rollups = conversions::batch_ops(ops);

            for op in rollups {
                op.apply_op(&kg_client, &space_id).await?;
            }
        }
        Command::CreateEntityId { n } => {
            for _ in 0..n {
                let entity_id = ids::create_geo_id();
                println!("{}", entity_id);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Find triples related to an entity
    FindTriples {
        /// Entity ID
        id: String,
    },

    /// Describe an entity
    Describe {
        /// Entity ID
        id: String,
    },

    /// Reset the database
    ResetDb,

    /// Import a space
    ImportSpace {
        /// IPFS hash
        ipfs_hash: String,

        /// Space ID (defaults to root space)
        // #[arg(default_value = ROOT_SPACE_ID)]
        space_id: String,
    },

    /// Codegen
    Codegen,

    /// Create a new unique entity id
    CreateEntityId {
        #[arg(default_value = "1")]
        n: usize,
    },
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

pub fn find_triples(
    ops: impl IntoIterator<Item = grc20::Op>,
    entity_id: &str,
) -> Vec<(grc20::OpType, grc20::Triple)> {
    ops.into_iter()
        .filter_map(|op| match (op.r#type(), &op.triple) {
            (
                grc20::OpType::SetTriple,
                Some(grc20::Triple {
                    entity,
                    attribute,
                    value: Some(grc20::Value { value, .. }),
                }),
            ) if *entity == entity_id || *attribute == entity_id || *value == entity_id => Some((
                grc20::OpType::SetTriple,
                op.triple.expect("Triple should be some"),
            )),

            (
                grc20::OpType::DeleteTriple,
                Some(grc20::Triple {
                    entity,
                    attribute,
                    value: Some(grc20::Value { value, .. }),
                }),
            ) if *entity == entity_id || *attribute == entity_id || *value == entity_id => Some((
                grc20::OpType::DeleteTriple,
                op.triple.expect("Triple should be some"),
            )),
            _ => None,
        })
        .collect()
}

async fn import_space(ipfs_client: &IpfsClient, ipfs_hash: &str) -> anyhow::Result<Vec<grc20::Op>> {
    let import = ipfs_client.get::<grc20::Import>(ipfs_hash, true).await?;

    Ok(stream::iter(import.edits)
        .then(|edit_hash| async move {
            let edit = ipfs_client
                .get::<grc20::ImportEdit>(&edit_hash, true)
                .await?;
            anyhow::Ok(edit.ops)
        })
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>())
}
