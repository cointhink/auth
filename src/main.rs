use crate::account::Account;
use mail_send::{self, mail_builder::MessageBuilder, SmtpClientBuilder};
use rocket::http::{Cookie, CookieJar, Header, Status};
use rocket::response::status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{fairing::AdHoc, serde::Deserialize};
use rocket::{Request, State};
use rocket_db_pools::{Connection, Database};

mod account;
mod sql;

#[macro_use]
extern crate rocket;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    smtp: String,
    site: String,
    from_name: String,
    from_email: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cors<R>(pub R);

impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for Cors<R> {
    #[inline]
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .header(Header::new("Access-Control-Allow-Origin", "*"))
            .ok()
    }
}

#[get("/auth/<token>")]
async fn auth(
    db: Connection<sql::AuthDb>,
    cookies: &CookieJar<'_>,
    token: &str,
) -> Cors<status::Custom<Json<String>>> {
    Cors(match sql::find_by_token(db, token).await {
        Some(account) => {
            let token_cookie = Cookie::build("token", token.to_owned()).finish();
            cookies.add(token_cookie);
            status::Custom(Status::Ok, Json(account.email))
        }
        None => status::Custom(Status::Unauthorized, Json("bad token".to_owned())),
    })
}

#[get("/register/<email>")]
async fn register(
    app_config: &State<AppConfig>,
    db: Connection<sql::AuthDb>,
    email: &str,
) -> String {
    let acct = sql::find_or_create_by_email(db, email).await;
    let body = format!("{}{}", app_config.site, acct.token);
    let email = build_message(&app_config.from_name, &app_config.from_email, &acct, &body);
    send_email(&app_config.smtp, email).await;
    format!("{}", acct.email)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        // .attach(AdHoc::on_liftoff("SQLX Migrate", |build| {
        //     Box::pin(async move {
        //         let db = sql::AuthDb::fetch(&build).unwrap();
        //         let mut conn = db.acquire().await.unwrap();
        //         // https://gist.github.com/hendi/3ff7f988a51125d757095d5fd2a8c216
        //         //<&'c mut sqlx::pool::PoolConnection<DB> as sqlx::Acquire<'c>>
        //         sqlx::migrate!("./sql").run(db);
        //     })
        // }))
        .attach(sql::AuthDb::init())
        .attach(AdHoc::config::<AppConfig>())
        .mount("/", routes![auth, register])
}

fn build_message<'b>(
    from_name: &'b str,
    from_email: &'b str,
    account: &'b Account,
    body: &'b str,
) -> MessageBuilder<'b> {
    MessageBuilder::new()
        .from((from_name, from_email))
        .to(account.email.as_str())
        .subject("Cointhink api token")
        .text_body(body)
}

async fn send_email<'b>(smtp_host: &str, email: MessageBuilder<'b>) {
    println!("smtp {} to {:?}", smtp_host, email);
    SmtpClientBuilder::new(smtp_host, 25)
        .allow_invalid_certs()
        .implicit_tls(false)
        .connect()
        .await
        .unwrap()
        .send(email)
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
        let token = "non-existant-token";
        let response = client.get(format!("/auth/{}", token)).dispatch();
        assert_eq!(response.status(), Status::new(401));
        assert_eq!(response.into_string().unwrap(), "bad token");
    }
}
