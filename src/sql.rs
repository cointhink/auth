use rocket_db_pools::{
    sqlx::{self, Postgres, Row},
    Connection, Database,
};

#[derive(Database)]
#[database("auth_db")]
pub struct AuthDb(sqlx::PgPool);

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub token: String,
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

// pub fn setup(config: crate::AppConfig) -> Client {
//     println!("{:?}", config.url);
//     let mut client = Client::connect(&config.url, NoTls).unwrap();
//     client
//         .execute(
//             "CREATE TABLE IF NOT EXISTS auth (
//             id VARCHAR(36),
//             email VARCHAR(256),
//             token VARCHAR(36)
//         )",
//             &[],
//         )
//         .unwrap();
//     client
// }

pub async fn find_or_create_by_email(mut db: Connection<AuthDb>, email: &str) -> Account {
    match sqlx::query("SELECT * FROM auth WHERE email = $1")
        .bind(email)
        .fetch_one(&mut *db)
        .await
    {
        Ok(row) => Account::from_row(&row),
        Err(_e) => {
            let account = Account::from_email(email);
            insert(db, &account).await;
            account
        }
    }
}

pub async fn find_by_token(mut db: Connection<AuthDb>, token: &str) -> Option<Account> {
    match sqlx::query("SELECT * FROM auth WHERE token = $1")
        .bind(token)
        .fetch_one(&mut *db)
        .await
    {
        Ok(row) => Some(Account::from_row(&row)),
        Err(_e) => None,
    }
}

pub async fn insert(mut db: Connection<AuthDb>, account: &Account) {
    sqlx::query("INSERT INTO auth values ($1, $2, $3)")
        .bind(account.id.as_str())
        .bind(account.email.as_str())
        .bind(account.token.as_str())
        .execute(&mut *db)
        .await
        .unwrap();
}
