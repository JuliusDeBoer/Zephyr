use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{self, get, routes};
use rocket::{Request, Response};
use serde::Serialize;

// CORS Fairing
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    }
}

#[derive(Serialize)]
struct Message {
    message: String,
}

#[get("/")]
fn hello() -> Json<Message> {
    Json(Message {
        message: String::from("Hello from Zephyr Backend!"),
    })
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/api", routes![hello])
        .launch()
        .await?;

    Ok(())
}
