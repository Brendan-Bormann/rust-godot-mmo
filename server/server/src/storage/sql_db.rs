use password_auth::{generate_hash, verify_password};
use sqlx::Row;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

#[derive(Clone)]
pub struct SQLDB {
    pool: PgPool,
}

impl SQLDB {
    pub async fn new(database_url: String) -> SQLDB {
        let pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("Failed to connect to db");

        let sql_db = SQLDB { pool };

        sql_db
            .init_schema()
            .await
            .expect("Failed to init db schema");

        sql_db
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DROP TABLE IF EXISTS accounts CASCADE
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id SERIAL PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            DROP TABLE IF EXISTS characters CASCADE
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS characters (
                id SERIAL PRIMARY KEY,
                account_id INTEGER REFERENCES accounts(id),
                name TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_account(&self, email: &str, password: &str) -> Result<bool, sqlx::Error> {
        let password_hash = generate_hash(password);

        let result = sqlx::query("INSERT INTO accounts (email, password_hash) VALUES ($1, $2)")
            .bind(email)
            .bind(password_hash)
            .execute(&self.pool)
            .await;

        Ok(result.is_ok())
    }

    pub async fn verify_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(String, bool), sqlx::Error> {
        let row = sqlx::query("SELECT * FROM accounts WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await;

        let Ok(Some(row)) = row else {
            return Ok(("".into(), false));
        };

        let hash: String = row.try_get("password_hash").unwrap();
        let account_id: i32 = row.try_get("id").unwrap();

        let correct_password = verify_password(password, hash.as_str());

        Ok((account_id.to_string(), correct_password.is_ok()))
    }

    pub async fn create_character(
        &self,
        account_id: &str,
        character_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO characters (account_id, name) VALUES ($1, $2)")
            .bind(account_id)
            .bind(character_name)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
}
