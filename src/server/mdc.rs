use std::marker::PhantomData;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::FutureExt;
use hyper::service::Service;
use hyper::{Body, Request};
use swagger::{Authorization, Has, XSpanIdString};

#[derive(Debug)]
pub struct MakeMDCService<Inner, RC> {
    inner: Inner,
    marker: PhantomData<RC>,
}

impl<Inner, RC> MakeMDCService<Inner, RC> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, RC, Target> Service<Target> for MakeMDCService<Inner, RC>
where
    Inner: Service<Target>,
    Inner::Future: Send + 'static,
{
    type Response = MDCService<Inner::Response, RC>;
    type Error = Inner::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, target: Target) -> Self::Future {
        Box::pin(self.inner.call(target).map(|s| Ok(MDCService::new(s?))))
    }
}

pub struct MDCService<Inner, C> {
    inner: Inner,
    marker: PhantomData<C>,
}

impl<Inner, C> MDCService<Inner, C> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, C> Service<(Request<Body>, C)> for MDCService<Inner, C>
where
    Inner: Service<(Request<Body>, C)>,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Inner::Response;
    type Error = Inner::Error;
    type Future = Inner::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        let (request, context) = req;
        // Not insert_scoped as it returns a future
        let _old = log_mdc::insert(
            "X-Span-ID",
            <C as Has<XSpanIdString>>::get(&context).0.clone(),
        );

        self.inner.call((request, context))
    }
}
