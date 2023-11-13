use rocket_db_pools::{
    sqlx::{self, Postgres, Row},
    Connection, Database,
};

use crate::models::{
    account::{self, Account},
    pool::{self, Pool},
};

#[derive(Database)]
#[database("auth_db")]
pub struct AuthDb(sqlx::PgPool);

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
            id: account::get_nice_rand_str(),
            email: email.to_string(),
            token: account::get_nice_rand_str(),
        }
    }
}

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

pub async fn top_pools(mut db: Connection<AuthDb>) -> Vec<Pool> {
    let sql = "select pool_contract_address, sum(in0) from swaps group by pool_contract_address order by sum desc";
    match sqlx::query(sql).fetch_all(&mut *db).await {
        Ok(rows) => rows
            .iter()
            .map(|r| Pool {
                contract_address: "same".to_string(),
                token0: "same".to_string(),
                token1: "same".to_string(),
            })
            .collect(),
        Err(_e) => vec![],
    }
}
