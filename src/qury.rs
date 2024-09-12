use std::ops::Add;
use std::time::Duration;

use crate::models;
use crate::sql;
use rocket::serde::Serialize;
use rocket_db_pools::Connection;
use time::macros::format_description;

#[derive(Serialize)]
pub struct PoolSinceResponse {
    block_time: String,
    price: f64,
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
    let pool = models::pool::find_by_address(&mut **db, pool_contract_address)
        .await
        .unwrap();
    let token0 = models::coin::find_by_address(&mut **db, &pool.token0)
        .await
        .unwrap();
    let token1 = models::coin::find_by_address(&mut **db, &pool.token1)
        .await
        .unwrap();
    let swap_opt = models::swap::swap_price_since(
        &mut **db,
        pool_contract_address,
        price0,
        price1,
        token0.decimals,
        token1.decimals,
    )
    .await;
    if let Some(swap) = swap_opt {
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
        return Ok(PoolSinceResponse {
            block_time: block_time
                .format(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second]"
                ))
                .unwrap(),
            price: swap
                .pricef_eth_buy()
                .unwrap_or(swap.pricef_eth_sell().unwrap()),
            swap,
            token0,
            token1,
        });
    } else {
        return Err("none".to_owned());
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn datetime() {
        assert_eq!(1, 1)
    }
}
