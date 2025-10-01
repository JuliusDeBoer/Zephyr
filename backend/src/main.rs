use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

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

async fn hello(req: Request<hyper::body::Incoming>) -> Result<Response<String>, Infallible> {
    dbg!(req.method());
    Ok(Response::new("Heya".into()))
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
