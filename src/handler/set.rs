use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{state::ServerRole, protocol::RObject, State};

pub async fn handle_set(args: &Vec<RObject>, stream: &mut TcpStream, storage: Arc<RwLock<HashMap<String, RObject>>>, state: Arc<RwLock<State>>) -> Result<(), Error> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("SET requires at least 2 arguments"));
    }

    let key = match &args[1] {
        RObject::BulkString(s) => s,
        _ => return Err(anyhow::anyhow!("Expected BulkString as key")),
    };

    let value = args[2].clone();

    storage.write().await.insert(key.clone(), value);
    

    if args.len() >= 5 {
        if let RObject::BulkString(s) = &args[3] {
            if s.to_lowercase() == "px" {
                if let RObject::BulkString(i) = &args[4] {
                    let i = i.parse::<u64>()?;
                    let storage = Arc::clone(&storage);
                    let key = key.clone();
                    eprintln!("Scheduling expiration in {}ms for key {}", i, key);
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(i)).await;
                        expire(key, storage).await;
                    });
                }
            }
        }
    }

    if state.read().await.role == ServerRole::Master {
        stream.write(
            RObject::SimpleString("OK".to_string()).to_string().as_bytes()
        ).await.expect(
            "failed to write response to stream handling SET"
        );
    }

    Ok(())
}

async fn expire(key: String, storage: Arc<RwLock<HashMap<String, RObject>>>) {
    eprintln!("Expiring key: {}", key);
    let mut storage = storage.write().await;
    storage.remove(&key);
}