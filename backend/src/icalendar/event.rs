use crate::icalendar::{ToIcalendarTimeStamp, if_some_write};
use chrono::{DateTime, Utc};
use hyper::body::{Body, Frame};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

pub enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled,
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tentative => "TENTATIVE",
                Self::Confirmed => "CONFIRMED",
                Self::Cancelled => "CANCELLED",
            }
        )
    }
}

pub struct Event {
    pub uid: Uuid,

    pub timestamp: DateTime<Utc>,
    // pub last_mod: DateTime<Utc>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,

    pub status: Option<EventStatus>,
    pub catagory: Option<String>,

    pub organizer: Option<String>,
    pub summary: Option<String>,

    // NOTE(Julius): I still don't like this.
    /// Used when serializing to a body.
    serialized: bool,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BEGIN:VEVENT\r\n")?;

        write!(
            f,
            "{}",
            format!(
                "DTSTAMP:{}\r\n\
                UID:{}\r\n",
                self.timestamp.to_icalendar_timestamp(),
                self.uid
            )
            .as_str(),
        )?;

        if_some_write(f, "ORGANIZER", &self.organizer)?;

        if let Some(start) = &self.start {
            write!(
                f,
                "{}",
                format!("DTSTART:{}\r\n", start.to_icalendar_timestamp()).as_str(),
            )?;
        }

        if let Some(end) = &self.end {
            write!(
                f,
                "{}",
                format!("DTEND:{}\r\n", end.to_icalendar_timestamp()).as_str()
            )?;
        }

        if_some_write(f, "STATUS", &self.status)?;
        if_some_write(f, "CATEGORIES", &self.catagory)?;
        if_some_write(f, "SUMMARY", &self.summary)?;
        write!(f, "END:VEVENT")
    }
}

impl Body for Event {
    type Data = hyper::body::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(if self.serialized {
            self.get_mut().serialized = false;
            None
        } else {
            let out = self.to_string();
            self.get_mut().serialized = true;
            Some(Ok(Frame::data(out.into())))
        })
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            uid: Uuid::new_v4(),
            timestamp: Utc::now(),
            start: None,
            end: None,
            status: None,
            catagory: None,
            organizer: None,
            summary: None,
            serialized: false,
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::*;

    /// Attempts to parse an [Event] to a String. This *might* fail even when
    /// the result is valid. Since the order of the properties does not matter.
    #[test]
    fn serialize_full_event() {
        let event = Event {
            uid: Uuid::from_str("5aa5c392-fd79-4d24-adc9-8349c49b0b71").unwrap(),
            timestamp: DateTime::from_timestamp_secs(836481600).unwrap(),
            start: Some(DateTime::from_timestamp_secs(843057000).unwrap()),
            end: Some(DateTime::from_timestamp_secs(843256800).unwrap()),
            status: Some(EventStatus::Confirmed),
            catagory: Some("CONFERENCE".into()),
            organizer: Some("mailto:jsmith@example.com".into()),
            summary: Some("Networld+Interop Conference".into()),
            ..Default::default()
        };

        let expected_result = "BEGIN:VEVENT\r\n\
        DTSTAMP:19960704T120000Z\r\n\
        UID:5aa5c392-fd79-4d24-adc9-8349c49b0b71\r\n\
        ORGANIZER:mailto:jsmith@example.com\r\n\
        DTSTART:19960918T143000Z\r\n\
        DTEND:19960920T220000Z\r\n\
        STATUS:CONFIRMED\r\n\
        CATEGORIES:CONFERENCE\r\n\
        SUMMARY:Networld+Interop Conference\r\n\
        END:VEVENT";

        assert_eq!(event.to_string(), expected_result);
    }

    #[test]
    fn serialize_minimal_event() {
        let event = Event {
            uid: Uuid::from_str("014145f2-adf7-49c4-bd42-bf34074f596d").unwrap(),
            timestamp: DateTime::from_timestamp_secs(836553600).unwrap(),
            ..Default::default()
        };

        let expected_result = "BEGIN:VEVENT\r\n\
        DTSTAMP:19960705T080000Z\r\n\
        UID:014145f2-adf7-49c4-bd42-bf34074f596d\r\n\
        END:VEVENT";

        assert_eq!(event.to_string(), expected_result);
    }
}
