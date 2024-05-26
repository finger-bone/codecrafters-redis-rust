use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{broadcast::Broadcaster, protocol::RObject, Config};

pub async fn handle_wait(
    _args: &Vec<RObject>, 
    stream: &mut TcpStream, 
    _storage: Arc<RwLock<HashMap<String, RObject>>>, 
    config: Arc<RwLock<Config>>,
    broadcaster: Arc<RwLock<Broadcaster>>,
) -> Result<(), Error> { 
    stream.write(
        RObject::Integer(
            broadcaster.write().await.check_sync(
                config.read().await.consumed
            ).await.expect(
                "Failed to check sync status in wait handler."
            ) as i64
        ).to_string().as_bytes()
    ).await.expect("Failed to write to stream handling wait.");
    Ok(())
}