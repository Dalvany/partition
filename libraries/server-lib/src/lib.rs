#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::disallowed_names)]

use async_trait::async_trait;
use futures::Stream;
use std::error::Error;
use std::task::{Poll, Context};
use swagger::{ApiError, ContextWrapper};
use serde::{Serialize, Deserialize};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "/api/v1";
pub const API_VERSION: &str = "0.1.0";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PlaylistsGetResponse {
    /// List all playslists
    ListAllPlayslists
    ,
    /// Unexpected error
    UnexpectedError
    (Vec<models::Playlist>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PlaylistsIdDeleteResponse {
    /// Playlist deleted
    PlaylistDeleted
    ,
    /// Unknown playlist
    UnknownPlaylist
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PlaylistsIdGetResponse {
    /// Playlist
    Playlist
    (models::Playlist)
    ,
    /// Unknown playlist
    UnknownPlaylist
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PlaylistsPostResponse {
    /// Playlist successfully created
    PlaylistSuccessfullyCreated
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RootGetResponse {
    /// Ok
    Ok
    (models::Informations)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SearchGetResponse {
    /// List of song matching query
    ListOfSongMatchingQuery
    (Vec<models::Song>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SongsIdDeleteResponse {
    /// Song deleted
    SongDeleted
    ,
    /// Unknown song
    UnknownSong
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SongsIdGetResponse {
    /// Song metadata
    SongMetadata
    (models::Song)
    ,
    /// Unknown song
    UnknownSong
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SongsIdPutResponse {
    /// Playlist updated
    PlaylistUpdated
    ,
    /// Unknown playlist
    UnknownPlaylist
    ,
    /// Wrong data
    WrongData
    ,
    /// Unexpected error
    UnexpectedError
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SongsPostResponse {
    /// Successful operation
    SuccessfulOperation
    ,
    /// File format not supported
    FileFormatNotSupported
    ,
    /// Unexpected error
    UnexpectedError
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn playlists_get(
        &self,
        context: &C) -> Result<PlaylistsGetResponse, ApiError>;

    async fn playlists_id_delete(
        &self,
        id: i32,
        context: &C) -> Result<PlaylistsIdDeleteResponse, ApiError>;

    async fn playlists_id_get(
        &self,
        id: i32,
        context: &C) -> Result<PlaylistsIdGetResponse, ApiError>;

    async fn playlists_post(
        &self,
        playlist: models::Playlist,
        context: &C) -> Result<PlaylistsPostResponse, ApiError>;

    async fn root_get(
        &self,
        context: &C) -> Result<RootGetResponse, ApiError>;

    async fn search_get(
        &self,
        q: String,
        limit: Option<i32>,
        offset: Option<i32>,
        context: &C) -> Result<SearchGetResponse, ApiError>;

    async fn songs_id_delete(
        &self,
        id: i32,
        context: &C) -> Result<SongsIdDeleteResponse, ApiError>;

    async fn songs_id_get(
        &self,
        id: i32,
        context: &C) -> Result<SongsIdGetResponse, ApiError>;

    async fn songs_id_put(
        &self,
        id: i32,
        playlist: models::Playlist,
        context: &C) -> Result<SongsIdPutResponse, ApiError>;

    async fn songs_post(
        &self,
        x_filename: String,
        body: String,
        context: &C) -> Result<SongsPostResponse, ApiError>;

}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {

    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn playlists_get(
        &self,
        ) -> Result<PlaylistsGetResponse, ApiError>;

    async fn playlists_id_delete(
        &self,
        id: i32,
        ) -> Result<PlaylistsIdDeleteResponse, ApiError>;

    async fn playlists_id_get(
        &self,
        id: i32,
        ) -> Result<PlaylistsIdGetResponse, ApiError>;

    async fn playlists_post(
        &self,
        playlist: models::Playlist,
        ) -> Result<PlaylistsPostResponse, ApiError>;

    async fn root_get(
        &self,
        ) -> Result<RootGetResponse, ApiError>;

    async fn search_get(
        &self,
        q: String,
        limit: Option<i32>,
        offset: Option<i32>,
        ) -> Result<SearchGetResponse, ApiError>;

    async fn songs_id_delete(
        &self,
        id: i32,
        ) -> Result<SongsIdDeleteResponse, ApiError>;

    async fn songs_id_get(
        &self,
        id: i32,
        ) -> Result<SongsIdGetResponse, ApiError>;

    async fn songs_id_put(
        &self,
        id: i32,
        playlist: models::Playlist,
        ) -> Result<SongsIdPutResponse, ApiError>;

    async fn songs_post(
        &self,
        x_filename: String,
        body: String,
        ) -> Result<SongsPostResponse, ApiError>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync> where Self: Sized
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
         ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    async fn playlists_get(
        &self,
        ) -> Result<PlaylistsGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().playlists_get(&context).await
    }

    async fn playlists_id_delete(
        &self,
        id: i32,
        ) -> Result<PlaylistsIdDeleteResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().playlists_id_delete(id, &context).await
    }

    async fn playlists_id_get(
        &self,
        id: i32,
        ) -> Result<PlaylistsIdGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().playlists_id_get(id, &context).await
    }

    async fn playlists_post(
        &self,
        playlist: models::Playlist,
        ) -> Result<PlaylistsPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().playlists_post(playlist, &context).await
    }

    async fn root_get(
        &self,
        ) -> Result<RootGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().root_get(&context).await
    }

    async fn search_get(
        &self,
        q: String,
        limit: Option<i32>,
        offset: Option<i32>,
        ) -> Result<SearchGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().search_get(q, limit, offset, &context).await
    }

    async fn songs_id_delete(
        &self,
        id: i32,
        ) -> Result<SongsIdDeleteResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().songs_id_delete(id, &context).await
    }

    async fn songs_id_get(
        &self,
        id: i32,
        ) -> Result<SongsIdGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().songs_id_get(id, &context).await
    }

    async fn songs_id_put(
        &self,
        id: i32,
        playlist: models::Playlist,
        ) -> Result<SongsIdPutResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().songs_id_put(id, playlist, &context).await
    }

    async fn songs_post(
        &self,
        x_filename: String,
        body: String,
        ) -> Result<SongsPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().songs_post(x_filename, body, &context).await
    }

}


#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
