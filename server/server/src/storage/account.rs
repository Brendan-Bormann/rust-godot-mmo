use super::mem_db::MemDB;
use redis::Commands;
use tracing::warn;

pub struct Account {
    username: String,
    password: String,
}

impl MemDB {
    pub fn create_account(&mut self, username: &String, password: &String) -> Result<(), ()> {
        // in-memory account is a temparary measure for testing
        // this is not safe, or even remotely close to acceptable
        // for my purposes, i am building a rough draft MMO and wanted something to test with
        // if i were to ever open this game to the public, I would hash and store pw's in sql
        let mut con = self.get_con();

        if !self.username_is_unique(username) {
            warn!("username {} was not unique", username);
            return Err(());
        }

        let key = format!("account:{}", username);
        let _: () = con.set(key, password).unwrap();
        Ok(())
    }

    pub fn login(&mut self, username: &String, password: &String) -> Result<(), ()> {
        let mut con = self.get_con();
        let key = format!("account:{}", username);
        let db_password: String = con.get(key).unwrap();

        if password == &db_password {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn username_is_unique(&mut self, username: &String) -> bool {
        let account_keys = self.get_all_keys("account");
        warn!("account_keys len {}", account_keys.len());
        !account_keys.contains(&format!("account:{}", username))
    }
}
