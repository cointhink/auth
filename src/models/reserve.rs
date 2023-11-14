use num_traits::Num;
use pg_bigdecimal::BigUint;
use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};

#[derive(Serialize, Debug)]
pub struct Reserve {
    pub block_number: u32,
    #[serde(serialize_with = "biguint_to_str")]
    pub x: BigUint,
    #[serde(serialize_with = "biguint_to_str")]
    pub y: BigUint,
}

pub fn biguint_to_str<S>(x: &BigUint, s: S) -> Result<S::Ok, S::Error>
where
    S: rocket::serde::Serializer,
{
    s.serialize_str(&x.to_str_radix(10))
}

impl Reserve {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Reserve {
        Reserve {
            block_number: row.get::<i32, &str>("block_number") as u32,
            x: BigUint::from_str_radix(row.get::<&str, &str>("x"), 10).unwrap(),
            y: BigUint::from_str_radix(row.get::<&str, &str>("y"), 10).unwrap(),
        }
    }
}

pub async fn find_by_address(db: &mut PgConnection, contract_address: &str) -> Option<Reserve> {
    match sqlx::query("SELECT * FROM reserves WHERE contract_address = $1")
        .bind(contract_address)
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Reserve::from_row(&row)),
        Err(_e) => None,
    }
}
