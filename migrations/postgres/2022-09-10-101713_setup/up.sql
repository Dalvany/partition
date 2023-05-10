CREATE TABLE users
(
    id       SERIAL PRIMARY KEY,
    user_id  VARCHAR(50) NOT NULL UNIQUE,
    -- MD5
    password VARCHAR(32) NOT NULL
);

INSERT INTO users (user_id, password)
VALUES ('admin', '456b7016a916a4b178dd72b947c152b7');

CREATE TABLE artists
(
    id   SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE albums
(
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(50) NOT NULL,
    year        INTEGER,
    total_track INTEGER
);

CREATE TABLE songs
(
    id       SERIAL PRIMARY KEY,
    albums_id INTEGER,
    name     VARCHAR(50) NOT NULL,
    genre    VARCHAR(50),
    track    INTEGER,
    duration INTEGER  NOT NULL,
    FOREIGN KEY (albums_id) REFERENCES albums (id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE artists_albums
(
    id        SERIAL PRIMARY KEY,
    artists_id INTEGER NOT NULL,
    albums_id  INTEGER NOT NULL,
    FOREIGN KEY (artists_id) REFERENCES artists (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (albums_id) REFERENCES albums (id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE playlists
(
    id    SERIAL PRIMARY KEY,
    name  VARCHAR(50) NOT NULL,
    query TEXT
);

-- playlist_song.add : 1 if song is added to playlist, 0 if remove
-- this is to allow removing or adding particular songs from a query playlist.
CREATE TABLE playlists_songs
(
    id          SERIAL PRIMARY KEY,
    playlists_id INTEGER NOT NULL,
    songs_id     INTEGER NOT NULL,
    added       INTEGER NOT NULL,
    FOREIGN KEY (playlists_id) REFERENCES playlists (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (songs_id) REFERENCES songs (id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE users_songs
(
    id      SERIAL PRIMARY KEY,
    users_id INTEGER NOT NULL,
    songs_id INTEGER NOT NULL,
    shared  INTEGER NOT NULL,
    FOREIGN KEY (users_id) REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (songs_id) REFERENCES songs (id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE users_playlists
(
    id          SERIAL PRIMARY KEY,
    users_id     INTEGER NOT NULL,
    playlists_id INTEGER NOT NULL,
    shared      INTEGER NOT NULL,
    FOREIGN KEY (users_id) REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (playlists_id) REFERENCES playlists (id) ON UPDATE CASCADE ON DELETE CASCADE
);
