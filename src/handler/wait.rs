use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{protocol::RObject, Config};

pub async fn handle_wait(
    _args: &Vec<RObject>, 
    stream: &mut TcpStream, 
    _storage: Arc<RwLock<HashMap<String, RObject>>>, 
    _config: Arc<RwLock<Config>>,
) -> Result<(), Error> { 
    stream.write(
        RObject::Integer(0).to_string().as_bytes()
    ).await.expect("Failed to write to stream handling wait.");
    Ok(())
}