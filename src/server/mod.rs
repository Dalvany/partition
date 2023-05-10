use crate::config::MainConfig;
use crate::index::TantivyIndex;
use crate::library::Library;
use crate::METRIC_DISALLOWED_PATH;
use anyhow::Result;
use endpoints::api_endpoint::Server;
use endpoints::metrics_endpoint::MakeMetricsEndpointService;
use endpoints::openapi_endpoint::MakeOpenAPIEndpointService;
use futures::future::BoxFuture;
use headers::MakeHeadersService;
use hyper::{Body, Response, StatusCode};
use log::{info, warn};
use mdc::MakeMDCService;
use metric::{MakeMetricsService, RESPONSE_COUNT};
use metrics::{describe_counter, describe_histogram, Unit};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use router::MakeRouterService;
use server_lib::server::MakeService;
use std::error::Error;
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use ui::MakeUIService;

mod authenticator;
mod endpoints;
mod headers;
mod mdc;
mod metric;
mod router;
mod ui;

static OPENAPI_URL: &str = "/openapi.json";

type ServiceFuture = BoxFuture<'static, Result<Response<Body>, ServiceError>>;
type ServiceError = Box<dyn Error + Send + Sync + 'static>;

fn not_found(xspanid: String) -> Result<Response<Body>, ServiceError> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("x-span-id", xspanid.as_str())
        .body(Body::empty())
        .expect("Unable to build response");
    Ok(response)
}

pub(crate) async fn create(
    addr: &str,
    tantivy_index: TantivyIndex,
    config: MainConfig,
) -> Result<()> {
    let addr = addr.parse().expect("Failed to parse bind address");

    //let db = config.database();
    //let connection = crate::database::Database::try_from(db)?;
    //connection.authenticate_user("admin", "admin")?;

    // Expose API
    let library: Library = config.library().into();
    let server = Server::new(tantivy_index, library)?;
    let api = MakeService::new(server);

    // Expose openapi spec in json
    let openapi = MakeOpenAPIEndpointService::default();

    // Expose metrics
    let bucket = [0.05, 0.1, 0.5, 1.0, 2.0, 5.0];
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(Matcher::Suffix("api_time".to_owned()), &bucket)
        .unwrap_or_else(|error| {
            warn!("Can't set bucket for 'api_time' metrics, defaulting to summary : {error:?}");
            PrometheusBuilder::new()
        })
        .install_recorder()
        .expect("failed to install recorder");
    let metrics = MakeMetricsEndpointService::new(handle);

    describe_counter!(
        METRIC_DISALLOWED_PATH,
        "Request count for path that contains '..'."
    );
    describe_counter!(RESPONSE_COUNT, "Response count by http status");
    describe_histogram!("api_time", Unit::Seconds, "API implementation time");

    // Expose ui and favicon
    let path = config.ui().map(|ui| ui.path());
    let ui = MakeUIService::new(path);

    // Route between different endpoint (api, openapi spec, metrics, ...etc)
    let service = MakeRouterService::new(api, openapi, metrics, ui);

    // Headers service
    let service = MakeHeadersService::new(service, config.headers());

    // Add metric service
    let service = MakeMetricsService::new(service);

    // Add MDC, especially set X-Span-ID in MDC
    let service = MakeMDCService::new(service);

    // TODO Change this to an authentication service...
    let service = MakeAllowAllAuthenticator::new(service, "admin");

    let service = server_lib::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    info!("Ready to server on {addr}");

    hyper::server::Server::bind(&addr).serve(service).await?;

    Ok(())
}
