use r2d2::{Pool, PooledConnection};
use redis::{Client, Commands};

#[derive(Clone)]
pub struct MemDB {
    pub con_pool: Pool<Client>,
}

impl MemDB {
    pub fn new(mem_db_addr: String) -> MemDB {
        let mem_db_client = redis::Client::open(mem_db_addr).unwrap();
        let pool = r2d2::Pool::builder().build(mem_db_client).unwrap();
        MemDB { con_pool: pool }
    }

    pub fn get_con(&mut self) -> PooledConnection<Client> {
        self.con_pool.get().unwrap()
    }

    pub fn get_next_id(&mut self, key: &str) -> String {
        let mut con = self.get_con();
        let key = format!("next_key:{}", key);

        let next_id: u32 = con.get(&key).unwrap();
        con.set::<String, u32, u32>(key, next_id + 1).unwrap();

        next_id.to_string()
    }

    pub fn get_all_keys(&mut self, pre: &str) -> Vec<String> {
        let mut con = self.get_con();
        let k = format!("{}:*", pre);
        let collection = con.keys(&k).unwrap();

        collection
    }

    pub fn wipe(&mut self) {
        let mut con = self.get_con();
        let _result: String = redis::cmd("FLUSHALL").query(&mut con).unwrap();
    }
}
