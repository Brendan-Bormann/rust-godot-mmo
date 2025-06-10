use super::mem_db::MemDB;
use redis::Commands;

impl MemDB {
    pub fn create_session(&mut self, addr: &str, username: &String) -> Result<(), ()> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let _: () = con.set(key, username).unwrap();
        Ok(())
    }

    // pub fn update_session(&mut self, session: &Session) -> Result<(), ()> {
    //     let key = format!("session:{}", session.peer);
    //     let encoded = bitcode::encode(session);
    //     let _: () = self.con.set(key, encoded).unwrap();
    //     Ok(())
    // }

    pub fn find_session(&mut self, addr: &str) -> Result<Option<String>, ()> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let value = con.get(key).unwrap();
        Ok(Some(value))
    }

    pub fn delete_session(&mut self, addr: &str) -> Result<(), ()> {
        let mut con = self.get_con();
        let key = format!("session:{}", addr);
        let _: () = con.del(key).unwrap();
        Ok(())
    }

    // pub fn find_all_sessions(&mut self) -> Vec<Session> {
    //     let session_keys = self.get_all_keys("session");
    //     let mut sessions: Vec<Session> = vec![];

    //     for session_key in session_keys {
    //         let encoded: Vec<_> = self.con.get(session_key).unwrap();
    //         let session = bitcode::decode(&encoded).expect("Decoding failed");
    //         sessions.push(session);
    //     }

    //     sessions
    // }
}
