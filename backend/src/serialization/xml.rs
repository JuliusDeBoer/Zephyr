use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};
use rootcause::Report;

/// A collection of namespaces used when serializing to XML
pub static WEBDAV_NAMESPACES: &[(&str, &str)] = &[
    ("xmlns:d", "DAV:"),
    ("xmlns:cal", "urn:ietf:params:xml:ns:caldav"),
    ("xmlns:cs", "http://calendarserver.org/ns/"),
];

pub trait SerializeXml {
    fn write_xml(self, writer: &mut XmlWriter) -> Result<(), Report>;
}

pub struct XmlWriter {
    writer: Writer<Vec<u8>>,
}

impl XmlWriter {
    pub fn new() -> Self {
        let mut writer = Writer::new(Vec::new());
        _ = writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)));
        Self { writer }
    }

    #[allow(unused, reason = "Used in testing for better diffs")]
    pub fn new_with_indent() -> Self {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 4);
        _ = writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)));
        Self { writer }
    }

    pub fn start_element(&mut self, name: &str) -> Result<(), Report> {
        self.writer
            .write_event(Event::Start(BytesStart::new(name)))?;
        Ok(())
    }

    pub fn start_element_with_attrs(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> Result<(), Report> {
        let mut element = BytesStart::new(name);

        for (key, value) in attrs {
            element.push_attribute((*key, *value));
        }

        self.writer.write_event(Event::Start(element))?;
        Ok(())
    }

    pub fn empty_element(&mut self, name: &str) -> Result<(), Report> {
        self.writer
            .write_event(Event::Empty(BytesStart::new(name)))?;
        Ok(())
    }

    pub fn empty_element_with_attrs(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> Result<(), Report> {
        let mut element = BytesStart::new(name);

        for (key, value) in attrs {
            element.push_attribute((*key, *value));
        }

        self.writer.write_event(Event::Empty(element))?;
        Ok(())
    }

    pub fn end_element(&mut self, name: &str) -> Result<(), Report> {
        self.writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }

    pub fn add_text(&mut self, content: &str) -> Result<(), Report> {
        self.writer
            .write_event(Event::Text(BytesText::new(content)))?;
        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.writer.into_inner()
    }
}
