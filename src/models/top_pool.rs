use pg_bigdecimal::BigUint;
use rocket::serde::Serialize;

use super::{pool::Pool, reserve::Reserve};

#[derive(Serialize, Debug)]
pub struct TopPool {
    pub pool: Pool,
    pub reserve: Reserve,
    #[serde(serialize_with = "super::reserve::biguint_to_str")]
    pub sum0: BigUint,
    #[serde(serialize_with = "super::reserve::biguint_to_str")]
    pub sum1: BigUint,
}

pub fn new(pool: Pool, reserve: Reserve, sum0: BigUint, sum1: BigUint) {
    TopPool {
        pool,
        reserve,
        sum0,
        sum1,
    };
}
