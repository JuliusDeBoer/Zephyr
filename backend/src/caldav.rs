//! At this point a temporary file to figure out how to implement WebDAV and CalDAV

use std::str::FromStr;

use actix_web::{
    HttpResponse,
    http::{Method, StatusCode},
    options, web,
};
use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};
use serde::{Serialize, ser::SerializeStruct};

#[derive(Debug)]
struct MultiStatusResponse {
    response: CaldavResponse,
}

#[derive(Serialize, Debug)]
struct CaldavResponse {
    #[serde(rename = "d:href")]
    href: String,
    #[serde(rename = "d:prop")]
    properties: Vec<PropertyStatus>,
}

impl Serialize for MultiStatusResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("d:multistatus", 3)?;
        s.serialize_field("@xmlns:d", "DAV:")?;
        s.serialize_field("@xmlns:cal", "urn:ietf:params:xml:ns:caldav")?;
        s.serialize_field("d:response", &self.response)?;
        s.end()
    }
}

#[derive(Debug)]
struct PropertyStatus {
    /// The status code this property represents. This *MUST* be in the following format:
    /// ```
    /// HTTP/1.1 200 OK
    /// ```
    status_code: StatusCode,
    prop: Property,
}

impl Serialize for PropertyStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("PropertyStatus", 2)?;
        s.serialize_field(
            "d:status",
            &format!(
                "HTTP/1.1 {} {}",
                self.status_code.as_str(),
                self.status_code.canonical_reason().unwrap_or_default()
            ),
        )?;
        s.serialize_field("d:prop", &self.prop)?;
        s.end()
    }
}

#[derive(Debug)]
struct Property {
    resource_type: ResourceType,
    display_name: String,
    last_modified: DateTime<Utc>,
    created_at: DateTime<Local>,
}

impl Serialize for Property {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Property", 4)?;
        s.serialize_field("d:resourcetype", &self.resource_type)?;
        s.serialize_field("d:displayname", &self.display_name)?;
        s.serialize_field(
            "d:getlastmodified",
            &self.last_modified.with_timezone(&Utc).to_rfc2822(),
        )?;
        s.serialize_field(
            "d:creationdate",
            &self
                .created_at
                .with_timezone(&Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
        )?;
        s.end()
    }
}

#[derive(Debug)]
enum ResourceType {
    Collection,
    ResourceType,
    Calendar,
}

#[derive(Serialize)]
struct Empty {}

impl serde::Serialize for ResourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // HACK(Julius): What you dont see can't hurt you.
        let mut s = serializer.serialize_struct("d:resourcetype", 1)?;
        match self {
            ResourceType::Collection => {
                s.serialize_field("d:collection", &Empty {})?;
            }
            ResourceType::ResourceType => {}
            ResourceType::Calendar => {
                s.serialize_field("cal:calendar", &Empty {})?;
            }
        };
        s.end()
    }
}

#[options("")]
async fn handle_options() -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("DAV", 1)) // Basic WebDAV (RFC 2518/4918)
        // Note(Julius): Now you *could* argue that having this hardcoded is a
        //               bad idea. However I dont really care.
        .append_header(("Allow", "OPTIONS, GET, HEAD, PUT, DELETE, PROPFIND, MKCOL"))
        .await
        .unwrap()
}

async fn handle_propfind(body: String) -> HttpResponse {
    dbg!(body);
    let body = MultiStatusResponse {
        response: CaldavResponse {
            href: "/caldav/".into(),
            properties: vec![PropertyStatus {
                status_code: StatusCode::OK,
                prop: Property {
                    resource_type: ResourceType::Collection,
                    display_name: "CalDav".into(),
                    created_at: DateTime::from_str("2025-11-02 14:30:00Z").unwrap(),
                    last_modified: DateTime::from_str("2025-11-01 10:00:00Z").unwrap(),
                },
            }],
        },
    };

    HttpResponse::MultiStatus()
        .append_header(("Content-Type", "application/xml"))
        .body(quick_xml::se::to_string(&body).unwrap())
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
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn serialize_mutli_status_response() {
        let expected =
            "<d:multistatus xmlns:d=\"DAV:\" xmlns:cal=\"urn:ietf:params:xml:ns:caldav\">
          <d:response>
            <d:href>/caldav/</d:href>
            <d:propstat>
              <d:prop>
                <d:resourcetype>
                  <d:collection/>
                </d:resourcetype>
                <d:displayname>CalDAV</d:displayname>
                <d:getlastmodified>Sun, 02 Nov 2025 14:30:00 GMT</d:getlastmodified>
                <d:creationdate>2025-11-01T10:00:00Z</d:creationdate>
              </d:prop>
              <d:status>HTTP/1.1 200 OK</d:status>
            </d:propstat>
          </d:response>
        </d:multistatus>";

        let result = quick_xml::se::to_string(&MultiStatusResponse {
            response: CaldavResponse {
                href: "/caldav/".into(),
                properties: vec![PropertyStatus {
                    status_code: StatusCode::OK,
                    prop: Property {
                        resource_type: ResourceType::Collection,
                        display_name: "CalDav".into(),
                        created_at: DateTime::from_str("2025-11-02 14:30:00Z").unwrap(),
                        last_modified: DateTime::from_str("2025-11-01 10:00:00Z").unwrap(),
                    },
                }],
            },
        })
        .unwrap();

        assert_eq!(expected, result);
    }
}
