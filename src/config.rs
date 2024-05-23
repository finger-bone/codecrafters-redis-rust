pub struct Config {
    pub role: String,
    pub master_replid: String,
    pub master_repl_offset: usize,
    pub replica_of: String,
    pub working_port: u64,
}

pub const BUFFER_SIZE: usize = 1024 * 4;