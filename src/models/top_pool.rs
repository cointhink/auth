use rocket::serde::Serialize;
use sqlx::types::BigDecimal;

use super::{pool::Pool, reserve::Reserve};

#[derive(Serialize, Debug)]
pub struct TopPool {
    pub pool: Pool,
    pub reserve: Reserve,
    #[serde(serialize_with = "super::reserve::bigdecimal_to_str")]
    pub sum0: BigDecimal,
    #[serde(serialize_with = "super::reserve::bigdecimal_to_str")]
    pub sum1: BigDecimal,
}
