use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::icalendar::{
    ToIcalendarTimeStamp, if_some_write, if_some_write_param, parameter::IcalParam,
};

pub struct AudioAlarm {
    pub trigger: DateTime<Utc>,
    pub duration: Option<String>,
    pub repeat: Option<u32>,
    pub attach: Option<IcalParam<String>>,
}

pub struct DisplayAlarm {
    pub trigger: DateTime<Utc>,
    pub description: String,
    pub duration: Option<String>,
    pub repeat: Option<u32>,
}

pub struct EmailAlarm {
    pub trigger: DateTime<Utc>,
    pub description: String,
    pub summary: String,
    pub duration: Option<String>,
    pub repeat: Option<u32>,
}

pub enum Alarm {
    Audio(AudioAlarm),
    Display(DisplayAlarm),
    Email(EmailAlarm),
}

impl Display for Alarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VALARM\r\n")?;
        write!(
            f,
            "ACTION:{}\r\n",
            match &self {
                Alarm::Audio(_) => "AUDIO",
                Alarm::Display(_) => "DISPLAY",
                Alarm::Email(_) => "EMAIL",
            }
        )?;

        match &self {
            Alarm::Audio(alarm) => {
                write!(f, "TRIGGER:{}\r\n", alarm.trigger.to_icalendar_timestamp())?;
                if_some_write_param(f, "ATTACH", &alarm.attach)?;
                if_some_write(f, "REPEAT", &alarm.repeat)?;
                if_some_write(f, "DURATION", &alarm.duration)?;
            }
            Alarm::Display(alarm) => {
                write!(f, "TRIGGER:{}\r\n", alarm.trigger.to_icalendar_timestamp())?;
            }
            Alarm::Email(alarm) => {
                write!(f, "TRIGGER:{}\r\n", alarm.trigger.to_icalendar_timestamp())?;
            }
        };

        write!(f, "END:VALARM")
    }
}
