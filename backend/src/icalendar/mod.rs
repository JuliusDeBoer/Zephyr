use std::fmt::{self, Display, Formatter};

use chrono::DateTime;

use crate::icalendar::parameter::IcalParam;

mod alarm;
mod event;
mod parameter;
mod todo;

trait ToIcalendarTimeStamp {
    fn to_icalendar_timestamp(&self) -> String;
}

impl<Tz: chrono::TimeZone> ToIcalendarTimeStamp for DateTime<Tz>
where
    Tz::Offset: std::fmt::Display,
{
    fn to_icalendar_timestamp(&self) -> String {
        self.format("%Y%m%dT%H%M%SZ").to_string()
    }
}

pub enum UserType {
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
}

pub enum EncodingType {
    EightBit,
    BaseSixtyFour,
}

pub enum FreeBusyType {
    Free,
    Busy,
    BusyUnavailable,
    BusyAvailable,
}

/// A utility function to write to a [Formatter].
///
/// When [val] is `Some()` the writer will write `{}:{}\r\n` to the formatter.
/// Where the two arguments are `name` and `val.unwrap()`
fn wrap_icalendar_line(s: &str, width: usize) -> String {
    let mut result = String::new();
    let mut start = 0;
    let len = s.len();
    let mut first = true;
    while start < len {
        let end = usize::min(start + width, len);
        if first {
            result.push_str(&s[start..end]);
            first = false;
        } else {
            result.push_str("\r\n ");
            result.push_str(&s[start..end]);
        }
        start = end;
    }
    result
}

pub fn if_some_write(
    f: &mut Formatter<'_>,
    name: &'static str,
    val: &Option<impl Display>,
) -> fmt::Result {
    if let Some(val) = val {
        let line = format!("{}:{}", name, val);
        let wrapped = wrap_icalendar_line(&line, 75);
        return write!(f, "{}\r\n", wrapped);
    }
    fmt::Result::Ok(())
}

// NOTE(Julius): Now you *COULD* say this is a hack. However I choose to believe
//               its a bespoke solution.
pub fn if_some_write_param<T: Display>(
    f: &mut Formatter<'_>,
    name: &'static str,
    val: &Option<IcalParam<T>>,
) -> fmt::Result {
    if let Some(val) = val {
        let line = format!("{}{}", name, val);
        let wrapped = wrap_icalendar_line(&line, 75);
        return write!(f, "{}\r\n", wrapped);
    }
    fmt::Result::Ok(())
}

fn if_some_write_date(
    f: &mut Formatter<'_>,
    name: &'static str,
    val: &Option<impl ToIcalendarTimeStamp>,
) -> fmt::Result {
    if let Some(val) = val {
        let line = format!("{}:{}", name, val.to_icalendar_timestamp());
        let wrapped = wrap_icalendar_line(&line, 75);
        return write!(f, "{}\r\n", wrapped);
    }
    fmt::Result::Ok(())
}
