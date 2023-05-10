use crate::index::TantivyIndex;
use crate::library::Library;
use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use function_timer::time;
use log::{debug, info, warn};
use server_lib::models::Informations;
use server_lib::{
    models, Api, PlaylistsGetResponse, PlaylistsIdDeleteResponse, PlaylistsIdGetResponse,
    PlaylistsPostResponse, RootGetResponse, SearchGetResponse, SongsIdDeleteResponse,
    SongsIdGetResponse, SongsIdPutResponse, SongsPostResponse,
};
use std::marker::PhantomData;
use std::sync::Arc;
use swagger::{ApiError, Has, XSpanIdString};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Server<C> {
    index: Arc<TantivyIndex>,
    library: Library,
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub(crate) fn new(tantivy_index: TantivyIndex, library: Library) -> Result<Self> {
        library.create_folder()?;
        Ok(Server {
            index: Arc::new(tantivy_index),
            library,
            marker: PhantomData,
        })
    }
}

#[async_trait]
#[time("api_time")]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    async fn playlists_get(&self, _context: &C) -> Result<PlaylistsGetResponse, ApiError> {
        info!("playlists_get()");
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_id_delete(
        &self,
        id: i32,
        _context: &C,
    ) -> Result<PlaylistsIdDeleteResponse, ApiError> {
        info!("playlists_id_delete({id})");
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_id_get(
        &self,
        id: i32,
        _context: &C,
    ) -> Result<PlaylistsIdGetResponse, ApiError> {
        info!("playlists_id_get({id})");
        Err(ApiError("Generic failure".into()))
    }

    async fn playlists_post(
        &self,
        playlist: models::Playlist,
        _context: &C,
    ) -> Result<PlaylistsPostResponse, ApiError> {
        info!("playlists_post({playlist:?})");
        Err(ApiError("Generic failure".into()))
    }

    async fn root_get(&self, _context: &C) -> Result<RootGetResponse, ApiError> {
        info!("root_get()");
        let version = Some(VERSION.to_string());
        Ok(RootGetResponse::Ok(Informations { version }))
    }

    async fn search_get(
        &self,
        q: String,
        limit: Option<i32>,
        offset: Option<i32>,
        _context: &C,
    ) -> Result<SearchGetResponse, ApiError> {
        info!("search_get(\"{}\", {:?}, {:?})", q, limit, offset);

        let limit = limit.unwrap_or(10);
        let limit = usize::try_from(limit).unwrap_or(0);
        let offset = offset.unwrap_or(0);
        let offset = usize::try_from(offset).unwrap_or(0);

        self.index.search(q, offset, limit).map_err(|error| {
            warn!("Error while searching : {error:?}");
            ApiError(format!("Error while searching : {error:?}"))
        })?;

        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_delete(
        &self,
        id: i32,
        _context: &C,
    ) -> Result<SongsIdDeleteResponse, ApiError> {
        info!("songs_id_delete({id})");
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_get(&self, id: i32, _context: &C) -> Result<SongsIdGetResponse, ApiError> {
        info!("songs_id_get({id})");
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_id_put(
        &self,
        id: i32,
        playlist: models::Playlist,
        _context: &C,
    ) -> Result<SongsIdPutResponse, ApiError> {
        info!("songs_id_put({id}, {playlist:?})");
        Err(ApiError("Generic failure".into()))
    }

    async fn songs_post(
        &self,
        x_filename: String,
        body: String,
        _context: &C,
    ) -> Result<SongsPostResponse, ApiError> {
        info!("songs_post(\"{x_filename}\")");
        let path = self.library.temporary_path().join(&x_filename);
        let file = base64::engine::general_purpose::STANDARD
            .decode(body)
            .map_err(|error| {
                warn!("Error uploading song : {error:?}");
                ApiError(error.to_string())
            })?;
        std::fs::write(&path, file).map_err(|error| {
            warn!("Error uploading song : {error:?}");
            ApiError(error.to_string())
        })?;

        let song = crate::library::Song::try_from(path)?;

        // TODO insert into database to get an id.

        // TODO move file from tmp into library

        // Index
        let op = self.index.index(song).map_err(|error| {
            warn!("Can't index \"{x_filename}\" : {error:?}");
            ApiError(format!("Can't index \"{x_filename}\" : {error:?}"))
        })?;
        debug!("Index \"{x_filename}\" {op}");

        Ok(SongsPostResponse::SuccessfulOperation)
    }
}
