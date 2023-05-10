use super::endpoints::metrics_endpoint::{MakeMetricsEndpointService, MetricsEndpointService};
use super::endpoints::openapi_endpoint::{MakeOpenAPIEndpointService, OpenAPIEndpointService};
use super::ui::{MakeUIService, UIService};
use super::{ServiceError, ServiceFuture, OPENAPI_URL};
use futures::executor::block_on;
use futures::future;
use hyper::service::Service as HyperService;
use hyper::{Body, Request, Response, StatusCode};
use log::debug;
use server_lib::server::MakeService;
use server_lib::{Api, Service};
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

pub struct MakeRouterService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    inner_api: MakeService<Inner, C>,
    inner_openapi: MakeOpenAPIEndpointService<C>,
    inner_metrics: MakeMetricsEndpointService<C>,
    inner_ui: MakeUIService<C>,
    marker: PhantomData<C>,
}

impl<Inner, C> MakeRouterService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(
        inner_api: MakeService<Inner, C>,
        inner_openapi: MakeOpenAPIEndpointService<C>,
        inner_metrics: MakeMetricsEndpointService<C>,
        inner_ui: MakeUIService<C>,
    ) -> Self {
        Self {
            inner_api,
            inner_openapi,
            inner_metrics,
            inner_ui,
            marker: PhantomData,
        }
    }

    fn run<Target>(&mut self, target: Target) -> Result<HeaderService<Inner, C>, ServiceError>
    where
        Target: Clone + Send,
    {
        let api = self.inner_api.call(target.clone());
        let openapi = self.inner_openapi.call(target.clone());
        let metrics = self.inner_metrics.call(target.clone());
        let ui = self.inner_ui.call(target);

        let future = async {
            let api = api.await;
            let openapi = openapi.await;
            let metrics = metrics.await;
            let ui = ui.await;
            (api, openapi, metrics, ui)
        };

        let (api, openapi, metrics, ui) = block_on(future);

        Ok(HeaderService::new(api?, openapi?, metrics?, ui?))
    }
}

impl<Inner, C, Target> hyper::service::Service<Target> for MakeRouterService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    Target: Clone + Send,
{
    type Response = HeaderService<Inner, C>;
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
pub struct HeaderService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    api: Service<Inner, C>,
    openapi: OpenAPIEndpointService<C>,
    metrics: MetricsEndpointService<C>,
    ui: UIService<C>,
    marker: PhantomData<C>,
}

impl<Inner, C> HeaderService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(
        api: Service<Inner, C>,
        openapi: OpenAPIEndpointService<C>,
        metrics: MetricsEndpointService<C>,
        ui: UIService<C>,
    ) -> Self {
        Self {
            api,
            openapi,
            metrics,
            ui,
            marker: PhantomData,
        }
    }
}

impl<Inner, C> hyper::service::Service<(Request<Body>, C)> for HeaderService<Inner, C>
where
    Inner: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.api.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        let (request, context) = req;

        let xspanid = <C as Has<XSpanIdString>>::get(&context).0.clone();

        let path = request.uri().path();
        debug!("Routing {path}");
        if path.starts_with("/api/") {
            debug!("Routing to api");
            self.api.call((request, context))
        } else if path == OPENAPI_URL {
            debug!("Routing to openapi specs");
            self.openapi.call((request, context))
        } else if path == "/metrics" {
            debug!("Routing to metrics");
            self.metrics.call((request, context))
        } else if path.is_empty() || path == "/" || path == "/ui" || path == "/ui/" {
            async fn run(xspanid: String) -> Result<Response<Body>, ServiceError> {
                let response = Response::builder()
                    .status(StatusCode::PERMANENT_REDIRECT)
                    .header("Location", "/ui/index.html")
                    .header("x-span-id", xspanid.as_str())
                    .body(Body::empty())
                    .expect("Unable to build response");
                Ok(response)
            }
            Box::pin(run(xspanid))
        } else if path == "/favicon.ico" || path.starts_with("/ui/") {
            debug!("Routing to ui");
            self.ui.call((request, context))
        } else {
            async fn run(xspanid: String) -> Result<Response<Body>, ServiceError> {
                super::not_found(xspanid)
            }
            Box::pin(run(xspanid))
        }
    }
}
