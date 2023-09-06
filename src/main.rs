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
    rocket::build().mount("/", routes![auth, register])
}
