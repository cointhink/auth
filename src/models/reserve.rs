use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};
use sqlx::types::BigDecimal;

#[derive(Serialize, Debug)]
pub struct Reserve {
    pub block_number: u32,
    pub x: String,
    pub y: String,
}

pub fn bigdecimal_to_str<S>(x: &BigDecimal, s: S) -> Result<S::Ok, S::Error>
where
    S: rocket::serde::Serializer,
{
    s.serialize_str(&x.to_string())
}

pub fn optbigdecimal_to_str<S>(x: &Option<BigDecimal>, s: S) -> Result<S::Ok, S::Error>
where
    S: rocket::serde::Serializer,
{
    bigdecimal_to_str(&x.to_owned().unwrap(), s)
}

impl Reserve {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Reserve {
        Reserve {
            block_number: row.get::<i32, &str>("block_number") as u32,
            x: row.get::<String, &str>("x"),
            y: row.get::<String, &str>("y"),
        }
    }
}

pub async fn find_by_address(db: &mut PgConnection, contract_address: &str) -> Option<Reserve> {
    match sqlx::query(
        "SELECT * FROM reserves WHERE contract_address = $1 order by block_number desc limit 1",
    )
    .bind(contract_address)
    .fetch_one(db)
    .await
    {
        Ok(row) => Some(Reserve::from_row(&row)),
        Err(_e) => None,
    }
}
