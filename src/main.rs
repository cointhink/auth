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
fn register(email: &str) -> String {
    format!("Hello, {} ", email)
}

#[launch]
fn rocket() -> _ {
    let toml = fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str(&toml).unwrap();
    sql::setup(config);
    rocket::build().mount("/", routes![auth, register])
}
