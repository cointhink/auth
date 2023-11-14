use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};

#[derive(Serialize, Debug)]
pub struct Pool {
    pub contract_address: String,
    pub token0: String,
    pub token1: String,
}

impl Pool {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Pool {
        Pool {
            contract_address: row.get::<String, &str>("contract_address"),
            token0: row.get::<String, &str>("token0"),
            token1: row.get::<String, &str>("token1"),
        }
    }
}

pub async fn find_by_address(db: &mut PgConnection, contract_address: &str) -> Option<Pool> {
    match sqlx::query("SELECT * FROM pools WHERE contract_address = $1")
        .bind(contract_address)
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Pool::from_row(&row)),
        Err(_e) => None,
    }
}
