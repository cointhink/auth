use crate::sql::query;
use num_traits::cast::ToPrimitive;
use sqlx::types::BigDecimal;
use std::ops::Add;
use std::time::Duration;

use crate::models;
use crate::sql;
use rocket::serde::Serialize;
use rocket_db_pools::sqlx::PgConnection;
use rocket_db_pools::Connection;
use time::macros::format_description;

#[derive(Serialize)]
pub struct PoolSinceResponse {
    block_time: String,
    price: f64,
    cash: f64,
    token0: models::coin::Coin,
    token1: models::coin::Coin,
    swap: models::swap::Swap,
}

pub async fn pool_price_since(
    mut db: Connection<sql::AuthDb>,
    pool_contract_address: &str,
    price0: Option<f64>,
    price1: Option<f64>,
) -> Result<PoolSinceResponse, String> {
    if price0.is_none() && price1.is_none() {
        return Err("bad params both empty".to_owned());
    }
    if price0.is_some() && price1.is_some() {
        return Err("bad params both full".to_owned());
    }

    let pool = models::pool::find_by_address(&mut **db, pool_contract_address)
        .await
        .unwrap();
    let token0 = models::coin::find_by_address(&mut **db, &pool.token0)
        .await
        .unwrap();
    let token1 = models::coin::find_by_address(&mut **db, &pool.token1)
        .await
        .unwrap();

    let mut price: f64 = 0.0;
    let mut decimal_difference = 0;
    let mut direction = true;
    if price0.is_some() && price1.is_none() {
        decimal_difference = token0.decimals - token1.decimals;
        price = price0.unwrap();
        direction = true;
    }
    if price0.is_none() && price1.is_some() {
        decimal_difference = token1.decimals - token0.decimals;
        price = price1.unwrap();
        direction = false;
    }

    let swap_opt = models::swap::swap_price_since(
        &mut **db,
        pool_contract_address,
        direction,
        price,
        decimal_difference,
    )
    .await;
    if let Some((swap_price_eth, swap)) = swap_opt {
        let block = models::block::find_by_number(&mut **db, swap.block_number)
            .await
            .unwrap();
        // let block_timestamp_str = block.timestamp.to_string();
        // let block_time = time::Time::parse(
        //     &block_timestamp_str,
        //     format_description!("[unix_timestamp precision:second]"),
        // ) // error TryFromParsed(InsufficientInformation)
        // .unwrap();
        let elapsed = Duration::from_secs(block.timestamp as u64);
        let utime = time::OffsetDateTime::UNIX_EPOCH;
        let block_time = utime.add(elapsed);
        const USDC_POOL: &str = "b4e16d0168e52d35cacd2c6185b44281ec28c9dc";
        let price_usd = pool_price_at(&mut **db, USDC_POOL, true, swap.block_number).await;
        return Ok(PoolSinceResponse {
            block_time: block_time
                .format(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second]"
                ))
                .unwrap(),
            price: swap_price_eth,
            cash: price_usd.unwrap_or(BigDecimal::from(-1)).to_f64().unwrap(),
            swap,
            token0,
            token1,
        });
    } else {
        return Err("none".to_owned());
    }
}

pub async fn pool_price_at(
    db: &mut PgConnection,
    pool_contract_address: &str,
    direction: bool,
    block_number: u32,
) -> Option<BigDecimal> {
    let sql = "select * from swaps where pool_contract_address = $1 and block_number = $2";
    match query(&sql)
        .bind(pool_contract_address)
        .bind(block_number as i32)
        .fetch_one(db)
        .await
    {
        Ok(row) => {
            let swap = models::swap::Swap::from_row(&row);
            let (numerator, denominator) = if direction {
                if swap.in0.is_some() {
                    (swap.in0.unwrap(), swap.out1.unwrap())
                } else {
                    (swap.out1.unwrap(), swap.in0.unwrap())
                }
            } else {
                if swap.in1.is_some() {
                    (swap.in1.unwrap(), swap.out0.unwrap())
                } else {
                    (swap.out0.unwrap(), swap.in1.unwrap())
                }
            };
            Some(numerator / denominator)
        }

        Err(_) => None,
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn datetime() {
        assert_eq!(1, 1)
    }
}
