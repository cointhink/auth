use rocket_db_pools::{
    sqlx::{self, Row},
    Connection, Database,
};
use std::fs;
use toml;

mod sql;

#[macro_use]
extern crate rocket;

#[get("/auth/<token>")]
fn auth(token: &str) -> String {
    format!("Hello, {} ", token)
}

#[get("/register/<email>")]
async fn register(mut db: Connection<sql::AuthDb>, email: &str) -> String {
    let row = sqlx::query("SELECT content FROM auth WHERE id = ?")
        .bind(1)
        .fetch_one(&mut *db)
        .await
        .and_then(|r| Ok(r.try_get(0)?))
        .ok();
    format!("Hello, {} {:?} ", email, row)
}

#[launch]
fn rocket() -> _ {
    let toml = fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str(&toml).unwrap();
    sql::setup(config);
    rocket::build()
        .attach(sql::AuthDb::init())
        .mount("/", routes![auth, register])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/auth/abcd1234").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, world!".into()));
    }
}
