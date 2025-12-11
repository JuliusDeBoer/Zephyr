use std::{env, sync::Arc};

use actix_web::{App, HttpServer, middleware, web};
use dotenvy::dotenv;
use env_logger::Env;
use sea_orm::Database;

use crate::endpoints::rest::auth;
use crate::endpoints::webdav::{caldav, middleware::CalDavAuth};

mod controller;
mod endpoints;
mod entity;
mod logic;
mod serialization;

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

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .service(web::scope("/auth").configure(auth::configure))
            .service(
                web::scope("/caldav")
                    .configure(caldav::configure)
                    .wrap(CalDavAuth::default()),
            )
    })
    .bind(("127.0.0.1", 3000));

    match server {
        Ok(s) => s.run().await,
        Err(e) => {
            println!("Could not bind to 127.0.0.1:3000: {e}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "Could not bind to 127.0.0.1:3000",
            ));
        }
    }
}
