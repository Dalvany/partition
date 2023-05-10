//! Main library entry point for server_lib implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::{Has, XSpanIdString};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use server_lib::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        server_lib::server::context::MakeAddContext::<_, EmptyContext>::new(
            service
        );

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM).expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem").expect("Failed to set certificate chain");
            ssl.check_private_key().expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr).serve(service).await.unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server{marker: PhantomData}
    }
}


use server_lib::{
    Api,
    PlaylistsGetResponse,
    PlaylistsIdDeleteResponse,
    PlaylistsIdGetResponse,
    PlaylistsPostResponse,
    RootGetResponse,
    SearchGetResponse,
    SongsIdDeleteResponse,
    SongsIdGetResponse,
    SongsIdPutResponse,
    SongsPostResponse,
};
use server_lib::server::MakeService;
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C> where C: Has<XSpanIdString> + Send + Sync
{
    async fn playlists_get(
        &self,
        context: &C) -> Result<PlaylistsGetResponse, ApiError>
    {
        let context = context.clone();
        info!("playlists_get() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_id_delete(
        &self,
        id: i32,
        context: &C) -> Result<PlaylistsIdDeleteResponse, ApiError>
    {
        let context = context.clone();
        info!("playlists_id_delete({}) - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_id_get(
        &self,
        id: i32,
        context: &C) -> Result<PlaylistsIdGetResponse, ApiError>
    {
        let context = context.clone();
        info!("playlists_id_get({}) - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_post(
        &self,
        playlist: models::Playlist,
        context: &C) -> Result<PlaylistsPostResponse, ApiError>
    {
        let context = context.clone();
        info!("playlists_post({:?}) - X-Span-ID: {:?}", playlist, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn root_get(
        &self,
        context: &C) -> Result<RootGetResponse, ApiError>
    {
        let context = context.clone();
        info!("root_get() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn search_get(
        &self,
        q: String,
        limit: Option<i32>,
        offset: Option<i32>,
        context: &C) -> Result<SearchGetResponse, ApiError>
    {
        let context = context.clone();
        info!("search_get(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}", q, limit, offset, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_delete(
        &self,
        id: i32,
        context: &C) -> Result<SongsIdDeleteResponse, ApiError>
    {
        let context = context.clone();
        info!("songs_id_delete({}) - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_get(
        &self,
        id: i32,
        context: &C) -> Result<SongsIdGetResponse, ApiError>
    {
        let context = context.clone();
        info!("songs_id_get({}) - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_put(
        &self,
        id: i32,
        playlist: models::Playlist,
        context: &C) -> Result<SongsIdPutResponse, ApiError>
    {
        let context = context.clone();
        info!("songs_id_put({}, {:?}) - X-Span-ID: {:?}", id, playlist, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_post(
        &self,
        x_filename: String,
        body: String,
        context: &C) -> Result<SongsPostResponse, ApiError>
    {
        let context = context.clone();
        info!("songs_post(\"{}\", \"{}\") - X-Span-ID: {:?}", x_filename, body, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

}
