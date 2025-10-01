use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use std::default::Default;
use uuid::Uuid;

use crate::icalendar::{
    ToIcalendarTimeStamp, alarm::Alarm, if_some_write, if_some_write_date, if_some_write_param,
    parameter::IcalParam,
};

pub enum TodoStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    Completed,
    InProcess,
}

impl Display for TodoStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TodoStatus::NeedsAction => "NEEDS-ACTION",
                TodoStatus::Accepted => "ACCEPTED",
                TodoStatus::Declined => "DECLINED",
                TodoStatus::Tentative => "TENTATIVE",
                TodoStatus::Delegated => "DELEGATED",
                TodoStatus::Completed => "COMPLETED",
                TodoStatus::InProcess => "IN-PROCESS",
            }
        )
    }
}

pub struct Todo {
    pub uid: Uuid,

    pub timestamp: DateTime<Utc>,

    pub object_class: Option<String>,
    pub completed: Option<bool>,
    pub created: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub geo: Option<String>,
    pub last_mod: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub organizer: Option<String>,
    pub percent: Option<f32>,
    pub priority: Option<std::convert::Infallible>,
    pub recur_id: Option<String>,
    pub seq: Option<i32>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,

    pub rrule: Option<String>,
    pub atendee: Option<IcalParam<String>>,

    // Only one of these can be set at the same time.
    pub due: Option<DateTime<Utc>>,
    pub duration: Option<std::convert::Infallible>,

    // NOTE(Julius): I think there can be multiple alarms. But we can deal with
    //               that later...
    pub alarm: Option<Alarm>,
}

impl Display for Todo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BEGIN:VTODO\r\n")?;

        write!(
            f,
            "{}",
            format!(
                "UID:{}\r\n\
                DTSTAMP:{}\r\n",
                self.uid,
                self.timestamp.to_icalendar_timestamp()
            )
            .as_str(),
        )?;

        if_some_write(f, "SEQUENCE", &self.seq)?;
        if_some_write(f, "ORGANIZER", &self.organizer)?;
        if_some_write_param(f, "ATTENDEE", &self.atendee)?;
        if_some_write_date(f, "DUE", &self.due)?;
        if_some_write(f, "STATUS", &self.status)?;
        if_some_write(f, "SUMMARY", &self.summary)?;
        if let Some(alarm) = &self.alarm {
            write!(f, "{}\r\n", alarm)?;
        }
        write!(f, "END:VTODO")
    }
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            uid: Uuid::new_v4(),
            timestamp: Utc::now(),
            object_class: None,
            completed: None,
            created: None,
            description: None,
            start: None,
            geo: None,
            last_mod: None,
            location: None,
            organizer: None,
            percent: None,
            priority: None,
            recur_id: None,
            seq: None,
            status: None,
            summary: None,
            rrule: None,
            due: None,
            duration: None,
            atendee: None,
            alarm: None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::icalendar::{alarm::AudioAlarm, parameter::PartStat};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn serialize_todo_with_alarm() {
        let todo = Todo {
            uid: Uuid::from_str("3e5b9fe5-8c0f-46ff-930f-7b4f2afb0b38").unwrap(),
            timestamp: DateTime::from_timestamp_secs(886167900).unwrap(),
            due: Some(DateTime::from_timestamp_secs(892598400).unwrap()),
            seq: Some(2),
            organizer: Some("mailto:unclesam@example.com".into()),
            status: Some(TodoStatus::NeedsAction),
            atendee: Some(IcalParam {
                value: "mailto:jqpublic@example.com".into(),
                part_stat: Some(PartStat::Accepted),
                ..Default::default()
            }),
            summary: Some("Submit Income Taxes".into()),
            alarm: Some(Alarm::Audio(AudioAlarm {
                trigger: DateTime::from_timestamp_secs(891604800).unwrap(),
                duration: Some("PT1H".into()),
                repeat: Some(4),
                attach: Some(IcalParam {
                    value: "http://example.com/pub/audio-files/ssbanner.aud".into(),
                    fmt_type: Some("audio/basic".into()),
                    ..Default::default()
                }),
            })),
            ..Default::default()
        };

        let expected_result = "BEGIN:VTODO\r\n\
        UID:3e5b9fe5-8c0f-46ff-930f-7b4f2afb0b38\r\n\
        DTSTAMP:19980130T134500Z\r\n\
        SEQUENCE:2\r\n\
        ORGANIZER:mailto:unclesam@example.com\r\n\
        ATTENDEE;PARTSTAT=ACCEPTED:mailto:jqpublic@example.com\r\n\
        DUE:19980415T000000Z\r\n\
        STATUS:NEEDS-ACTION\r\n\
        SUMMARY:Submit Income Taxes\r\n\
        BEGIN:VALARM\r\n\
        ACTION:AUDIO\r\n\
        TRIGGER:19980403T120000Z\r\n\
        ATTACH;FMTTYPE=audio/basic:http://example.com/pub/audio-files/ssbanner.aud\r\n\
        REPEAT:4\r\n\
        DURATION:PT1H\r\n\
        END:VALARM\r\n\
        END:VTODO";

        assert_eq!(todo.to_string(), expected_result);
    }
}
