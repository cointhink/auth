use rocket_db_pools::{
    sqlx::{self, Postgres, Row},
    Connection, Database,
};

use crate::models::{
    account::{self, Account},
    pool::{self},
    reserve,
    top_pool::TopPool,
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
        .fetch_one(&mut **db)
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
        .fetch_one(&mut **db)
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
        .execute(&mut **db)
        .await
        .unwrap();
}

pub async fn top_pools(mut db: Connection<AuthDb>) -> Vec<TopPool> {
    let sql = "select pool_contract_address, sum(in0) as sum_in0, sum(in1) as sum_in1 from swaps group by pool_contract_address order by sum_in0 desc limit 10";
    match sqlx::query(sql).fetch_all(&mut **db).await {
        Ok(rows) => {
            let mut r = vec![];
            for row in rows {
                let pool_contract_address = row.get("pool_contract_address");
                let pool = pool::find_by_address(&mut **db, pool_contract_address)
                    .await
                    .unwrap();
                let reserve = reserve::find_by_address(&mut **db, pool_contract_address)
                    .await
                    .unwrap();
                let top_pool = TopPool {
                    pool,
                    reserve,
                    sum0: row.get::<sqlx::types::BigDecimal, &str>("sum_in0"),
                    sum1: row.get::<sqlx::types::BigDecimal, &str>("sum_in1"),
                };
                r.push(top_pool)
            }
            r
        }
        Err(_e) => vec![],
    }
}
