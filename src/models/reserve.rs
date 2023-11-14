use pg_bigdecimal::BigUint;
use rocket::serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Reserve {
    #[serde(serialize_with = "biguint_to_str")]
    pub sum0: BigUint,
    #[serde(serialize_with = "biguint_to_str")]
    pub sum1: BigUint,
}

pub fn biguint_to_str<S>(x: &BigUint, s: S) -> Result<S::Ok, S::Error>
where
    S: rocket::serde::Serializer,
{
    s.serialize_str(&x.to_str_radix(10))
}
