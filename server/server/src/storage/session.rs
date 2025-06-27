use super::mem_db::MemDB;
use bitcode::{Decode, Encode};
use redis::Commands;

#[derive(Encode, Decode)]
pub struct SessionData {
    pub auth_token: String,
    pub account_id: String,
}

impl MemDB {
    pub fn open_session(
        &mut self,
        addr: &str,
        auth_token: &String,
        account_id: String,
    ) -> Result<(), String> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let session_data = SessionData {
            auth_token: auth_token.clone(),
            account_id,
        };

        let _: () = con.set(key, bitcode::encode(&session_data)).unwrap();
        Ok(())
    }

    pub fn find_session(&mut self, addr: &str) -> Result<Option<SessionData>, ()> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let value: Option<Vec<u8>> = con.get(key).expect("Failed to enter session");

        if value.is_none() {
            return Ok(None);
        }

        let session_data: SessionData = bitcode::decode(&value.unwrap()).unwrap();
        Ok(Some(session_data))
    }

    pub fn consume_session(&mut self, addr: &str) -> Result<(), ()> {
        self.close_session(addr)
    }

    pub fn close_session(&mut self, addr: &str) -> Result<(), ()> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let _: () = con.del(key).unwrap();
        Ok(())
    }
}
