use rocket::fairing::AdHoc;
use rocket::serde::Deserialize;
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
