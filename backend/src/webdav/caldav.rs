//! At this point a temporary file to figure out how to implement `WebDAV` and `CalDAV`

use actix_web::{
    HttpRequest, HttpResponse,
    http::{Method, StatusCode},
    options, web,
};
use chrono::DateTime;
use rootcause::{Report, report};
use std::str::FromStr;

use crate::{
    util::EndpointError,
    webdav::{
        middleware::UserClaims,
        principals,
        response::{
            MultiStatusResponse, PropStat, Property, ResourceType, Response, RootProperty,
            WebDavPermissions,
        },
        users,
        xml::{SerializeXml, XmlWriter},
    },
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
    body: String,
    user_claims: web::ReqData<UserClaims>,
) -> Result<HttpResponse, EndpointError> {
    dbg!(body);

    let depth: i32 = match req.headers().iter().find(|h| h.0 == "Depth") {
        Some(v) => String::from(v.1.to_str()?).parse()?,
        None => {
            return Err(EndpointError::StatusCode(StatusCode::FORBIDDEN));
        }
    };

    let body = match depth {
        i32::MIN..0 => {
            return Err(EndpointError::StatusCode(StatusCode::BAD_REQUEST));
        }
        0..=i32::MAX => MultiStatusResponse {
            responses: vec![Response {
                href: "/caldav/".into(),
                properties: vec![PropStat {
                    status_code: StatusCode::OK,
                    prop: Property::Root(RootProperty {
                        resource_type: ResourceType::Collection,
                        display_name: "CalDAV".into(),
                        created_at: DateTime::from_str("2025-11-02 14:30:00Z")
                            .expect("Could not parse string"),
                        last_modified: DateTime::from_str("2025-11-01 10:00:00Z")
                            .expect("Could not parse string"),
                        current_user_principal: format!(
                            "/caldav/principals/users/{}/",
                            user_claims.user_id
                        ),
                        permissions: WebDavPermissions {
                            // TODO(Julius): This isn't secure...
                            read: true,
                            write: true,
                            write_content: true,
                        },
                        ctag: "AAA".into(),
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
    cfg.service(handle_options).route(
        "",
        web::route()
            .method(Method::from_str("PROPFIND").expect("Could not create PROPFIND method"))
            .to(handle_propfind),
    );

    principals::configure(cfg);
    users::configure(cfg);
}

#[cfg(test)]
mod test {
    use crate::webdav::{response::Property, xml::XmlWriter};

    use actix_web::http::StatusCode;
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn serialize_mutli_status_response() {
        let expected = "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<d:multistatus xmlns:d=\"DAV:\" xmlns:cal=\"urn:ietf:params:xml:ns:caldav\">
    <d:response>
        <d:href>/caldav/</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype>
                    <d:collection/>
                </d:resourcetype>
                <d:displayname>CalDAV</d:displayname>
                <d:getlastmodified>Sun, 2 Nov 2025 14:30:00 +0000</d:getlastmodified>
                <d:creationdate>2025-11-01T10:00:00Z</d:creationdate>
                <d:current-user-principal>
                    <d:href>/caldav/principals/user123/</d:href>
                </d:current-user-principal>
                <d:current-user-privilege-set>
                    <d:privilege>
                        <d:read/>
                    </d:privilege>
                    <d:privilege>
                        <d:write/>
                    </d:privilege>
                    <d:privilege>
                        <d:write-content/>
                    </d:privilege>
                </d:current-user-privilege-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>";

        let body = MultiStatusResponse {
            responses: vec![Response {
                href: "/caldav/".into(),
                properties: vec![PropStat {
                    status_code: StatusCode::OK,
                    prop: Property::Root(RootProperty {
                        resource_type: ResourceType::Collection,
                        display_name: "CalDAV".into(),
                        created_at: DateTime::from_str("2025-11-01 10:00:00Z")
                            .expect("Could not parse date"),
                        last_modified: DateTime::from_str("2025-11-02 14:30:00Z")
                            .expect("Could not parse date"),
                        current_user_principal: "/caldav/principals/user123/".into(),
                        ctag: "AAA".into(),
                        permissions: WebDavPermissions {
                            read: true,
                            write: true,
                            write_content: true,
                        },
                    }),
                }],
            }],
        };

        let mut writer = XmlWriter::new_with_indent();
        body.write_xml(&mut writer)
            .expect("Could not serialize body");
        let bytes: &[u8] = &writer.into_bytes();
        let result = std::str::from_utf8(bytes).expect("Could not parse body");

        assert_eq!(result, expected);
    }
}
