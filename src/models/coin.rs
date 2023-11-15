use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};

#[derive(Serialize, Debug)]
pub struct Coin {
    pub contract_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

impl Coin {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Coin {
        Coin {
            contract_address: row.get::<String, &str>("contract_address"),
            name: row.get::<String, &str>("name"),
            symbol: row.get::<String, &str>("symbol"),
            decimals: row.get::<i32, &str>("decimals"),
        }
    }
}

pub async fn find_by_address(db: &mut PgConnection, contract_address: &str) -> Option<Coin> {
    match sqlx::query("SELECT * FROM coins WHERE contract_address = $1")
        .bind(contract_address)
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Coin::from_row(&row)),
        Err(_e) => None,
    }
}
