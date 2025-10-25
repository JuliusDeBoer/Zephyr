use std::{env, sync::Arc};

use actix_web::{App, HttpServer, dev::Service, web};
use dotenvy::dotenv;
use futures_util::future::FutureExt;
use sea_orm::Database;

mod auth;
mod entity;
mod icalendar;

fn get_db_string() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = Arc::new(
        Database::connect(get_db_string())
            .await
            .expect("Could not connect to database"),
    );

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .service(web::scope("/auth").configure(auth::configure))
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    match res {
                        Ok(ref v) => println!(
                            "[{}]: {} -> {}",
                            v.request().method(),
                            v.request().path(),
                            v.status()
                        ),
                        Err(ref e) => println!("{}", &e),
                    };
                    res
                })
            })
    })
    .bind(("127.0.0.1", 3000));

    if server.is_err() {
        println!("Could not bind to 127.0.0.1:3000");
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Could not bind to 127.0.0.1:3000",
        ));
    }

    println!("Hosting on 127.0.0.1:3000");
    server.unwrap().run().await
}
