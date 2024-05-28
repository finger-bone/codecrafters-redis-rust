#[derive(Debug, PartialEq, Eq)]
pub enum ServerRole {
    Master,
    Slave,
}

impl std::fmt::Display for ServerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerRole::Master => write!(f, "master"),
            ServerRole::Slave => write!(f, "slave"),
        }
    }
}

pub struct State {
    pub role: ServerRole,
    pub master_replid: String,
    pub master_repl_offset: usize,
    pub replica_of: Option<String>,
    pub working_port: u64,
    pub consumed: usize,
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
}

pub const BUFFER_SIZE: usize = 128 * 2;