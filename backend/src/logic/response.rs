use actix_web::http::StatusCode;
use chrono::{DateTime, Local, Utc};
use rootcause::Report;

use crate::serialization::xml::{SerializeXml, WEBDAV_NAMESPACES, XmlWriter};

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
pub enum Property {
    Root(RootProperty),
    User(UserProperty),
    NameOnly(NameOnlyProperty),
    Calendar(CalendarProperty),
}

impl SerializeXml for Property {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        match self {
            Self::Root(v) => v.write_xml(writer),
            Self::User(v) => v.write_xml(writer),
            Self::NameOnly(v) => v.write_xml(writer),
            Self::Calendar(v) => v.write_xml(writer),
        }
    }
}

#[derive(Debug)]
pub struct RootProperty {
    pub resource_type: ResourceType,
    pub display_name: String,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Local>,
    pub current_user_principal: String,
    pub permissions: WebDavPermissions,
    pub ctag: String,
}

#[derive(Debug)]
pub struct WebDavPermissions {
    pub read: bool,
    pub write: bool,
    pub write_content: bool,
}

#[derive(Debug)]
pub struct UserProperty {
    pub display_name: String,
    pub calendar_home_set: String,
    pub principal: String,
    pub current_user_principal: String,
}

#[derive(Debug)]
pub struct NameOnlyProperty {
    pub resource_type: ResourceType,
    pub display_name: String,
}

#[derive(Debug)]
pub struct CalendarProperty {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug)]
pub enum ResourceType {
    Collection,
    Empty,
    Calendar,
}

impl SerializeXml for MultiStatusResponse {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        writer.start_element_with_attrs("d:multistatus", WEBDAV_NAMESPACES)?;
        for response in self.responses {
            response.write_xml(writer)?;
        }
        writer.end_element("d:multistatus")?;
        Ok(())
    }
}

impl SerializeXml for Response {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
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

impl SerializeXml for NameOnlyProperty {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
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
        }
        writer.end_element("d:resourcetype")?;
        writer.start_element("d:displayname")?;
        writer.add_text(self.display_name.as_str())?;
        writer.end_element("d:displayname")?;
        writer.end_element("d:prop")?;
        Ok(())
    }
}

impl SerializeXml for CalendarProperty {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        writer.start_element("d:prop")?;

        // NOTE(Julius): It might be worth it to allow changing these
        //               properties.
        writer.start_element("d:resourcetype")?;
        writer.empty_element("d:collection")?;
        writer.empty_element("cal:calendar")?;
        writer.end_element("d:resourcetype")?;

        writer.start_element("d:displayname")?;
        writer.add_text(self.display_name.as_str())?;
        writer.end_element("d:displayname")?;

        writer.start_element("cal:calendar-description")?;
        writer.add_text(self.description.as_str())?;
        writer.end_element("cal:calendar-description")?;

        writer.start_element("cal:supported-calendar-component-set")?;
        writer.empty_element_with_attrs("cal:comp", &[("name", "VEVENT")])?;
        writer.end_element("cal:supported-calendar-component-set")?;
        writer.end_element("d:prop")?;
        Ok(())
    }
}

impl SerializeXml for PropStat {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        writer.start_element("d:propstat")?;
        self.prop.write_xml(writer)?;
        writer.start_element("d:status")?;
        writer.add_text(format!("HTTP/1.1 {}", self.status_code,).as_str())?;
        writer.end_element("d:status")?;
        writer.end_element("d:propstat")?;
        Ok(())
    }
}

impl SerializeXml for WebDavPermissions {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        writer.start_element("d:current-user-privilege-set")?;
        if self.read {
            writer.start_element("d:privilege")?;
            writer.empty_element("d:read")?;
            writer.end_element("d:privilege")?;
        }
        if self.write {
            writer.start_element("d:privilege")?;
            writer.empty_element("d:write")?;
            writer.end_element("d:privilege")?;
        }
        if self.write_content {
            writer.start_element("d:privilege")?;
            writer.empty_element("d:write-content")?;
            writer.end_element("d:privilege")?;
        }
        writer.end_element("d:current-user-privilege-set")?;
        Ok(())
    }
}

impl SerializeXml for RootProperty {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
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
        }
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
        self.permissions.write_xml(writer)?;
        writer.end_element("d:prop")?;
        Ok(())
    }
}

impl SerializeXml for UserProperty {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report> {
        writer.start_element("d:prop")?;
        writer.start_element("d:displayname")?;
        writer.add_text(self.display_name.as_str())?;
        writer.end_element("d:displayname")?;
        writer.start_element("cal:calendar-home-set")?;
        writer.start_element("d:href")?;
        writer.add_text(self.calendar_home_set.as_str())?;
        writer.end_element("d:href")?;
        writer.end_element("cal:calendar-home-set")?;
        writer.start_element("d:principal-URL")?;
        writer.start_element("d:href")?;
        writer.add_text(self.principal.as_str())?;
        writer.end_element("d:href")?;
        writer.end_element("d:principal-URL")?;
        writer.start_element("d:current-user-principal")?;
        writer.start_element("d:href")?;
        writer.add_text(self.current_user_principal.as_str())?;
        writer.end_element("d:href")?;
        writer.end_element("d:current-user-principal")?;
        writer.end_element("d:prop")?;
        Ok(())
    }
}
