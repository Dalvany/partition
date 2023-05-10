use std::path::PathBuf;

use audiotags::Tag;
use log::warn;
use swagger::ApiError;
use tantivy::schema::Schema;
use tantivy::Document;

use crate::index::PartitionFields;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Song(server_lib::models::Song);

impl Song {
    pub(crate) fn into_document(self, schema: &Schema) -> Document {
        let mut document = Document::new();

        document.add_text(
            schema
                .get_field(PartitionFields::Title.field_name())
                .unwrap(),
            &self.title(),
        );

        document.add_text(
            schema
                .get_field(PartitionFields::Album.field_name())
                .unwrap(),
            &self.album(),
        );

        document.add_text(
            schema
                .get_field(PartitionFields::Artist.field_name())
                .unwrap(),
            &self.artist(),
        );

        if let Some(id) = self.id() {
            document.add_i64(
                schema.get_field(PartitionFields::Id.field_name()).unwrap(),
                id as i64,
            );
        }

        document
    }

    pub(crate) fn title(&self) -> String {
        self.0
            .title
            .clone()
            .unwrap_or_else(|| "Unknown title".to_string())
    }

    pub(crate) fn album(&self) -> String {
        self.0
            .album
            .clone()
            .unwrap_or_else(|| "Unknown album".to_string())
    }

    pub(crate) fn artist(&self) -> String {
        self.0
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown artist".to_string())
    }

    pub(crate) fn id(&self) -> Option<i32> {
        self.0.id
    }
}

impl From<server_lib::models::Song> for Song {
    fn from(value: server_lib::models::Song) -> Self {
        Self(value)
    }
}

impl TryFrom<PathBuf> for Song {
    type Error = ApiError;

    fn try_from(file_path: PathBuf) -> Result<Self, Self::Error> {
        let tag = Tag::new().read_from_path(&file_path).map_err(|error| {
            warn!("Can't read tag of {} : {error}", file_path.display());
            ApiError(format!(
                "Can't read tag of {} : {error}",
                file_path.display()
            ))
        })?;

        Ok(Self(server_lib::models::Song {
            id: None,
            title: tag.title().map(|v| v.to_string()),
            album: tag.album().map(|v| v.title.to_string()),
            track: tag.track().0.map(|v| v as i32),
            artist: tag.artist().map(|v| v.to_string()),
            duration: tag.duration().map(|v| v as i32),
        }))
    }
}
