use mail_send::{self, mail_builder::MessageBuilder, SmtpClientBuilder};
use rocket_db_pools::{Connection, Database};
use sql::Account;
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
async fn register(db: Connection<sql::AuthDb>, email: &str) -> String {
    let account = sql::find_or_create_by_email(db, email).await;
    send_email(&account).await;
    format!("{}", account.email)
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

async fn send_email(account: &Account) {
    let message = MessageBuilder::new()
        .from(("John Doe", "john@example.com"))
        .to(account.email.as_str())
        .subject("Cointhink api token")
        .text_body(format!("{}", account.token));
    SmtpClientBuilder::new("localhost", 25)
        .implicit_tls(false)
        .connect()
        .await
        .unwrap()
        .send(message)
        .await
        .unwrap();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn register() {
        let email = "a@b.c";
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(format!("/register/{}", email)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert_eq!(body, email.to_string());
    }

    #[test]
    fn auth() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/auth/abcd1234").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, world!".into()));
    }
}
