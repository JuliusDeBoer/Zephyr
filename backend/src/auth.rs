use std::collections::BTreeMap;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::post;
use actix_web::web;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use eyre::Context;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use rand::{Rng, distr::Alphanumeric, rng};
use sea_orm::DatabaseConnection;
use sea_orm::entity::*;
use sea_orm::{ColumnTrait, QueryFilter};
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use uuid::Uuid;

use crate::entity::prelude::{InstanceSetting, User};
use crate::entity::{instance_setting, user};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SignUpBody {
    email: String,
    password: String,
    first_name: String,
    affix: Option<String>,
    last_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    email: String,
    password: String,
}

#[post("/login")]
async fn login(db: web::Data<Arc<DatabaseConnection>>, body: web::Json<LoginBody>) -> HttpResponse {
    let db = db.as_ref().as_ref();

    let user_result = User::find()
        .filter(user::Column::Email.eq(body.email.clone()))
        .one(db)
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
            Hmac::new_from_slice(get_jwt_signing_key(db).await.unwrap().as_bytes()).unwrap();
        let mut claims = BTreeMap::new();
        claims.insert("sub", user.id.to_string());
        HttpResponse::Ok().body(claims.sign_with_key(&key).unwrap())
    }
}

#[post("/sign-up")]
async fn sign_up(
    body: web::Json<SignUpBody>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> HttpResponse {
    let db = db.as_ref().as_ref();

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

    let _ = user.insert(db).await;

    HttpResponse::Created()
        .await
        .expect("Could not create response")
}

/// Returns the signing key for authorization tokens. And creates one if needed.
async fn get_jwt_signing_key(db: &DatabaseConnection) -> eyre::Result<String> {
    let setting = InstanceSetting::find()
        .filter(instance_setting::Column::Key.eq("AUTH_JWT_SIGNING_KEY"))
        .one(db)
        .await
        .context("Error while attempting to obtain AUTH_JWT_SIGNING_KEY from instance settings")?;

    if let Some(setting) = setting {
        return Ok(setting.value.clone());
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

    new_setting
        .insert(db)
        .await
        .context("Could not insert new setting")?;

    Ok(signing_key)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(sign_up);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::SignUpBody;
    use actix_web::{App, test, web};
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[actix_web::test]
    async fn create_account() {
        let conn = Arc::new(MockDatabase::new(DatabaseBackend::Postgres).into_connection());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(conn.clone()))
                .service(web::scope("/auth").configure(super::configure)),
        )
        .await;

        let body = SignUpBody {
            first_name: "John".into(),
            affix: None,
            last_name: "Doe".into(),
            email: "john.doe@example.com".into(),
            password: "very_secure_password".into(),
        };

        let req = test::TestRequest::post()
            .uri("/auth/sign-up")
            .set_json(body.clone())
            .to_request();

        let service_result = test::call_service(&app, req).await;
        let resp = service_result.response();

        assert!(resp.status().is_success());
    }
}
