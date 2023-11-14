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
