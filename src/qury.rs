use std::ops::Add;
use std::time::Duration;

use crate::models;
use crate::sql;
use rocket::serde::Serialize;
use rocket_db_pools::Connection;
use time::macros::format_description;

#[derive(Serialize)]
pub struct PoolSinceResponse {
    pool_contract_address: String,
    block_time: String,
}

pub async fn pool_price_since(
    mut db: Connection<sql::AuthDb>,
    pool_contract_address: &str,
    limit: u32,
) -> PoolSinceResponse {
    let swap = models::swap::swap_price_since(&mut **db, pool_contract_address, limit)
        .await
        .unwrap();
    let block = models::block::find_by_number(&mut **db, swap.block_number)
        .await
        .unwrap();
    // let block_timestamp_str = block.timestamp.to_string();
    // let block_time = time::Time::parse(
    //     &block_timestamp_str,
    //     format_description!("[unix_timestamp precision:second]"),
    // )
    // .unwrap();
    let elapsed = Duration::from_secs(block.timestamp as u64);
    let utime = time::OffsetDateTime::UNIX_EPOCH;
    let block_time = utime.add(elapsed);

    return PoolSinceResponse {
        pool_contract_address: pool_contract_address.to_owned(),
        block_time: block_time
            .format(format_description!("[hour]:[minute]:[second]"))
            .unwrap(),
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn datetime() {
        assert_eq!(1, 1)
    }
}
