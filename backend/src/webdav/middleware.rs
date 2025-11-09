use actix_web::{
    HttpResponse,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::InternalError,
    web,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::Result;
use futures_util::future::LocalBoxFuture;
use sea_orm::DatabaseConnection;
use std::{
    future::{Ready, ready},
    rc::Rc,
    sync::Arc,
};

use crate::jwt::validate_credentials;

#[derive(Default)]
pub struct CalDavAuth {}

impl<S, B> Transform<S, ServiceRequest> for CalDavAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CalDavAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CalDavAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CalDavAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CalDavAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let err = Box::pin(async move {
            let response = HttpResponse::Unauthorized()
                .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                .finish();
            Err(InternalError::from_response("Invalid credentials", response).into())
        });

        let auth = req.headers().iter().find(|h| h.0 == "Authorization");

        if auth.is_none() {
            return err;
        };

        let auth_header = auth.unwrap().1.to_str().unwrap_or("");
        let auth_value = if auth_header.starts_with("Basic ") {
            &auth_header[6..]
        } else {
            auth_header
        };

        let decoded: Vec<String> = match BASE64_STANDARD.decode(auth_value.as_bytes()) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded_str) => decoded_str
                    .split(":")
                    .map(|v: &str| String::from(v))
                    .collect(),
                Err(_) => {
                    return err;
                }
            },
            Err(_) => {
                return err;
            }
        };

        if decoded.len() < 2 {
            return err;
        }

        let db = req
            .app_data::<web::Data<Arc<DatabaseConnection>>>()
            .unwrap()
            .clone();

        let service = self.service.clone();
        let username = decoded[0].clone();
        let password = decoded[1].clone();

        Box::pin(async move {
            let db_ref = db.as_ref().as_ref();
            match validate_credentials(&username, &password, db_ref).await {
                Ok(valid) => {
                    if valid {
                        service.call(req).await
                    } else {
                        let response = HttpResponse::Unauthorized()
                            .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                            .finish();
                        Err(InternalError::from_response("Unauthorized", response).into())
                    }
                }
                Err(_) => {
                    let response = HttpResponse::InternalServerError().finish();
                    Err(InternalError::from_response("Authentication error", response).into())
                }
            }
        })
    }
}
