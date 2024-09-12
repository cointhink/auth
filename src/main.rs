use crate::models::pool::Pool;
use models::block;
use rocket::http::{Cookie, CookieJar, Header, Status};
use rocket::response::status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{fairing::AdHoc, serde::Deserialize};
use rocket::{Request, State};
use rocket_db_pools::{Connection, Database};

mod email;
mod models;
mod qury;
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
        sql::top_pools(db, &latest_block.number.hours_ago(24), &latest_block.number).await,
    ))
}

#[get("/pools/<pool_id>/since?<price0>&<price1>")]
async fn pools_since(
    db: Connection<sql::AuthDb>,
    pool_id: &str,
    price0: Option<f64>,
    price1: Option<f64>,
) -> Cors<Json<qury::PoolSinceResponse>> {
    Cors(Json(
        qury::pool_price_since(db, pool_id, price0.unwrap_or(0.0), price1.unwrap_or(0.0)).await,
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
    let email = email::build_message(&app_config.from_name, &app_config.from_email, &acct, &url);
    email::send_email(&app_config.smtp, email).await;
    Cors(Json(format!("{}", acct.email)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::on_liftoff("SQLX Migrate", |build| {
            Box::pin(async move {
                let db = sql::AuthDb::fetch(&build).unwrap();
                match sqlx::migrate!("./sql").run(&**db).await {
                    Ok(_) => (),
                    Err(e) => error!("migration error: {}", e),
                }
            })
        }))
        .attach(sql::AuthDb::init())
        .attach(AdHoc::config::<AppConfig>())
        .mount("/", routes![auth, register, pools_top, pools_since])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::serde::json;

    #[test]
    fn register() {
        let email = "a@b.c";
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.post(format!("/register/{}", email)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_json = json::from_str::<String>(&response.into_string().unwrap()).unwrap();
        assert_eq!(body_json, email);
    }

    #[test]
    fn auth() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let token = "non-existant-token";
        let response = client.get(format!("/auth/{}", token)).dispatch();
        assert_eq!(response.status(), Status::new(401));
        assert_eq!(response.into_string().unwrap(), "\"bad token\"");
    }
}
