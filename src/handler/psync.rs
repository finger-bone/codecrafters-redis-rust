use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::RwLock};

use anyhow::Error;
use tokio::net::TcpStream;

use crate::{protocol::RObject, Config};
use hex;

pub async fn handle_psync(
    _args: &Vec<RObject>,
    stream: &mut TcpStream,
    config: Arc<RwLock<Config>>,
) -> Result<(), Error> {
    stream.write(
        RObject::SimpleString(
            format!("FULLRESYNC {} 0", config.read().await.master_replid)
        ).to_string().as_bytes()
    ).await.expect(
        "Failed to respond when fullsync."
    );

    let rdb_str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
    let rdb_bytes = hex::decode(rdb_str).expect("Failed to decode RDB hex string");

    stream.write(
        format!("${}\r\n", rdb_bytes.len()).as_bytes()
    ).await.expect("Failed to write RDB length");

    stream.write(&rdb_bytes).await.expect("Failed to write RDB file");

    Ok(())
}