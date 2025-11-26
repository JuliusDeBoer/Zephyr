use std::collections::BTreeMap;
use std::sync::Arc;

use actix_web::{HttpResponse, post, web};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sea_orm::{
    ColumnTrait, DatabaseConnection, QueryFilter,
    entity::{ActiveEnum, ActiveModelBehavior, ActiveModelTrait, EntityTrait, Related, Set},
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

use crate::entity::calendar;
use crate::entity::prelude::User;
use crate::entity::user;
use crate::jwt::{get_jwt_signing_key, validate_credentials};
use crate::util::EndpointError;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SignUpBody {
    email: String,
    password: String,
    first_name: String,
    affix: Option<String>,
    last_name: String,
    display_name: String,
}

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
) -> Result<HttpResponse, EndpointError> {
    let db = db.as_ref().as_ref();

    let valid = validate_credentials(&body.email, &body.password, db).await?;

    if valid {
        let user = User::find()
            .filter(user::Column::Email.eq(&body.email))
            .one(db)
            .await?
            .expect("Could not find user");

        let key: Hmac<Sha256> = Hmac::new_from_slice(get_jwt_signing_key(db).await?.as_bytes())?;
        let mut claims = BTreeMap::new();
        claims.insert("sub", user.id.to_string());
        Ok(HttpResponse::Ok().body(claims.sign_with_key(&key)?))
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}

#[post("/sign-up")]
async fn sign_up(
    body: web::Json<SignUpBody>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, EndpointError> {
    let db = db.as_ref().as_ref();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)?
        .to_string();

    let user_id = Uuid::new_v4();
    let user = user::ActiveModel {
        id: Set(user_id),
        email: Set(body.email.clone()),
        password: Set(password_hash),
        first_name: Set(body.first_name.clone()),
        last_name: Set(body.last_name.clone()),
        affix: Set(body.affix.clone()),
        display_name: Set(body.display_name.clone()),
    };

    let calendar = calendar::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(format!("{}'s calendar", body.first_name)),
        colour: Set("#63a6d7".into()),
        owner: Set(user_id),
        ..Default::default()
    };

    user.insert(db).await?;
    calendar.insert(db).await?;

    Ok(HttpResponse::Created().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(sign_up);
}
