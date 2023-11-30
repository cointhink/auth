use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};
use sqlx::types::BigDecimal;

use super::block;

#[derive(Serialize, Debug)]
pub struct Reserve {
    pub block_number: u32,
    pub x: String,
    pub y: String,
}

#[derive(Serialize, Debug)]
pub struct Summary {
    pub start_block_number: block::Number,
    pub stop_block_number: block::Number,
    pub count: u64,
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub stddev: Option<BigDecimal>,
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub min: Option<BigDecimal>,
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub max: Option<BigDecimal>,
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
    match x {
        Some(x) => bigdecimal_to_str(&x, s),
        None => s.serialize_none(),
    }
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

pub async fn summarize(
    db: &mut PgConnection,
    contract_address: &str,
    start_block_number: &block::Number,
    stop_block_number: &block::Number,
) -> Summary {
    let row = sqlx::query(
        "SELECT stddev_pop(x::numeric), count(x), min(x::numeric), max(x::numeric)  FROM reserves WHERE contract_address = $1 and block_number > $2 and block_number <= $3",
    )
    .bind(contract_address)
    .bind::<i32>(start_block_number.into())
    .bind::<i32>(stop_block_number.into())
    .fetch_one(db)
    .await.unwrap();
    return Summary {
        start_block_number: start_block_number.clone(),
        stop_block_number: stop_block_number.clone(),
        stddev: row.get::<Option<sqlx::types::BigDecimal>, &str>("stddev_pop"),
        count: row.get::<i64, &str>("count") as u64,
        min: row.get::<Option<sqlx::types::BigDecimal>, &str>("min"),
        max: row.get::<Option<sqlx::types::BigDecimal>, &str>("max"),
    };
}
