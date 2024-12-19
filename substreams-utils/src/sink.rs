use std::{env, process::exit, sync::Arc};

use anyhow::{format_err, Context};
use futures::StreamExt;
use prost::Message;

use crate::{
    pb::sf::substreams::{
        rpc::v2::{BlockScopedData, BlockUndoSignal},
        v1::Package,
    },
    substreams::SubstreamsEndpoint,
    substreams_stream::{BlockResponse, SubstreamsStream},
};

pub trait Sink: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn process_block_scoped_data(
        &self,
        data: &BlockScopedData,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    fn process_block_undo_signal(&self, _undo_signal: &BlockUndoSignal) -> Result<(), Self::Error> {
        // `BlockUndoSignal` must be treated as "delete every data that has been recorded after
        // block height specified by block in BlockUndoSignal". In the example above, this means
        // you must delete changes done by `Block #7b` and `Block #6b`. The exact details depends
        // on your own logic. If for example all your added record contain a block number, a
        // simple way is to do `delete all records where block_num > 5` which is the block num
        // received in the `BlockUndoSignal` (this is true for append only records, so when only `INSERT` are allowed).
        unimplemented!("you must implement some kind of block undo handling, or request only final blocks (tweak substreams_stream.rs)")
    }

    fn persist_cursor(&self, _cursor: String) -> Result<(), Self::Error> {
        // FIXME: Handling of the cursor is missing here. It should be saved each time
        // a full block has been correctly processed/persisted. The saving location
        // is your responsibility.
        //
        // By making it persistent, we ensure that if we crash, on startup we are
        // going to read it back from database and start back our SubstreamsStream
        // with it ensuring we are continuously streaming without ever losing a single
        // element.
        Ok(())
    }

    fn load_persisted_cursor(&self) -> Result<Option<String>, Self::Error> {
        // FIXME: Handling of the cursor is missing here. It should be loaded from
        // somewhere (local file, database, cloud storage) and then `SubstreamStream` will
        // be able correctly resume from the right block.
        Ok(None)
    }

    fn run(
        &self,
        endpoint_url: &str,
        spkg_file: &str,
        module_name: &str,
        start_block: i64,
        end_block: u64,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async move {
            let token_env = env::var("SUBSTREAMS_API_TOKEN").unwrap_or("".to_string());
            let mut token: Option<String> = None;
            if !token_env.is_empty() {
                token = Some(token_env);
            }

            let cursor: Option<String> = self.load_persisted_cursor()?;

            let package = read_package(spkg_file).await?;

            let endpoint = Arc::new(SubstreamsEndpoint::new(&endpoint_url, token).await?);

            let mut stream = SubstreamsStream::new(
                endpoint.clone(),
                cursor,
                package.modules.clone(),
                module_name.to_string(),
                start_block,
                end_block,
            );

            loop {
                match stream.next().await {
                    None => {
                        println!("Stream consumed");
                        break;
                    }
                    Some(Ok(BlockResponse::New(data))) => {
                        self.process_block_scoped_data(&data).await?;
                        self.persist_cursor(data.cursor)?;
                    }
                    Some(Ok(BlockResponse::Undo(undo_signal))) => {
                        self.process_block_undo_signal(&undo_signal)?;
                        self.persist_cursor(undo_signal.last_valid_cursor)?;
                    }
                    Some(Err(err)) => {
                        println!();
                        println!("Stream terminated with error");
                        println!("{:?}", err);
                        exit(1);
                    }
                }
            }

            Ok(())
        }
    }
}

async fn read_package(input: &str) -> Result<Package, anyhow::Error> {
    if input.starts_with("http") {
        return read_http_package(input).await;
    }

    // Assume it's a local file
    let content =
        std::fs::read(input).context(format_err!("read package from file '{}'", input))?;
    Package::decode(content.as_ref()).context("decode command")
}

async fn read_http_package(input: &str) -> Result<Package, anyhow::Error> {
    let body = reqwest::get(input).await?.bytes().await?;

    Package::decode(body).context("decode command")
}

#[allow(dead_code)]
fn read_block_range(pkg: &Package, module_name: &str) -> Result<(i64, u64), anyhow::Error> {
    let module = pkg
        .modules
        .as_ref()
        .unwrap()
        .modules
        .iter()
        .find(|m| m.name == module_name)
        .ok_or_else(|| format_err!("module '{}' not found in package", module_name))?;

    let mut input: String = "".to_string();
    if let Some(range) = env::args().nth(4) {
        input = range;
    };

    let (prefix, suffix) = match input.split_once(":") {
        Some((prefix, suffix)) => (prefix.to_string(), suffix.to_string()),
        None => ("".to_string(), input),
    };

    let start: i64 = match prefix.as_str() {
        "" => module.initial_block as i64,
        x if x.starts_with("+") => {
            let block_count = x
                .trim_start_matches("+")
                .parse::<u64>()
                .context("argument <stop> is not a valid integer")?;

            (module.initial_block + block_count) as i64
        }
        x => x
            .parse::<i64>()
            .context("argument <start> is not a valid integer")?,
    };

    let stop: u64 = match suffix.as_str() {
        "" => 0,
        "-" => 0,
        x if x.starts_with("+") => {
            let block_count = x
                .trim_start_matches("+")
                .parse::<u64>()
                .context("argument <stop> is not a valid integer")?;

            start as u64 + block_count
        }
        x => x
            .parse::<u64>()
            .context("argument <stop> is not a valid integer")?,
    };

    Ok((start, stop))
}
