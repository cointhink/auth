use num_traits::ToPrimitive;
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

    pub fn pricef_eth_buy(&self) -> Option<f64> {
        div(self.in0_eth.clone(), self.out1.clone())
    }

    pub fn pricef_eth_sell(&self) -> Option<f64> {
        div(self.in1_eth.clone(), self.out0.clone())
    }
}

fn div(ine: Option<BigDecimal>, out: Option<BigDecimal>) -> Option<f64> {
    match out {
        Some(out_some) => {
            let flt = ine.unwrap().to_f64().unwrap() / out_some.to_f64().unwrap();
            Some(flt)
        }
        None => None,
    }
}

pub async fn swap_price_since(
    db: &mut PgConnection,
    pool_contract_address: &str,
    price0: Option<f64>,
    price1: Option<f64>,
    out0_decimals: i32,
    out1_decimals: i32,
) -> Option<Swap> {
    if price0.is_none() && price1.is_none() {
        return None;
    }
    if price0.is_some() && price1.is_some() {
        return None;
    }

    let mut sql: &str = "";
    let mut decimal_difference = 0;
    let mut price: f64 = 0.0;
    if price0.is_some() && price1.is_none() {
        sql = "select *, in0_eth / (out1 * power(10,$2)) as price_eth from swaps where pool_contract_address = $1 and out1 > 0 and in0_eth / (out1 * power(10, $2)) < $3 order by block_number desc limit 1";
        decimal_difference = 18 - out1_decimals;
        price = price0.unwrap();
    }
    if price0.is_none() && price1.is_some() {
        sql = "select *, in1_eth / (out0 * power(10,$2)) as price_eth from swaps where pool_contract_address = $1 and out0 > 0 and in1_eth / (out0 * power(10, $2)) < $3 order by block_number desc limit 1";
        decimal_difference = 18 - out0_decimals;
        price = price1.unwrap();
    }
    match query(sql)
        .bind(pool_contract_address)
        .bind(decimal_difference)
        .bind(price)
        .fetch_optional(db)
        .await
    {
        Ok(row_opt) => match row_opt {
            Some(row) => Some(Swap::from_row(&row)),
            None => None,
        },
        Err(_e) => None,
    }
}
