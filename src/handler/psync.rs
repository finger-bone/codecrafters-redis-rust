use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::RwLock};

use anyhow::Error;
use tokio::net::TcpStream;

use crate::{protocol::RObject, Config};

pub async fn handle_psync(
    _args: &Vec<RObject>,
    stream: &mut TcpStream,
    config: Arc<RwLock<Config>>,
) -> Result<(), Error> {
    stream.write(
        RObject::SimpleString(
            format!("FULLSYNC {} 0", config.read().await.master_replid)
        ).to_string().as_bytes()
    ).await.expect(
        "Failed to response when fullsync."
    );
    Ok(())
} 