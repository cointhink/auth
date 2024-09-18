use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Data, Request, Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

pub struct Timer {
    start_ms: AtomicU64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start_ms: AtomicU64::new(0),
        }
    }
}

#[rocket::async_trait]
impl Fairing for Timer {
    fn info(&self) -> Info {
        Info {
            name: "Timer",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, _request: &mut Request<'_>, _: &mut Data<'_>) {
        self.start_ms.store(unixtime_ms() as u64, Ordering::Relaxed)
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let duration_ms = unixtime_ms() as u64 - self.start_ms.load(Ordering::Relaxed);
        response.set_header(Header {
            name: "X-Duration".into(),
            value: mmssms(duration_ms).into(),
        });
    }
}

pub fn mmssms(total_ms: u64) -> String {
    let mins = total_ms / 60 / 1000;
    let secs = (total_ms / 1000) % 60;
    let ms = total_ms % 1000;
    format!("{}m {}s {}ms", mins, secs, ms)
}

pub fn unixtime_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
