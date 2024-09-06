use crate::models;
use crate::sql;
use rocket::serde::Serialize;
use rocket_db_pools::Connection;

#[derive(Serialize)]
pub struct PoolSinceResponse {
    pool_contract_address: String,
    block_number: u32,
}

pub async fn pool_since(
    mut db: Connection<sql::AuthDb>,
    pool_contract_address: &str,
    limit: u32,
) -> PoolSinceResponse {
    let swap = models::swap::swap_price_since(&mut **db, pool_contract_address, limit)
        .await
        .unwrap();
    let block = models::block::find_by_number(&mut **db, swap.block_number);
    return PoolSinceResponse {
        pool_contract_address: pool_contract_address.to_owned(),
        block_number: swap.block_number,
    };
}
