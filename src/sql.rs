use rocket_db_pools::{
    sqlx::{self, Postgres, Row},
    Connection, Database,
};

use crate::models::{
    account::{self, Account},
    block, coin,
    pool::{self, Pool},
    reserve,
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

pub async fn top_pools(
    mut db: Connection<AuthDb>,
    start_block: block::Number,
    stop_block: block::Number,
) -> Vec<Pool> {
    let sql = "select pool_contract_address, sum(in0) as sum_in0, sum(in0_eth) as sum_in0_eth, sum(in1) as sum_in1, sum(in1_eth) as sum_in1_eth, sum(in0_eth + in1_eth) as sum_eth from swaps where block_number > $1 and block_number <= $2 group by pool_contract_address order by sum_eth desc limit 10";
    match sqlx::query(sql)
        .bind::<i32>(start_block.into())
        .bind::<i32>(stop_block.into())
        .fetch_all(&mut **db)
        .await
    {
        Ok(rows) => {
            let mut r = vec![];
            for row in rows {
                let pool_contract_address = row.get("pool_contract_address");
                let mut pool = pool::find_by_address(&mut **db, pool_contract_address)
                    .await
                    .unwrap();
                let reserve = reserve::find_by_address(&mut **db, pool_contract_address)
                    .await
                    .unwrap();
                pool.reserve = Some(reserve);
                pool.sum0 = Some(row.get::<sqlx::types::BigDecimal, &str>("sum_in0"));
                pool.sum0_eth = Some(row.get::<sqlx::types::BigDecimal, &str>("sum_in0_eth"));
                pool.sum1 = Some(row.get::<sqlx::types::BigDecimal, &str>("sum_in1"));
                pool.sum1_eth = Some(row.get::<sqlx::types::BigDecimal, &str>("sum_in1_eth"));
                pool.sum_eth = Some(row.get::<sqlx::types::BigDecimal, &str>("sum_eth"));
                let coin0 = coin::find_by_address(&mut **db, &pool.token0)
                    .await
                    .unwrap();
                pool.coin0 = Some(coin0);
                let coin1 = coin::find_by_address(&mut **db, &pool.token1)
                    .await
                    .unwrap();
                pool.coin1 = Some(coin1);
                r.push(pool)
            }
            r
        }
        Err(_e) => vec![],
    }
}
