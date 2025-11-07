use actix_web::http::StatusCode;
use anyhow::Result;
use chrono::{DateTime, Local, Utc};

use super::xml::{SerializeXml, WEBDAV_NAMESPACES, XmlWriter};

#[derive(Debug)]
pub struct MultiStatusResponse {
    pub responses: Vec<Response>,
}

#[derive(Debug)]
pub struct Response {
    pub href: String,
    pub properties: Vec<PropStat>,
}

#[derive(Debug)]
pub struct PropStat {
    /// The status code this property represents. This *MUST* be in the following format:
    /// ```
    /// HTTP/1.1 200 OK
    /// ```
    pub status_code: StatusCode,
    pub prop: Property,
}

#[derive(Debug)]
pub struct Property {
    pub resource_type: ResourceType,
    pub display_name: String,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Local>,
    pub current_user_principal: String,
}

#[derive(Debug)]
pub enum ResourceType {
    Collection,
    Empty,
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
            ResourceType::Empty => {}
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
