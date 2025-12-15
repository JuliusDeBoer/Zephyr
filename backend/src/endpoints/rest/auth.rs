use std::collections::BTreeMap;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, post, web};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use rootcause::prelude::ResultExt;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::controller::jwt::{get_jwt_signing_key, validate_credentials};
use crate::controller::users::{SignUpData, create_user_with_calendar, get_user_by_email};
use crate::logic::error::ApiError;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    email: String,
    password: String,
}

#[post("/login")]
async fn login(
    db: web::Data<Arc<DatabaseConnection>>,
    body: web::Json<LoginBody>,
) -> Result<HttpResponse, ApiError> {
    let db = db.as_ref().as_ref();

    let valid = validate_credentials(&body.email, &body.password, db)
        .await
        .context("Could not validate credentials")?;

    if valid {
        let user = get_user_by_email(db, body.email.clone())
            .await?
            .ok_or_else(|| ApiError::new("Could not find user", StatusCode::NOT_FOUND))?;

        let key: Hmac<Sha256> = Hmac::new_from_slice(get_jwt_signing_key(db).await?.as_bytes())?;
        let mut claims = BTreeMap::new();
        claims.insert("sub", user.id.to_string());
        Ok(HttpResponse::Ok().body(claims.sign_with_key(&key)?))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[post("/sign-up")]
async fn sign_up(
    body: web::Json<SignUpData>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, ApiError> {
    let db = db.as_ref().as_ref();

    create_user_with_calendar(db, body.0).await?;

    Ok(HttpResponse::Created().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(sign_up);
}
