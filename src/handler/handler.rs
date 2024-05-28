use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Error};
use tokio::{net::TcpStream, sync::RwLock};

use crate::{broadcast::Broadcaster, handler::{handle_echo, handle_config, handle_get, handle_info, handle_ping, handle_psync, handle_replconf, handle_set, handle_wait}, protocol::{self, RObject}, State};

pub enum HandleResult {
    Subscribed,
    Normal(TcpStream),
}

pub async fn handle(request: &[u8], mut stream: TcpStream, storage: Arc<RwLock<HashMap<String, RObject>>>, state: Arc<RwLock<State>>, broadcaster: Arc<RwLock<Broadcaster>>) -> Result<HandleResult, Error> {
    
    let str_req = String::from_utf8_lossy(request).to_string();

    let mut start = 0;

    while start < str_req.len() {
        let (parsed, consumed) = protocol::RObject::decode(&str_req, start)?;

        if let protocol::RObject::Array(a) = parsed {
            let command = match a.get(0)
                .ok_or_else(|| anyhow::anyhow!("Empty array"))? {
                    protocol::RObject::SimpleString(s) => s,
                    protocol::RObject::BulkString(s) => s,
                    _ => bail!("Expected string as command"),
                };
            match command.as_str() {
                "PING" => {
                    handle_ping(&mut stream, Arc::clone(&state)).await?;
                },
                "ECHO" => {
                    handle_echo(&a, &mut stream).await?;
                },
                "SET" => {
                    let future = handle_set(&a, &mut stream, Arc::clone(&storage), Arc::clone(&state));
                    broadcaster.write().await.broadcast(&request[start..consumed]).await?;
                    let _ = future.await;
                },
                "GET" => {
                    handle_get(&a, &mut stream, Arc::clone(&storage)).await?;
                },
                "INFO" => {
                    handle_info(&a, Arc::clone(&state), &mut stream).await?;
                },
                "REPLCONF" => {
                    handle_replconf(&a, &mut stream, Arc::clone(&state)).await?;
                },
                "PSYNC" => {
                    handle_psync(&a, stream, Arc::clone(&state), Arc::clone(&broadcaster)).await?;
                    return Ok(HandleResult::Subscribed);
                },
                "WAIT" => {
                    handle_wait(&a, &mut stream, Arc::clone(&storage), Arc::clone(&state), Arc::clone(&broadcaster)).await?;
                },
                "CONFIG" => {
                    handle_config(&a, &mut stream, Arc::clone(&state)).await?;
                },
                _ => bail!("Unknown command: {}", command),
            }
        } else {
            bail!("Expected array as request");
        }
        state.write().await.consumed += consumed - start;    
        start = consumed;
    }

    Ok(HandleResult::Normal(stream))
}