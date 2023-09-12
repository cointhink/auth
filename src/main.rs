use mail_send::{self, mail_builder::MessageBuilder, SmtpClientBuilder};
use rocket::{Config, State};
use rocket_db_pools::{Connection, Database};
use serde::Deserialize;
use sql::Account;

mod sql;

#[macro_use]
extern crate rocket;

#[get("/auth/<token>")]
fn auth(token: &str) -> String {
    format!("{}", token)
}

#[get("/register/<email>")]
async fn register(rocket_config: &Config, db: Connection<sql::AuthDb>, email: &str) -> String {
    let account = sql::find_or_create_by_email(db, email).await;
    println!("{:?}", rocket_config);
    send_email("fiizb.gk.donp.org", &account).await;
    format!("{}", account.email)
}

#[derive(Deserialize)]
pub struct AppConfig {}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(sql::AuthDb::init())
        .mount("/", routes![auth, register])
}

async fn send_email(smtp_host: &str, account: &Account) {
    let message = MessageBuilder::new()
        .from(("John Doe", "john@example.com"))
        .to(account.email.as_str())
        .subject("Cointhink api token")
        .text_body(format!("{}", account.token));
    SmtpClientBuilder::new(smtp_host, 25)
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
        let token = "abcd1234";
        let response = client.get(format!("/auth/{}", token)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some(token.into()));
    }
}
