use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};
use sqlx::types::BigDecimal;

use super::{
    coin::{self, Coin},
    reserve::{self, Reserve},
};

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
    pub reserve_summary: Option<reserve::Summary>,
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
            reserve_summary: None,
            sum0: None,
            sum0_eth: None,
            sum1: None,
            sum1_eth: None,
            sum_eth: None,
        }
    }

    pub(crate) fn has_cash_token(&self) -> bool {
        self.cash_token_is_0() || self.cash_token_is_1()
    }
    pub(crate) fn cash_token_is_0(&self) -> bool {
        coin::is_cash_token(&self.token0)
    }
    pub(crate) fn cash_token_is_1(&self) -> bool {
        coin::is_cash_token(&self.token1)
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
