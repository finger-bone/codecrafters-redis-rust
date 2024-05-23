use std::sync::Arc;

use anyhow::{bail, Error};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{protocol::RObject, Config};

pub async fn handle_info(
    args: &Vec<RObject>,
    config: Arc<RwLock<Config>>,
    stream: &mut TcpStream
) -> Result<(), Error> {
    let specification = match args.get(1).expect("No specification") {
        RObject::BulkString(s) => s,
        _ => bail!("Expect a specification after the info command")
    };

    match specification.as_str() {
        "replication" => {
            stream.write(
                RObject::BulkString(
                    format!(
                        "role:{}\n", config.read().await.role
                    )
                ).to_string().as_bytes()
            ).await.expect("Failed to write to stream handling info replication.")
        }
        _ => bail!("Specification not allowed")
    };
    Ok(())
}