use rocket_db_pools::{Connection, Database};
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
    match sql::by_email(db, email).await {
        Some(account) => {
            println!("{:?}", account);
            format!("Hello, {} {:?} ", email, account)
        }
        None => {
            println!("by_email none");
            let account = sql::Account::from_email(email);
            "None".to_string()
        }
    }
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
    fn register() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/register/a@b.c").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn auth() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/auth/abcd1234").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, world!".into()));
    }
}
