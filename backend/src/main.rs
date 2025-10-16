use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use argon2::Argon2;
use argon2::PasswordHasher;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use diesel::Connection;
use diesel::PgConnection;
use diesel::RunQueryDsl;
use dotenvy::dotenv;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use serde::{self, Deserialize};
use uuid::Uuid;

use crate::models::User;

mod icalendar;
mod models;
mod schema;

#[derive(Deserialize, Debug)]
struct SignUpBody {
    email: String,
    password: String,
    first_name: String,
    affix: Option<String>,
    last_name: String,
}

async fn sign_up(state: Arc<AppState>, body: SignUpBody) -> Result<(), hyper::Error> {
    use crate::schema::users;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = User {
        id: Uuid::new_v4(),
        email: body.email,
        password: password_hash,
        first_name: body.first_name,
        last_name: body.last_name,
        affix: body.affix,
    };

    let mut db = state.db.lock().await;
    diesel::insert_into(users::table)
        .values(&user)
        .execute(&mut *db)
        .unwrap();

    Ok(())
}

async fn handle_request(
    state: Arc<AppState>,
    req: Request<Incoming>,
) -> Result<Response<String>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().path().to_owned();

    println!("Incoming request [{}]: {}", method, uri);

    let collection = req.collect().await.unwrap();
    let bytes = collection.to_bytes();
    let body = str::from_utf8(&bytes).unwrap();

    match (method, uri.as_str()) {
        (Method::POST, "/auth/sign-up") => {
            sign_up(state.clone(), serde_json::from_str(body).unwrap()).await?;
        }
        _ => {
            let mut response = Response::new("".into());
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    }

    Ok(Response::new("Ok".into()))
}

fn connect_to_db() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

struct AppState {
    pub db: Mutex<PgConnection>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let db = connect_to_db();
    let state = Arc::new(AppState { db: Mutex::new(db) });

    println!("Hosting on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let state = state.clone();

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(state.clone(), req)),
                )
                .await
            {
                eprintln!("Error serving: {:?}", err);
            }
        });
    }
}
