use crate::{
    controller::calendars::{CalendarData, create_calendar_for_user},
    entity::user,
};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use rootcause::{Report, prelude::ResultExt};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignUpData {
    pub email: String,
    /// A password in plain text
    pub password: String,
    pub first_name: String,
    pub affix: Option<String>,
    pub last_name: String,
    pub display_name: String,
}

pub async fn get_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<user::Model>, Report> {
    Ok(user::Entity::find()
        .filter(user::Column::Id.eq(id))
        .one(db)
        .await
        .context(format!(
            "Error while attempting to fetch user with id: `{id}`"
        ))?)
}

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<Option<user::Model>, Report> {
    Ok(user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(db)
        .await
        .context(format!(
            "Error while attempting to fetch user with email: `{email}`"
        ))?)
}

/// Creates a new user with a new calendar.
pub async fn create_user_with_calendar(
    db: &DatabaseConnection,
    data: SignUpData,
) -> Result<(), Report> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(data.password.as_bytes(), &salt)?
        .to_string();

    let user_id = Uuid::new_v4();
    let user = user::ActiveModel {
        id: Set(user_id),
        email: Set(data.email.clone()),
        password: Set(password_hash),
        first_name: Set(data.first_name.clone()),
        last_name: Set(data.last_name.clone()),
        affix: Set(data.affix.clone()),
        display_name: Set(data.display_name.clone()),
    };

    user.insert(db).await?;

    create_calendar_for_user(
        db,
        user_id,
        CalendarData {
            title: format!("{}'s calendar", data.display_name),
            colour: "#63a6d7".into(),
        },
    )
    .await?;

    Ok(())
}
