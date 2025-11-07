use actix_web::{
    HttpResponse,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::InternalError,
};
use anyhow::Result;
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};

#[derive(Default)]
pub struct CalDavAuth {}

impl<S, B> Transform<S, ServiceRequest> for CalDavAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CalDavAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CalDavAuthMiddleware { service }))
    }
}

#[derive(Default)]
pub struct CalDavAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CalDavAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth = req.headers().iter().find(|h| h.0 == "Authorization");

        match auth {
            Some((_, status)) if status == "Authorized" => {
                let fut = self.service.call(req);

                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            _ => Box::pin(async move {
                let response = HttpResponse::Unauthorized()
                    .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                    .finish();

                Err(InternalError::from_response("Unauthorized", response).into())
            }),
        }
    }
}
