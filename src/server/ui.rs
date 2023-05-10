use crate::server::{ServiceError, ServiceFuture};
use crate::METRIC_DISALLOWED_PATH;
use futures::future;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Response, StatusCode};
use log::{debug, warn};
use metrics::increment_counter;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::task::{Context, Poll};
use swagger::{Authorization, Has, XSpanIdString};

static PARTITION_ICON: &[u8] = include_bytes!("../../resources/ui/partition.ico");

#[derive(Clone)]
pub struct MakeUIService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    path: Option<PathBuf>,
    marker: PhantomData<C>,
}

impl<C> MakeUIService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            marker: PhantomData,
        }
    }
}

impl<C, Target> hyper::service::Service<Target> for MakeUIService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = UIService<C>;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: Target) -> Self::Future {
        future::ok(UIService::new(self.path.clone()))
    }
}

#[derive(Clone)]
pub struct UIService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    path: Option<PathBuf>,
    marker: PhantomData<C>,
}

impl<C> UIService<C>
where
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            marker: PhantomData,
        }
    }
}

impl<C> hyper::service::Service<(Request<Body>, C)> for UIService<C>
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
        let (request, context) = req;

        let xspanid = <C as Has<XSpanIdString>>::get(&context).0.clone();

        let path = request.uri().path();
        debug!("Serving {path}");
        if path == "/favicon.ico" {
            // Function to server favicon.
            async fn favicon(xspanid: String) -> Result<Response<Body>, ServiceError> {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("x-span-id", xspanid.as_str())
                    .body(Body::from(PARTITION_ICON))
                    .expect("Unable to build favicon");
                Ok(response)
            }

            Box::pin(favicon(xspanid))
        } else if let Some(ui_path) = self.path.as_ref() {
            // Function to server ui files.
            async fn file(
                ui_path: PathBuf,
                file_path: String,
                xspanid: String,
            ) -> Result<Response<Body>, ServiceError> {
                if file_path.contains("/..") {
                    let response = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("x-span-id", xspanid.as_str())
                        .body(Body::empty())
                        .expect("Unable to build response");
                    warn!("Path {file_path} contains '..' that are not allowed");
                    increment_counter!(METRIC_DISALLOWED_PATH);
                    return Ok(response);
                }
                let path = ui_path.join(file_path);
                if path.exists() {
                    debug!("Reading {path:?}");
                    let extension = path.extension();
                    match extension.and_then(|v| v.to_str()) {
                        Some("html") => {
                            let content = std::fs::read_to_string(path).expect("Can't read file");
                            let response = Response::builder()
                                .status(StatusCode::OK)
                                .header("x-span-id", xspanid.as_str())
                                .header(CONTENT_TYPE.as_str(), "text/html; charset=utf-8")
                                .body(Body::from(content))
                                .expect("Unable to build response");
                            Ok(response)
                        }
                        _ => todo!(),
                    }
                } else {
                    super::not_found(xspanid)
                }
            }

            let file_path = path.strip_prefix("/ui/").expect("Can't strip ui prefix");

            Box::pin(file(ui_path.clone(), file_path.to_string(), xspanid))
        } else {
            todo!()
        }
    }
}
