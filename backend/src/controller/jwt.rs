use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rand::{Rng, distr::Alphanumeric, rng};
use rootcause::{Report, prelude::ResultExt};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    entity::{ActiveModelTrait, Set},
};

use crate::entity::prelude::InstanceSetting;
use crate::{controller::users::get_user_by_email, entity::instance_setting};

pub async fn validate_credentials(
    email: &str,
    password: &String,
    db: &DatabaseConnection,
) -> Result<bool, Report> {
    match get_user_by_email(db, email.to_string()).await? {
        None => Ok(false),
        Some(user) => {
            let parsed_hash = PasswordHash::new(&user.password)?;
            let valid = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
            Ok(valid.is_ok())
        }
    }
}

/// Returns the signing key for authorization tokens. And creates one if needed.
pub async fn get_jwt_signing_key(db: &DatabaseConnection) -> Result<String, Report> {
    let setting = InstanceSetting::find()
        .filter(instance_setting::Column::Key.eq("AUTH_JWT_SIGNING_KEY"))
        .one(db)
        .await
        .context("Error while attempting to obtain AUTH_JWT_SIGNING_KEY from instance settings")?;

    if let Some(setting) = setting {
        return Ok(setting.value);
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
