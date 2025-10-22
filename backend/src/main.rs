use crate::entity::prelude::{InstanceSetting, User};
use crate::entity::{instance_setting, user};
use actix_web::{App, HttpResponse, HttpServer, post, web};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use rand::{Rng, distr::Alphanumeric, rng};
use sea_orm::entity::*;
use sea_orm::{ColumnTrait, QueryFilter};
use sea_orm::{Database, DatabaseConnection};
use serde::{self, Deserialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;
use uuid::Uuid;

mod entity;
mod icalendar;

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

#[post("/auth/login")]
async fn login(db: web::Data<DatabaseConnection>, body: web::Json<LoginBody>) -> HttpResponse {
    let user_result = User::find()
        .filter(user::Column::Email.eq(body.email.clone()))
        .one(db.as_ref())
        .await
        .expect("Could not query DB");

    if user_result.is_none() {
        return HttpResponse::BadRequest().await.unwrap();
    }

    let user = user_result.unwrap();

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    let valid = Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash);

    if valid.is_err() {
        HttpResponse::BadRequest().await.unwrap()
    } else {
        let key: Hmac<Sha256> =
            Hmac::new_from_slice(get_jwt_signing_key(&db).await.as_bytes()).unwrap();
        let mut claims = BTreeMap::new();
        claims.insert("sub", user.id.to_string());
        HttpResponse::Ok().body(claims.sign_with_key(&key).unwrap())
    }
}

#[post("/auth/sign-up")]
async fn sign_up(body: web::Json<SignUpBody>, conn: web::Data<DatabaseConnection>) -> HttpResponse {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(body.email.clone()),
        password: Set(password_hash),
        first_name: Set(body.first_name.clone()),
        last_name: Set(body.last_name.clone()),
        affix: Set(body.affix.clone()),
    };

    let _ = user.insert(conn.as_ref()).await;

    HttpResponse::Created()
        .await
        .expect("Could not create response")
}

fn get_db_string() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

/// Returns the signing key for authorization tokens. And creates one if needed.
async fn get_jwt_signing_key(db: &DatabaseConnection) -> String {
    let setting = InstanceSetting::find()
        .filter(instance_setting::Column::Key.eq("AUTH_JWT_SIGNING_KEY"))
        .one(db)
        .await
        .expect("Could not query database");

    if let Some(setting) = setting {
        return setting.value.clone();
    }

    println!("No JWT signing key present. Generating one...");

    let signing_key: String = rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let new_setting = instance_setting::ActiveModel {
        key: Set("AUTH_JWT_SIGNING_KEY".to_string()),
        value: Set(signing_key.clone()),
    };

    let _ = new_setting.insert(db).await;

    signing_key
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = Database::connect(get_db_string())
        .await
        .expect("Could not connect to database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .service(sign_up)
            .service(login)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
