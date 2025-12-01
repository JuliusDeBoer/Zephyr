use std::{str::FromStr, sync::Arc};

use actix_web::{
    HttpRequest, HttpResponse,
    http::{Method, StatusCode},
    web,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entity::prelude::User,
    util::status_error,
    webdav::{
        middleware::UserClaims,
        response::{CalendarProperty, NameOnlyProperty, Property, ResourceType},
        xml::XmlWriter,
    },
};
use crate::{entity::user, webdav::xml::SerializeXml};
use crate::{
    util::EndpointError,
    webdav::response::{MultiStatusResponse, PropStat, Response},
};

#[allow(clippy::manual_let_else)]
async fn handle_propfind(
    req: HttpRequest,
    db: web::Data<Arc<DatabaseConnection>>,
    user_claims: web::ReqData<UserClaims>,
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

    let mut calendars = if depth >= 1 {
        User::find()
            .filter(user::Column::Id.eq(user_claims.user_id))
            .find_with_related(crate::entity::calendar::Entity)
            .all(db)
            .await?[0]
            .1
            .iter()
            .map(|v| PropStat {
                status_code: StatusCode::OK,
                prop: Property::Calendar(CalendarProperty {
                    display_name: v.title.clone(),
                    description: String::new(),
                }),
            })
            .collect()
    } else {
        vec![]
    };

    let mut properties = vec![PropStat {
        status_code: StatusCode::OK,
        prop: Property::NameOnly(NameOnlyProperty {
            display_name: user.display_name,
            resource_type: ResourceType::Collection,
        }),
    }];

    properties.append(&mut calendars);

    let body = match depth {
        i32::MIN..0 => return Err(status_error(StatusCode::BAD_REQUEST)),
        0..=i32::MAX => MultiStatusResponse {
            responses: vec![Response {
                href: "/caldav/".into(),
                properties,
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
        "users/{user_id}",
        web::route()
            .method(Method::from_str("PROPFIND").expect("Could not create PROPFIND method"))
            .to(handle_propfind),
    );
}
