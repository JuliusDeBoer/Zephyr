use std::collections::BTreeMap;
use std::sync::Arc;

use actix_web::{HttpResponse, post, web};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sea_orm::{ColumnTrait, DatabaseConnection, QueryFilter, entity::*};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

use crate::entity::prelude::User;
use crate::entity::user;
use crate::jwt::{get_jwt_signing_key, validate_credentials};

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

    let valid = validate_credentials(&body.email, &body.password, db)
        .await
        .unwrap();

    if valid {
        let user = User::find()
            .filter(user::Column::Email.eq(&body.email))
            .one(db)
            .await
            .unwrap()
            .unwrap();

        let key: Hmac<Sha256> =
            Hmac::new_from_slice(get_jwt_signing_key(db).await.unwrap().as_bytes()).unwrap();
        let mut claims = BTreeMap::new();
        claims.insert("sub", user.id.to_string());
        HttpResponse::Ok().body(claims.sign_with_key(&key).unwrap())
    } else {
        HttpResponse::BadRequest().await.unwrap()
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
