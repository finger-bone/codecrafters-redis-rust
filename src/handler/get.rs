use std::{collections::HashMap, sync::{Arc, RwLock}};

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::protocol::RObject;

pub async fn handle_get(args: &Vec<RObject>, stream: &mut TcpStream, storage: &Arc<RwLock<HashMap<String, RObject>>>) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("GET requires at least 1 argument"));
    }

    let key = match &args[1] {
        RObject::BulkString(s) => s,
        _ => return Err(anyhow::anyhow!("Expected BulkString as key")),
    };

    let storage = storage.read().expect(
        "failed to acquire read lock handling GET"
    );

    let value = storage.get(key).cloned().unwrap_or(RObject::Null);

    drop(storage);

    stream.write(
        value.to_string().as_bytes()
    ).await.expect(
        "failed to write response to stream handling GET"
    );

    Ok(())
}