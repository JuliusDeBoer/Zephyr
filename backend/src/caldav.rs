//! At this point a temporary file to figure out how to implement WebDAV and CalDAV

use actix_web::{
    HttpResponse,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::InternalError,
    http::{Method, StatusCode},
    options, web,
};
use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    str::FromStr,
};

use crate::xml::{SerializeXml, WEBDAV_NAMESPACES, XmlWriter};

#[derive(Debug)]
struct MultiStatusResponse {
    responses: Vec<Response>,
}

#[derive(Debug)]
struct Response {
    href: String,
    properties: Vec<PropStat>,
}

#[derive(Debug)]
struct PropStat {
    /// The status code this property represents. This *MUST* be in the following format:
    /// ```
    /// HTTP/1.1 200 OK
    /// ```
    status_code: StatusCode,
    prop: Property,
}

#[derive(Debug)]
struct Property {
    resource_type: ResourceType,
    display_name: String,
    last_modified: DateTime<Utc>,
    created_at: DateTime<Local>,
    current_user_principal: String,
}

#[derive(Debug)]
enum ResourceType {
    Collection,
    ResourceType,
    Calendar,
}

impl SerializeXml for MultiStatusResponse {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<()> {
        writer.start_element_with_attrs("d:multistatus", WEBDAV_NAMESPACES)?;
        for response in self.responses {
            response.write_xml(writer)?;
        }
        writer.end_element("d:multistatus")?;
        Ok(())
    }
}

impl SerializeXml for Response {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<()> {
        writer.start_element("d:response")?;
        writer.start_element("d:href")?;
        writer.add_text(self.href.as_str())?;
        writer.end_element("d:href")?;
        for property in self.properties {
            property.write_xml(writer)?;
        }
        writer.end_element("d:response")?;
        Ok(())
    }
}

impl SerializeXml for PropStat {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<()> {
        writer.start_element("d:propstat")?;
        self.prop.write_xml(writer)?;
        writer.start_element("d:status")?;
        writer.add_text(format!("HTTP/1.1 {}", self.status_code,).as_str())?;
        writer.end_element("d:status")?;
        writer.end_element("d:propstat")?;
        Ok(())
    }
}

impl SerializeXml for Property {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<()> {
        writer.start_element("d:prop")?;
        writer.start_element("d:resourcetype")?;
        match self.resource_type {
            ResourceType::Collection => {
                writer.empty_element("d:collection")?;
            }
            ResourceType::ResourceType => {}
            ResourceType::Calendar => {
                writer.empty_element("cal:calendar")?;
            }
        };
        writer.end_element("d:resourcetype")?;
        writer.start_element("d:displayname")?;
        writer.add_text(self.display_name.as_str())?;
        writer.end_element("d:displayname")?;
        writer.start_element("d:getlastmodified")?;
        writer.add_text(self.last_modified.to_rfc2822().as_str())?;
        writer.end_element("d:getlastmodified")?;
        writer.start_element("d:creationdate")?;
        writer.add_text(
            self.created_at
                .with_timezone(&Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
                .as_str(),
        )?;
        writer.end_element("d:creationdate")?;
        writer.start_element("d:current-user-principal")?;
        writer.start_element("d:href")?;
        writer.add_text(self.current_user_principal.as_str())?;
        writer.end_element("d:href")?;
        writer.end_element("d:current-user-principal")?;
        writer.end_element("d:prop")?;
        Ok(())
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

#[derive(Default)]
pub struct CalDavAuth {}

impl<S, B> Transform<S, ServiceRequest> for CalDavAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CalDavAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CalDavAuthMiddleware { service }))
    }
}

#[derive(Default)]
pub struct CalDavAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CalDavAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth = req.headers().iter().find(|h| h.0 == "Authorization");

        match auth {
            Some((_, status)) if status == "Authorized" => {
                let fut = self.service.call(req);

                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            _ => Box::pin(async move {
                let response = HttpResponse::Unauthorized()
                    .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                    .finish();

                Err(InternalError::from_response("Unauthorized", response).into())
            }),
        }
    }
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
    use crate::xml::XmlWriter;

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
