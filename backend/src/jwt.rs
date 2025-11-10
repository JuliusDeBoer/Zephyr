use argon2::{Argon2, PasswordHash, PasswordVerifier};
use eyre::{Context, Result};
use rand::{Rng, distr::Alphanumeric, rng};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult, QueryFilter,
    entity::*,
};

use crate::entity::instance_setting;
use crate::entity::prelude::{InstanceSetting, User};
use crate::entity::user;

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "user::Entity")]
struct PasswordOnly {
    password: String,
}

pub async fn validate_credentials(
    email: &String,
    password: &String,
    db: &DatabaseConnection,
) -> Result<bool> {
    let user_result: Option<PasswordOnly> = User::find()
        .filter(user::Column::Email.eq(email))
        .into_partial_model()
        .one(db)
        .await?;

    match user_result {
        None => Ok(false),
        Some(user) => {
            let parsed_hash = PasswordHash::new(&user.password).unwrap();
            let valid = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
            Ok(valid.is_ok())
        }
    }
}

/// Returns the signing key for authorization tokens. And creates one if needed.
pub async fn get_jwt_signing_key(db: &DatabaseConnection) -> eyre::Result<String> {
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
