use std::{collections::HashMap, sync::{Arc, RwLock}};

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::protocol::RObject;

pub async fn handle_set(args: &Vec<RObject>, stream: &mut TcpStream, storage: &Arc<RwLock<HashMap<String, RObject>>>) -> Result<(), Error> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("SET requires at least 2 arguments"));
    }

    let key = match &args[1] {
        RObject::BulkString(s) => s,
        _ => return Err(anyhow::anyhow!("Expected BulkString as key")),
    };

    let value = args[2].clone();

    storage.write().expect(
        "failed to acquire write lock handling SET"
    ).insert(key.clone(), value);

    stream.write(
        RObject::SimpleString("OK".to_string()).to_string().as_bytes()
    ).await.expect(
        "failed to write response to stream handling SET"
    );

    Ok(())
}