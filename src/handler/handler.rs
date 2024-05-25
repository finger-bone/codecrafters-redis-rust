use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Error};
use tokio::{net::TcpStream, sync::RwLock};

use crate::{broadcast::Broadcaster, handler::{handle_echo, handle_get, handle_info, handle_ping, handle_psync, handle_replconf, handle_set}, protocol::{self, RObject}, Config};

pub enum HandleResult {
    Subscribed,
    Normal(TcpStream),
}

pub async fn handle(request: &[u8], mut stream: TcpStream, storage: Arc<RwLock<HashMap<String, RObject>>>, config: Arc<RwLock<Config>>, broadcaster: Arc<RwLock<Broadcaster>>) -> Result<HandleResult, Error> {
    
    let str_req = std::str::from_utf8(request)?;

    let (parsed, _) = protocol::RObject::decode(str_req, 0)?;

    // eprintln!("Parsed request: {:#?}", parsed); 

    if let protocol::RObject::Array(a) = parsed {
        let command = match a.get(0)
            .ok_or_else(|| anyhow::anyhow!("Empty array"))? {
                protocol::RObject::SimpleString(s) => s,
                protocol::RObject::BulkString(s) => s,
                _ => bail!("Expected string as command"),
            };
        match command.as_str() {
            "PING" => {
                handle_ping(&mut stream).await?;
            },
            "ECHO" => {
                handle_echo(&a, &mut stream).await?;
            },
            "SET" => {
                handle_set(&a, &mut stream, Arc::clone(&storage)).await?;
                broadcaster.write().await.broadcast(request).await?;
            },
            "GET" => {
                handle_get(&a, &mut stream, Arc::clone(&storage)).await?;
            },
            "INFO" => {
                handle_info(&a, Arc::clone(&config), &mut stream).await?;
            },
            "REPLCONF" => {
                handle_replconf(&a, &mut stream, Arc::clone(&config)).await?;
            },
            "PSYNC" => {
                handle_psync(&a, stream, Arc::clone(&config), Arc::clone(&broadcaster)).await?;
                return Ok(HandleResult::Subscribed);
            },
            _ => bail!("Unknown command: {}", command),
        }
    } else {
        bail!("Expected array as request");
    }

    Ok(HandleResult::Normal(stream))
}