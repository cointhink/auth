use rocket::{fairing::AdHoc, serde::Deserialize};
use rocket_db_pools::Database;

mod email;
mod models;
mod qury;
mod route;
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(sql::AuthDb::init())
        .attach(sql::migrate())
        .attach(AdHoc::config::<AppConfig>())
        .mount(
            "/",
            routes![
                route::auth,
                route::register,
                route::pools_top,
                route::pools_since
            ],
        )
}
