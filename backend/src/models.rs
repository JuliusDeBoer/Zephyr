use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,

    pub email: String,
    pub password: String,

    pub first_name: String,
    pub affix: Option<String>,
    pub last_name: String,
}

// NOTE(Julius): This name will be annoying. Just you wait.
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Setting {
    pub key: String,
    pub value: String,
}
