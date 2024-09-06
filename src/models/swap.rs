use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};
use sqlx::types::BigDecimal;

use crate::sql::query;

#[derive(Serialize, Debug)]
pub struct Swap {
    pub pool_contract_address: String,
    pub block_number: u32,
    pub transaction_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub in0_eth: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub in1_eth: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub in0: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub in1: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub out0: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "super::reserve::optbigdecimal_to_str")]
    pub out1: Option<BigDecimal>,
}

impl Swap {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Swap {
        Swap {
            pool_contract_address: row.get::<String, &str>("pool_contract_address"),
            block_number: row.get::<i32, &str>("block_number") as u32,
            transaction_index: row.get::<i32, &str>("transaction_index") as u32,
            in0_eth: row.get("in0_eth"),
            in1_eth: row.get("in1_eth"),
            in0: row.get("in0"),
            in1: row.get("in1"),
            out0: row.get("out0"),
            out1: row.get("out1"),
        }
    }
}

pub async fn swap_price_since(
    db: &mut PgConnection,
    pool_contract_address: &str,
    limit: u32,
) -> Option<Swap> {
    let sql = "select * from swaps where pool_contract_address = $1 and in0 / in0_eth < $2 limit 1";
    match query(sql)
        .bind(pool_contract_address)
        .bind(limit as i32)
        .fetch_all(db)
        .await
    {
        Ok(rows) => {
            if rows.len() == 1 {
                let row = rows.first().unwrap();
                Some(Swap::from_row(row))
            } else {
                None
            }
        }
        Err(_e) => None,
    }
}
