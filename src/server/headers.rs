//! Service that add custom headers in response
//! This is useful for CORS.
use crate::server::router::{HeaderService, MakeRouterService};
use crate::server::{ServiceError, ServiceFuture};
use futures::executor::block_on;
use futures::future;
use hyper::header::HeaderName;
use hyper::http::HeaderValue;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use server_lib::Api;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

pub struct MakeHeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    inner: MakeRouterService<Inner, C>,
    custom_headers: BTreeMap<String, String>,
    marker: PhantomData<C>,
}

impl<Inner, C> MakeHeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(
        inner: MakeRouterService<Inner, C>,
        custom_headers: BTreeMap<String, String>,
    ) -> Self {
        Self {
            inner,
            custom_headers,
            marker: PhantomData,
        }
    }

    fn run<Target>(&mut self, target: Target) -> Result<HeadersService<Inner, C>, ServiceError>
    where
        Target: Clone + Send,
    {
        let inner = self.inner.call(target);

        let future = async { inner.await };

        let inner = block_on(future);

        Ok(HeadersService::new(inner?, self.custom_headers.clone()))
    }
}

impl<Inner, C, Target> Service<Target> for MakeHeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    Target: Clone + Send,
{
    type Response = HeadersService<Inner, C>;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        let result = self.run(target);
        future::ready(result)
    }
}

#[derive(Clone)]
pub struct HeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    inner: HeaderService<Inner, C>,
    custom_headers: BTreeMap<String, String>,
    marker: PhantomData<C>,
}

impl<Inner, C> HeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(inner: HeaderService<Inner, C>, custom_headers: BTreeMap<String, String>) -> Self {
        Self {
            inner,
            custom_headers,
            marker: PhantomData,
        }
    }
}

impl<Inner, C> Service<(Request<Body>, C)> for HeadersService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        let (request, context) = req;

        let reponse = self.inner.call((request, context));
        let custom_header = self.custom_headers.clone();

        let response = async move {
            let result = reponse.await;
            match result {
                Ok(mut response) => {
                    let headers = response.headers_mut();
                    for (key, value) in custom_header {
                        let header_name = HeaderName::try_from(key)?;
                        headers.insert(header_name, HeaderValue::from_str(value.as_str())?);
                    }
                    Ok(response)
                }
                Err(error) => Err(error),
            }
        };

        Box::pin(response)
    }
}
