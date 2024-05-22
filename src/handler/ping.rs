use anyhow::Error;

use crate::protocol::RObject;

pub fn handle_ping() -> Result<RObject, Error> {
    Ok(RObject::SimpleString("PONG".to_string()))
}