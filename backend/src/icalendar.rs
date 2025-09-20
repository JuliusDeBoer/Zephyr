use chrono::{DateTime, SecondsFormat, Utc};
use hyper::body::Body;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

/// An implementation of the ICalendar datastructure as defined in RFC 5545.

//
// NOTE(Julius): This type does not support `x-name` and `iana-token` like
//               described in RFC 5545 3.2.3
pub enum ICalendarUserType {
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
}

pub enum ICalendarEncodingType {
    EightBit,
    BaseSixtyFour,
}

// NOTE(Julius): This type does not support `x-name` and `iana-token` like
//               described in RFC 5545 3.2.9
pub enum ICalendarFreeBusyType {
    Free,
    Busy,
    BusyUnavailable,
    BusyAvailable,
}

// TODO(Julius): Write my own serializer/deserializer for iCalendar objects
// NOTE(Julius): When I *do* implement the serialization. Make sure to remember
//               that iCalendar objects use CRLF instead of just LF.
pub struct ICalendarObject {
    // Alternate text representation
    pub alt_rep: String,
    // Common name
    pub cn: String,
    // Calendar user type
    // TODO(Julius): This enum doesn't allow for any aditional attributes
    pub cu_type: ICalendarUserType,
    // Delegator
    pub del_from: String,
    // Delegatee
    pub del_to: String,
    // Directory entry
    pub dir: String, // URI
    // Inline encoding
    pub encoding: ICalendarEncodingType,
    // Format type
    pub fmt_type: String,
    // Freeusy time type
    pub fb_type: ICalendarFreeBusyType,
    // Language for text
    pub language: String, // Oh languages. Yipee!
    // Group or list membership
    pub member: String,
    // Participation status
    pub part_stat: String,
    // Recurrence identifier range
    pub range: u32,
    // Alarm trigger relationship
    pub trig_rel: u32,
    // Relationship type
    pub rel_type: u32,
    // Participation role
    pub role: u32,
    // RSVP expectation
    pub rsvp: bool,
    // Sent by
    pub sent_by_: u32,
    // Reference to time zone object
    pub tzid: u32,
    // Property value data type
    pub value_type: u32,
    pub other: u32,
}

impl Body for ICalendarObject {
    type Data = hyper::body::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let ical_string = format!(
            "BEGIN:VCALENDAR\r\n\
            VERSION:2.0\r\n\
            PRODID:-//Zephyr//NONSGML v1.0//EN\r\n\
            END:VCALENDAR\r\n"
        );

        // FIXME
        static mut DONE: bool = false;

        unsafe {
            Poll::Ready(match DONE {
                false => {
                    DONE = true;
                    Some(Ok(hyper::body::Frame::data(ical_string.into())))
                }
                true => {
                    DONE = false;
                    None
                }
            })
        }
    }
}

pub enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled,
}

impl ToString for EventStatus {
    fn to_string(&self) -> String {
        (match self {
            EventStatus::Tentative => "TENTATIVE",
            EventStatus::Confirmed => "CONFIRMED",
            EventStatus::Cancelled => "CANCELLED",
        })
        .into()
    }
}

pub struct Event {
    pub uid: Uuid,

    pub timestamp: DateTime<Utc>,
    // pub last_mod: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,

    pub status: EventStatus,
    pub catagory: String,

    pub organizer: String,
    pub summery: String,
}

impl ToString for Event {
    fn to_string(&self) -> String {
        format!(
            "BEGIN:VEVENT\r\n\
            DTSTAMP:{}\r\n\
            UID:{}\r\n\
            ORGANIZER:{}\r\n\
            DTSTART:{}\r\n\
            DTEND:{}\r\n\
            STATUS:{}\r\n\
            CATEGORIES:{}\r\n\
            SUMMARY:{}\r\n\
            END:VEVENT",
            self.timestamp.format("%Y%m%dT%H%M%SZ"),
            self.uid,
            self.organizer,
            self.start.format("%Y%m%dT%H%M%SZ"),
            self.end.format("%Y%m%dT%H%M%SZ"),
            self.status.to_string(),
            self.catagory,
            self.summery
        )
    }
}

impl Body for Event {
    type Data = hyper::body::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let out = self.to_string();
        // FIXME
        static mut DONE: bool = false;

        unsafe {
            Poll::Ready(match DONE {
                false => {
                    DONE = true;
                    Some(Ok(hyper::body::Frame::data(out.into())))
                }
                true => {
                    DONE = false;
                    None
                }
            })
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[allow(unused)]
    use super::*;
    /// Attempts to parse an [Event] to a String. This *might* fail even when
    /// the result is valid. Since the order of the properties does not matter.
    #[test]
    fn serialize_event() {
        let event = Event {
            uid: Uuid::from_str("5aa5c392-fd79-4d24-adc9-8349c49b0b71").unwrap(),
            timestamp: DateTime::from_timestamp_secs(836481600).unwrap(),
            start: DateTime::from_timestamp_secs(843057000).unwrap(),
            end: DateTime::from_timestamp_secs(843256800).unwrap(),
            status: EventStatus::Confirmed,
            catagory: "CONFERENCE".into(),
            organizer: "mailto:jsmith@example.com".into(),
            summery: "Networld+Interop Conference".into(),
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
}
