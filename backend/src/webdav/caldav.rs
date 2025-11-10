//! At this point a temporary file to figure out how to implement WebDAV and CalDAV

use actix_web::{
    HttpResponse,
    http::{Method, StatusCode},
    options, web,
};
use chrono::DateTime;
use std::str::FromStr;

use crate::webdav::{
    response::{MultiStatusResponse, PropStat, Property, ResourceType, Response},
    xml::{SerializeXml, XmlWriter},
};

#[options("")]
async fn handle_options() -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("DAV", 1)) // Basic WebDAV (RFC 2518/4918)
        // Note(Julius): Now you *could* argue that having this hardcoded is a
        //               bad idea. However I dont really care.
        .append_header(("Allow", "OPTIONS, GET, HEAD, PUT, DELETE, PROPFIND, MKCOL"))
        .finish()
}

async fn handle_propfind() -> HttpResponse {
    let body = MultiStatusResponse {
        responses: vec![Response {
            href: "/caldav/".into(),
            properties: vec![PropStat {
                status_code: StatusCode::OK,
                prop: Property {
                    resource_type: ResourceType::Collection,
                    display_name: "CalDAV".into(),
                    created_at: DateTime::from_str("2025-11-02 14:30:00Z").unwrap(),
                    last_modified: DateTime::from_str("2025-11-01 10:00:00Z").unwrap(),
                    current_user_principal: "/caldav/principals/user123/".into(),
                },
            }],
        }],
    };

    let mut writer = XmlWriter::new();
    body.write_xml(&mut writer).unwrap();
    HttpResponse::MultiStatus()
        .append_header(("Content-Type", "application/xml"))
        .body(writer.into_bytes())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_options).route(
        "",
        web::route()
            .method(Method::from_str("PROPFIND").unwrap())
            .to(handle_propfind),
    );
}

#[cfg(test)]
mod test {
    use crate::webdav::xml::XmlWriter;

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
                    prop: Property {
                        resource_type: ResourceType::Collection,
                        display_name: "CalDAV".into(),
                        created_at: DateTime::from_str("2025-11-01 10:00:00Z").unwrap(),
                        last_modified: DateTime::from_str("2025-11-02 14:30:00Z").unwrap(),
                        current_user_principal: "/caldav/principals/user123/".into(),
                    },
                }],
            }],
        };

        let mut writer = XmlWriter::new_with_indent();
        body.write_xml(&mut writer).unwrap();
        let bytes: &[u8] = &writer.into_bytes();
        let result = std::str::from_utf8(bytes).unwrap();

        assert_eq!(result, expected);
    }
}
