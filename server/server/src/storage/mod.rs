pub mod mem_db;
pub mod session;
pub mod sql_db;

#[derive(Clone)]
pub struct Storage {
    pub mem: mem_db::MemDB,
    pub sql: sql_db::SQLDB,
}

impl Storage {
    pub fn new(mem: mem_db::MemDB, sql: sql_db::SQLDB) -> Self {
        Storage { mem, sql }
    }
}
