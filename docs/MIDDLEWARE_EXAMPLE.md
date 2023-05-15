# An example of MakeService and Service

It goes between an authenticator service and the service that wraps `Api`.

With this, we can manipulate the request and the response :
* logging request information
* add header to response
* ...etc

```rust
use std::error::Error;
use std::marker::PhantomData;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::FutureExt;
use hyper::http::HeaderValue;
use hyper::service::Service;
use hyper::{Request, Response};
use log::info;
use swagger::{Authorization, Has, XSpanIdString};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub struct MakeTest<T, C> {
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> MakeTest<T, C> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, RC, Target> Service<Target> for MakeTest<Inner, RC>
where
    Inner: Service<Target>,
    Inner::Future: Send + 'static,
{
    type Response = Test<Inner::Response, RC>;
    type Error = Inner::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, target: Target) -> Self::Future {
        Box::pin(self.inner.call(target).map(|s| Ok(Test::new(s?))))
    }
}

#[derive(Clone, Debug)]
pub struct Test<T, C> {
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> Test<T, C> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<B, T, C> Service<(Request<B>, C)> for Test<T, C>
where
    T: Service<
            (Request<B>, C),
            Response = Response<B>,
            Error = ServiceError,
            Future = BoxFuture<'static, Result<Response<B>, ServiceError>>,
        > + Clone
        + Send
        + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    B: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = BoxFuture<'static, Result<Response<B>, ServiceError>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<B>, C)) -> Self::Future {
        async fn test<B, T, C>(
            mut inner: T,
            req: (Request<B>, C),
        ) -> Result<Response<B>, ServiceError>
        where
            T: Service<
                    (Request<B>, C),
                    Response = Response<B>,
                    Error = ServiceError,
                    Future = BoxFuture<'static, Result<Response<B>, ServiceError>>,
                > + Clone
                + Send
                + 'static,
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
            B: Send + 'static,
        {
            info!("Doing stuff on request inside async function");

            let result = inner.call(req).await;

            info!("Doing stuff on response");
            let result = match result {
                Ok(mut response) => {
                    response
                        .headers_mut()
                        .insert("toto", HeaderValue::from_static("tata"));
                    Ok(response)
                }
                Err(error) => Err(error),
            };

            result
        }
        
        let (request, context) = req;
        info!("Doing stuff on request inside call function");
        Box::pin(test(self.inner.clone(), (request, context)))
    }
}

```