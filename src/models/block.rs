use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, PgConnection, Postgres, Row};

#[derive(Debug, Serialize)]
pub struct Block {
    pub hash: String,
    pub number: u32,
    pub timestamp: u32,
}
impl Block {
    pub fn from_row(row: &<Postgres as rocket_db_pools::sqlx::Database>::Row) -> Block {
        Block {
            hash: row.get::<String, &str>("hash"),
            number: row.get::<i32, &str>("number") as u32,
            timestamp: row.get::<i32, &str>("timestamp") as u32,
        }
    }
}

pub async fn find_by_number(db: &mut PgConnection, number: u32) -> Option<Block> {
    match sqlx::query("SELECT * FROM blocks WHERE number = $1")
        .bind(number as i32)
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Block::from_row(&row)),
        Err(_e) => None,
    }
}

pub async fn find_by_timestamp(db: &mut PgConnection, timestamp: u32) -> Option<Block> {
    match sqlx::query("SELECT * FROM blocks WHERE timestamp >= $1 order by timestamp asc limit 1")
        .bind(timestamp as i32)
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Block::from_row(&row)),
        Err(_e) => None,
    }
}

pub async fn find_latest(db: &mut PgConnection) -> Option<Block> {
    match sqlx::query("SELECT * FROM blocks order by number desc limit 1")
        .fetch_one(db)
        .await
    {
        Ok(row) => Some(Block::from_row(&row)),
        Err(_e) => None,
    }
}
