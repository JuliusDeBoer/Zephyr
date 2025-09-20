use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::icalendar::{ICalendarFreeBusyType, ICalendarObject, ICalendarUserType};

mod icalendar;

// TODO(Julius): Make sure that this deserializes correctly to have the same
//               names as in RFC 4791 4.1
pub enum CalendarObjectResourceType {
    Event,
    Todo,
    Journal,
    FreeBusy,
}

struct CalendarObjectResource {
    // TODO(Julius): Figure out the type for this
    pub uid: u32,
    pub object_type: CalendarObjectResourceType,
}

async fn hello(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<ICalendarObject>, Infallible> {
    dbg!(req.method());
    let out = ICalendarObject {
        alt_rep: String::new(),
        cn: "Cool name!".into(),
        cu_type: ICalendarUserType::Individual,
        del_from: String::new(),
        del_to: String::new(),
        dir: String::new(),
        encoding: icalendar::ICalendarEncodingType::BaseSixtyFour,
        fmt_type: String::new(),
        fb_type: ICalendarFreeBusyType::BusyUnavailable,
        language: String::new(),
        member: String::new(),
        part_stat: String::new(),
        range: 0,
        trig_rel: 0,
        rel_type: 0,
        role: 0,
        rsvp: false,
        sent_by_: 0,
        tzid: 0,
        value_type: 0,
        other: 0,
    };

    let mut response = Response::new(out);
    // TODO(Julius): Figure out how to not have to set the content type every
    //               time.
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/calendar"));
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    println!("Hosting on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving: {:?}", err);
            }
        });
    }
}
