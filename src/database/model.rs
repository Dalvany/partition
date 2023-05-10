use super::schema::{albums, artists, artists_albums, users};
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[diesel(table_name = artists)]
pub(crate) struct Artists {
    id: i32,
    name: String,
}

#[derive(
    Identifiable, Selectable, Queryable, Associations, Clone, Debug, Ord, PartialOrd, Eq, PartialEq,
)]
#[diesel(belongs_to(Artists))]
#[diesel(belongs_to(Albums))]
#[diesel(table_name = artists_albums)]
#[diesel(primary_key(artists_id, albums_id))]
pub struct ArtistsAlbums {
    id: i32,
    artists_id: i32,
    albums_id: i32,
}

#[derive(Queryable, Identifiable, Selectable, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[diesel(table_name = albums)]
pub(crate) struct Albums {
    id: i32,
    name: String,
    year: Option<i32>,
    total_track: Option<i32>,
}

#[derive(Queryable, Identifiable, Selectable, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[diesel(table_name = users)]
pub(crate) struct Users {
    id: i32,
    user_id: String,
    password: String,
}
