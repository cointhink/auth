use std::collections::HashMap;

use crate::models::account::Account;
use crate::models::pool::Pool;
use handlebars::Handlebars;
use mail_send::{self, mail_builder::MessageBuilder, SmtpClientBuilder};
use models::block;
use rocket::http::{Cookie, CookieJar, Header, Status};
use rocket::response::status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{fairing::AdHoc, serde::Deserialize};
use rocket::{Request, State};
use rocket_db_pools::{Connection, Database};
use sql::top_pools;

mod models;
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

#[get("/pools/top")]
async fn pools_top(mut db: Connection<sql::AuthDb>) -> Cors<Json<Vec<Pool>>> {
    let latest_block = block::find_latest(&mut db).await.unwrap();
    Cors(Json(
        top_pools(db, latest_block.number.hours_ago(24), latest_block.number).await,
    ))
}

#[get("/auth/<token>")]
async fn auth(
    db: Connection<sql::AuthDb>,
    cookies: &CookieJar<'_>,
    token: &str,
) -> Cors<status::Custom<Json<String>>> {
    Cors(match sql::find_by_token(db, token).await {
        Some(account) => {
            let token_cookie = Cookie::build(("token", token.to_owned()));
            cookies.add(token_cookie);
            status::Custom(Status::Ok, Json(account.email))
        }
        None => status::Custom(Status::Unauthorized, Json("bad token".to_owned())),
    })
}

#[post("/register/<email>")]
async fn register(
    app_config: &State<AppConfig>,
    db: Connection<sql::AuthDb>,
    email: &str,
) -> Cors<Json<String>> {
    let acct = sql::find_or_create_by_email(db, email).await;
    let url = format!("{}{}", app_config.site, acct.token);
    let email = build_message(&app_config.from_name, &app_config.from_email, &acct, &url);
    send_email(&app_config.smtp, email).await;
    Cors(Json(format!("{}", acct.email)))
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
        .mount("/", routes![auth, register, pools_top])
}

fn build_message<'b>(
    from_name: &'b str,
    from_email: &'b str,
    account: &'b Account,
    url: &'b str,
) -> MessageBuilder<'b> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("register_body", "emails/register_body.hbs")
        .unwrap();
    let mut data = HashMap::new();
    data.insert("url", url);
    let body = handlebars.render("register_body", &data).unwrap();

    handlebars
        .register_template_file("register_subject", "emails/register_subject.hbs")
        .unwrap();
    let data: HashMap<&str, &str> = HashMap::new();
    let subject = handlebars.render("register_subject", &data).unwrap();

    MessageBuilder::new()
        .from((from_name, from_email))
        .to(account.email.as_str())
        .subject(subject)
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
