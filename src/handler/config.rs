use std::sync::Arc;

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{protocol::RObject, State};

pub async fn handle_config(
    args: &Vec<RObject>,
    stream: &mut TcpStream,
    state: Arc<RwLock<State>>,
) -> Result<(), Error> {
    // CONFIG GET dir
    let command = match args.get(1) {
        Some(RObject::BulkString(s)) => s,
        _ => return Err(anyhow::anyhow!("Expected BulkString as key")),
    };
    let target = match args.get(2) {
        Some(RObject::BulkString(s)) => s,
        _ => return Err(anyhow::anyhow!("Expected BulkString as key")),
    };

    match command.as_str() {
        "GET" => {
            match target.as_str() {
                "dir" => {
                    stream.write_all(
                        RObject::Array(
                            vec![
                                RObject::BulkString("dir".to_string()),
                                RObject::BulkString(state.read().await.dir.clone())
                            ]
                        ).to_string().as_bytes()
                    ).await.expect("Failed to write to stream handling config GET dir.")
                },
                "dbfilename" => {
                    stream.write_all(
                        RObject::Array(
                            vec![
                                RObject::BulkString("dbfilename".to_string()),
                                RObject::BulkString(state.read().await.dbfilename.clone())
                            ]
                        ).to_string().as_bytes()
                    ).await.expect("Failed to write to stream handling config GET dbfilename.")
                },
                _ => return Err(anyhow::anyhow!("Target not allowed")),
            }
        }
        _ => return Err(anyhow::anyhow!("Command not allowed")),
    }

    Ok(())
}