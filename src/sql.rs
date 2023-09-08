use postgres::{Client, NoTls};
use rocket_db_pools::{
    sqlx::{self, Postgres, Row},
    Connection, Database,
};

#[derive(Database)]
#[database("auth_db")]
pub struct AuthDb(sqlx::PgPool);

#[derive(Debug)]
pub struct Account {
    id: String,
    email: String,
    token: String,
}

impl Account {
    fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Account {
        Account {
            id: row.get::<String, &str>("id"),
            email: row.get::<String, &str>("email"),
            token: row.get::<String, &str>("token"),
        }
    }

    pub fn from_email(email: &str) -> Account {
        Account {
            id: "uuid".to_string(),
            email: email.to_string(),
            token: "uuid".to_string(),
        }
    }
}

pub fn setup(config: toml::Table) -> Client {
    let db_url = config.get("db_url").unwrap().as_str().unwrap();
    let mut client = Client::connect(db_url, NoTls).unwrap();
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS auth (
            id VARCHAR(36),
            email VARCHAR(256),
            token VARCHAR(36)
        )",
            &[],
        )
        .unwrap();
    client
}
pub async fn by_email(mut db: Connection<AuthDb>, email: &str) -> Option<Account> {
    match sqlx::query("SELECT * FROM auth WHERE email = $1")
        .bind(email)
        .fetch_one(&mut *db)
        .await
    {
        Ok(row) => Some(Account::from_row(&row)),
        Err(_e) => None,
    }
}
