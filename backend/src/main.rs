use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::{
    Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use jwt::SignWithKey;
use rand::{Rng, distr::Alphanumeric, rng};
use serde::{self, Deserialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::User;

mod icalendar;
mod models;
mod schema;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignUpBody {
    email: String,
    password: String,
    first_name: String,
    affix: Option<String>,
    last_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    email: String,
    password: String,
}

async fn login(state: Arc<AppState>, body: LoginBody) -> anyhow::Result<String> {
    use crate::schema::users::dsl::*;

    let mut db = state.db.lock().await;
    let user = users
        .select(crate::models::User::as_select())
        .filter(email.eq(body.email))
        .limit(1)
        .load(&mut *db)
        .context("Could not get user from database")?;

    assert!(user.len() <= 1);
    if user.is_empty() {
        // TODO(Julius): Do good user-facing errors
        return Err(anyhow::format_err!("Invalid email or password"));
    }

    let parsed_hash = PasswordHash::new(&user[0].password)?;
    let valid = Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash);

    if valid.is_err() {
        // TODO(Julius): Do good user-facing errors
        Err(anyhow::format_err!("Invalid email or password"))
    } else {
        let key: Hmac<Sha256> = Hmac::new_from_slice(get_jwt_signing_key(&mut db).as_bytes())?;
        let mut claims = BTreeMap::new();
        claims.insert("sub", user[0].id.to_string());
        Ok(claims.sign_with_key(&key)?)
    }
}

async fn sign_up(state: Arc<AppState>, body: SignUpBody) -> anyhow::Result<()> {
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
) -> Result<Response<String>, anyhow::Error> {
    let method = req.method().clone();
    let uri = req.uri().path().to_owned();

    println!("[{}]: {}", method, uri);

    let collection = req.collect().await.unwrap();
    let bytes = collection.to_bytes();
    let body = str::from_utf8(&bytes).unwrap();

    let mut result = None;

    match (method, uri.as_str()) {
        (Method::POST, "/auth/sign-up") => {
            sign_up(state.clone(), serde_json::from_str(body)?).await?;
        }
        (Method::POST, "/auth/login") => {
            result = Some(login(state.clone(), serde_json::from_str(body)?).await?);
        }
        _ => {
            let mut response = Response::new("".into());
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    }

    Ok(Response::new(result.unwrap_or("".into())))
}

fn connect_to_db() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Returns the signing key for authorization tokens. And creates one if needed.
fn get_jwt_signing_key(db: &mut PgConnection) -> String {
    use crate::models::Setting;
    use crate::schema::settings;
    use crate::schema::settings::dsl::*;

    let setting = settings
        .select(crate::models::Setting::as_select())
        .filter(key.eq("AUTH_JWT_SIGNING_KEY"))
        .limit(1)
        .load(db)
        .context("Could not get JWT signing key from database")
        .unwrap();

    if setting.len() == 1 {
        return setting[0].value.clone();
    }

    println!("No JWT signing key present. Generating one...");

    let signing_key: String = rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    diesel::insert_into(settings::table)
        .values(&Setting {
            key: "AUTH_JWT_SIGNING_KEY".into(),
            value: signing_key.clone(),
        })
        .execute(&mut *db)
        .unwrap();

    signing_key
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
