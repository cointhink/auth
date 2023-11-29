use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};
use sqlx::types::BigDecimal;

use super::{coin::Coin, reserve::Reserve};

#[derive(Serialize, Debug)]
pub struct Pool {
    pub contract_address: String,
    pub token0: String,
    pub token1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Reserve>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin0: Option<Coin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin1: Option<Coin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count0: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count1: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub sum0: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub sum0_eth: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub sum1: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub sum1_eth: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub sum_eth: Option<BigDecimal>,
}

impl Pool {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Pool {
        Pool {
            contract_address: row.get::<String, &str>("contract_address"),
            token0: row.get::<String, &str>("token0"),
            token1: row.get::<String, &str>("token1"),
            reserve: None,
            coin0: None,
            coin1: None,
            count0: None,
            count1: None,
            sum0: None,
            sum0_eth: None,
            sum1: None,
            sum1_eth: None,
            sum_eth: None,
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
