use std::{str::FromStr, sync::Arc};

use actix_web::{
    HttpRequest, HttpResponse,
    http::{Method, StatusCode},
    options, web,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entity::prelude::User,
    util::status_error,
    webdav::{
        response::{Property, UserProperty},
        xml::XmlWriter,
    },
};
use crate::{entity::user, webdav::xml::SerializeXml};
use crate::{
    util::EndpointError,
    webdav::response::{MultiStatusResponse, PropStat, Response},
};

#[options("")]
async fn handle_options() -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("DAV", 1))
        .append_header(("Allow", "OPTIONS, HEAD, PROPFIND"))
        .finish()
}

async fn handle_propfind(
    req: HttpRequest,
    db: web::Data<Arc<DatabaseConnection>>,
) -> Result<HttpResponse, EndpointError> {
    let db = db.as_ref().as_ref();

    let user_id = match req.match_info().get("user_id") {
        Some(v) => v,
        None => return Err(status_error(StatusCode::NOT_FOUND)),
    };

    // NOTE(Julius): I hate this.
    let user_id = match Uuid::from_str(user_id) {
        Ok(v) => v,
        Err(_) => return Err(status_error(StatusCode::BAD_REQUEST)),
    };

    let user = match User::find()
        .filter(user::Column::Id.eq(user_id))
        .one(db)
        .await?
    {
        Some(v) => v,
        None => return Err(status_error(StatusCode::NOT_FOUND)),
    };

    let depth: i32 = match req.headers().iter().find(|h| h.0 == "Depth") {
        Some(v) => String::from(v.1.to_str()?).parse()?,
        None => return Err(status_error(StatusCode::FORBIDDEN)),
    };

    let body = match depth {
        i32::MIN..0 => return Err(status_error(StatusCode::BAD_REQUEST)),
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
    body.write_xml(&mut writer).unwrap();
    Ok(HttpResponse::MultiStatus()
        .append_header(("Content-Type", "application/xml"))
        .body(writer.into_bytes()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_options).route(
        "principals/users/{user_id}",
        web::route()
            .method(Method::from_str("PROPFIND").unwrap())
            .to(handle_propfind),
    );
}
