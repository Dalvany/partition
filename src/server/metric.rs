use crate::server::headers::{HeadersService, MakeHeadersService};
use crate::server::{ServiceError, ServiceFuture};
use futures::executor::block_on;
use futures::future;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use log::error;
use metrics::increment_counter;
use server_lib::Api;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

pub const RESPONSE_COUNT: &str = "response_count";

pub struct MakeMetricsService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    inner: MakeHeadersService<Inner, C>,
    marker: PhantomData<C>,
}

impl<Inner, C> MakeMetricsService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(inner: MakeHeadersService<Inner, C>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    fn run<Target>(&mut self, target: Target) -> Result<MetricsService<Inner, C>, ServiceError>
    where
        Target: Clone + Send,
    {
        let inner = self.inner.call(target);

        let future = async { inner.await };

        let inner = block_on(future);

        Ok(MetricsService::new(inner?))
    }
}

impl<Inner, C, Target> Service<Target> for MakeMetricsService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    Target: Clone + Send,
{
    type Response = MetricsService<Inner, C>;
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
pub struct MetricsService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    inner: HeadersService<Inner, C>,
    marker: PhantomData<C>,
}

impl<Inner, C> MetricsService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(inner: HeadersService<Inner, C>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, C> Service<(Request<Body>, C)> for MetricsService<Inner, C>
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

        let path = request.uri().path().to_string();

        let reponse = self.inner.call((request, context));

        let response = async move {
            let result = reponse.await;
            match result {
                Ok(response) => {
                    let status = response.status();
                    if status.as_u16() >= 400 {
                        error!("Error accessing {path}, status {status}");
                    }
                    increment_counter!(RESPONSE_COUNT, "http_status" => status.as_str().to_string(), "error" => "false");
                    Ok(response)
                }
                Err(error) => {
                    error!("{path} raise an error {error:?}");
                    // TODO status code 500 if error ? See how server respond
                    increment_counter!(RESPONSE_COUNT, "error" => "true");
                    Err(error)
                }
            }
        };

        Box::pin(response)
    }
}
