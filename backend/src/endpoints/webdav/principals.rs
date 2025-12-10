use std::{str::FromStr, sync::Arc};

use actix_web::{
    HttpRequest, HttpResponse,
    http::{Method, StatusCode},
    web,
};
use rootcause::report;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entity::prelude::User,
    logic::error::WithStatusCode,
    logic::response::{Property, UserProperty},
    serialization::xml::XmlWriter,
};
use crate::{entity::user, serialization::xml::SerializeXml};
use crate::{
    logic::error::ApiError,
    logic::response::{MultiStatusResponse, PropStat, Response},
};

#[allow(clippy::manual_let_else)]
async fn handle_propfind(
    req: HttpRequest,
    db: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, ApiError> {
    let db = db.as_ref().as_ref();

    let user_id = req
        .match_info()
        .get("user_id")
        .ok_or_else(|| report!("Coult not find user"))
        .with_status(StatusCode::NOT_FOUND)?;

    let user_id = Uuid::from_str(user_id).with_status(StatusCode::BAD_REQUEST)?;

    let user = User::find()
        .filter(user::Column::Id.eq(user_id))
        .one(db)
        .await?
        .ok_or_else(|| report!("Could not find user"))
        .with_status(StatusCode::NOT_FOUND)?;

    let depth: i32 = match req.headers().iter().find(|h| h.0 == "Depth") {
        Some(v) => String::from(v.1.to_str()?).parse()?,
        None => {
            return Err(ApiError::new(
                "Invalid `Depth` header",
                StatusCode::FORBIDDEN,
            ));
        }
    };

    let body = match depth {
        i32::MIN..0 => {
            return Err(ApiError::new(
                "Invalid `Depth` header",
                StatusCode::BAD_REQUEST,
            ));
        }
        0..=i32::MAX => MultiStatusResponse {
            responses: vec![Response {
                href: "/caldav/".into(),
                properties: vec![PropStat {
                    status_code: StatusCode::OK,
                    prop: Property::User(UserProperty {
                        display_name: user.display_name,
                        calendar_home_set: format!("/caldav/users/{}", user.id),
                        principal: format!("/caldav/principals/users/{}", user.id),
                        current_user_principal: format!("/caldav/principals/users/{}", user.id),
                    }),
                }],
            }],
        },
    };

    let mut writer = XmlWriter::new();
    body.write_xml(&mut writer)?;
    Ok(HttpResponse::MultiStatus()
        .append_header(("Content-Type", "application/xml"))
        .body(writer.into_bytes()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "principals/users/{user_id}",
        web::route()
            .method(Method::from_str("PROPFIND").expect("Could not create PROPFIND method"))
            .to(handle_propfind),
    );
}
