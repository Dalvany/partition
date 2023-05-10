// @generated automatically by Diesel CLI.

diesel::table! {
    albums (id) {
        id -> Integer,
        name -> Varchar,
        year -> Nullable<Integer>,
        total_track -> Nullable<Integer>,
    }
}

diesel::table! {
    artists (id) {
        id -> Integer,
        name -> Varchar,
    }
}

diesel::table! {
    artists_albums (id) {
        id -> Integer,
        artists_id -> Integer,
        albums_id -> Integer,
    }
}

diesel::table! {
    playlists (id) {
        id -> Integer,
        name -> Varchar,
        query -> Nullable<Text>,
    }
}

diesel::table! {
    playlists_songs (id) {
        id -> Integer,
        playlists_id -> Integer,
        songs_id -> Integer,
        added -> Integer,
    }
}

diesel::table! {
    songs (id) {
        id -> Integer,
        albums_id -> Nullable<Integer>,
        name -> Varchar,
        genre -> Nullable<Varchar>,
        track -> Nullable<Integer>,
        duration -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        user_id -> Varchar,
        password -> Varchar,
    }
}

diesel::table! {
    users_playlists (id) {
        id -> Integer,
        users_id -> Integer,
        playlists_id -> Integer,
        shared -> Integer,
    }
}

diesel::table! {
    users_songs (id) {
        id -> Integer,
        users_id -> Integer,
        songs_id -> Integer,
        shared -> Integer,
    }
}

diesel::joinable!(artists_albums -> albums (albums_id));
diesel::joinable!(artists_albums -> artists (artists_id));
diesel::joinable!(playlists_songs -> playlists (playlists_id));
diesel::joinable!(playlists_songs -> songs (songs_id));
diesel::joinable!(songs -> albums (albums_id));
diesel::joinable!(users_playlists -> playlists (playlists_id));
diesel::joinable!(users_playlists -> users (users_id));
diesel::joinable!(users_songs -> songs (songs_id));
diesel::joinable!(users_songs -> users (users_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    artists_albums,
    playlists,
    playlists_songs,
    songs,
    users,
    users_playlists,
    users_songs,
);
