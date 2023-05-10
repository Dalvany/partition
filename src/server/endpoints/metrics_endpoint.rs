use crate::server::{ServiceError, ServiceFuture};
use futures::future;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Response, StatusCode};
use metrics_exporter_prometheus::PrometheusHandle;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

#[derive(Clone)]
pub struct MakeMetricsEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    handle: PrometheusHandle,
    marker: PhantomData<C>,
}

impl<C> MakeMetricsEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(handle: PrometheusHandle) -> Self {
        Self {
            handle,
            marker: PhantomData,
        }
    }
}

impl<C, Target> hyper::service::Service<Target> for MakeMetricsEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = MetricsEndpointService<C>;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: Target) -> Self::Future {
        future::ok(MetricsEndpointService::new(self.handle.clone()))
    }
}

#[derive(Clone)]
pub struct MetricsEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    handle: PrometheusHandle,
    marker: PhantomData<C>,
}

impl<C> MetricsEndpointService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(handle: PrometheusHandle) -> Self {
        Self {
            handle,
            marker: PhantomData,
        }
    }
}

impl<C> hyper::service::Service<(Request<Body>, C)> for MetricsEndpointService<C>
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
        async fn run<C>(metrics: String, context: C) -> Result<Response<Body>, ServiceError>
        where
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
        {
            let xspanid = <C as Has<XSpanIdString>>::get(&context).0.clone();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("x-span-id", xspanid.as_str())
                .header(CONTENT_TYPE.as_str(), "text/plain")
                .body(Body::from(metrics))
                .expect("Unable to build favicon");
            Ok(response)
        }
        let (_, context) = req;
        let metrics = self.handle.render();
        Box::pin(run(metrics, context))
    }
}
