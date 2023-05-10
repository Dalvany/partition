use crate::server::{ServiceError, ServiceFuture, OPENAPI_URL};
use futures::future;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Response, StatusCode};
use log::debug;
use okapi::openapi3::SchemaObject;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

static OPENAPI: &str = include_str!("../../../docs/openapi.yml");

#[derive(Clone, Debug, Default)]
pub struct MakeOpenAPIEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    marker: PhantomData<C>,
}

impl<C, Target> hyper::service::Service<Target> for MakeOpenAPIEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = OpenAPIEndpointService<C>;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: Target) -> Self::Future {
        future::ok(OpenAPIEndpointService::new())
    }
}

#[derive(Clone, Debug)]
pub struct OpenAPIEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    marker: PhantomData<C>,
}

impl<C> OpenAPIEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<C> hyper::service::Service<(Request<Body>, C)> for OpenAPIEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        async fn run<C>(req: (Request<Body>, C)) -> Result<Response<Body>, ServiceError>
        where
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
        {
            let (request, context) = req;

            let xspanid = <C as Has<XSpanIdString>>::get(&context).0.clone();

            let path = request.uri().path();
            debug!("Serving {path}");
            if path == OPENAPI_URL {
                let content: SchemaObject =
                    serde_yaml::from_str(OPENAPI).expect("Can't read openapi file");
                let content = serde_json::to_string(&content).expect("Can't convert openapi file");
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("x-span-id", xspanid.as_str())
                    .header(CONTENT_TYPE.as_str(), "application/json")
                    .body(Body::from(content))
                    .expect("Unable to build openapi.json");
                Ok(response)
            } else {
                super::super::not_found(xspanid)
            }
        }
        Box::pin(run(req))
    }
}
