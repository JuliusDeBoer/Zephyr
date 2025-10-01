use std::fmt::{self, Display};

pub enum PartStat {
    // For both events and todos
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delagated,

    // Todos only
    Completed,
    InProgress,
}

impl Display for PartStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PartStat::NeedsAction => "NEEDS-ACTION",
                PartStat::Accepted => "ACCEPTED",
                PartStat::Declined => "DECLINED",
                PartStat::Tentative => "TENTATIVE",
                PartStat::Delagated => "DELAGATED",
                PartStat::Completed => "COMPLETED",
                PartStat::InProgress => "IN-PROGRESS",
            }
        )
    }
}

// NOTE(Julius): It might be a better idea to keep an array for this.
pub struct IcalParam<T> {
    /// Alternate text representation
    pub alt_rep: Option<String>,
    /// Common name
    pub cn: Option<String>,
    /// Calendar user type
    pub cut_ype: Option<u32>,
    /// Delegator
    pub del_from: Option<u32>,
    /// Delegatee
    pub del_to: Option<u32>,
    /// Directory entry
    pub dir: Option<u32>,
    /// Inline encoding
    pub encoding: Option<u32>,
    /// Format type
    pub fmt_type: Option<String>,
    /// Free busy: u32, time type
    pub fb_type: Option<u32>,
    /// Language for text
    pub language: Option<u32>,
    /// Group or list membership
    pub member: Option<u32>,
    /// Participation status
    pub part_stat: Option<PartStat>,
    /// Recurrence identifier range
    pub range: Option<u32>,
    /// Alarm trigger relationship
    pub trig_rel: Option<u32>,
    /// Relationship type
    pub rel_type: Option<u32>,
    /// Participation role
    pub role: Option<u32>,
    /// RSVP expectation
    pub rsvp: Option<u32>,
    /// Sent by
    pub sent_by: Option<u32>,
    /// Reference to time zone object
    pub tz_id: Option<u32>,
    /// Property value data type
    pub value_type: Option<u32>,

    pub value: T,
}

impl<T> IcalParam<T> {
    pub fn new(value: T) -> IcalParam<T> {
        Self {
            alt_rep: Default::default(),
            cn: Default::default(),
            cut_ype: Default::default(),
            del_from: Default::default(),
            del_to: Default::default(),
            dir: Default::default(),
            encoding: Default::default(),
            fmt_type: Default::default(),
            fb_type: Default::default(),
            language: Default::default(),
            member: Default::default(),
            part_stat: Default::default(),
            range: Default::default(),
            trig_rel: Default::default(),
            rel_type: Default::default(),
            role: Default::default(),
            rsvp: Default::default(),
            sent_by: Default::default(),
            tz_id: Default::default(),
            value_type: Default::default(),
            value: value,
        }
    }
}

impl<T: Default> Default for IcalParam<T> {
    fn default() -> Self {
        Self {
            alt_rep: Default::default(),
            cn: Default::default(),
            cut_ype: Default::default(),
            del_from: Default::default(),
            del_to: Default::default(),
            dir: Default::default(),
            encoding: Default::default(),
            fmt_type: Default::default(),
            fb_type: Default::default(),
            language: Default::default(),
            member: Default::default(),
            part_stat: Default::default(),
            range: Default::default(),
            trig_rel: Default::default(),
            rel_type: Default::default(),
            role: Default::default(),
            rsvp: Default::default(),
            sent_by: Default::default(),
            tz_id: Default::default(),
            value_type: Default::default(),
            value: Default::default(),
        }
    }
}

impl<T: Display> Display for IcalParam<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(part_stat) = &self.part_stat {
            write!(f, ";PARTSTAT={}", part_stat)?;
        }

        if let Some(fmt_type) = &self.fmt_type {
            write!(f, ";FMTTYPE={}", fmt_type)?;
        }
        write!(f, ":{}", self.value)?;
        fmt::Result::Ok(())
    }
}
