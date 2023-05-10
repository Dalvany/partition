use crate::library::Song;
use anyhow::Result;
use log::{debug, warn};
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::schema::{IndexRecordOption, NumericOptions, Schema, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};
use tantivy::{Index, IndexWriter, Opstamp};
use tantivy_analysis_contrib::commons::LengthTokenFilter;
use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};

/// `/!\` DON'T FORGET TO MODIFY `init_index` WHEN ADDING MORE VARIANT.
// TODO Rework this (multiple analysis, ...etc)
pub(crate) enum PartitionFields {
    Id,
    Title,
    Artist,
    Album,
}

impl PartitionFields {
    pub(crate) fn field_name(&self) -> &str {
        match self {
            Self::Id => "id",
            Self::Title => "title",
            Self::Artist => "artist",
            Self::Album => "album",
        }
    }

    fn index_analysis_name(&self) -> &str {
        match self {
            Self::Id => "unused",
            Self::Title => "index_analysis_title",
            Self::Artist => "index_analysis_artist",
            Self::Album => "index_analysis_album",
        }
    }

    fn text_options(&self) -> TextOptions {
        let field_indexing = TextFieldIndexing::default()
            .set_tokenizer(self.index_analysis_name())
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);

        TextOptions::default()
            .set_indexing_options(field_indexing)
            .set_stored()
    }
}

pub(crate) struct TantivyIndex {
    index: Index,
    schema: Schema,
    writer: RwLock<IndexWriter>,
}

impl TantivyIndex {
    pub(crate) fn index(&self, song: Song) -> tantivy::Result<Opstamp> {
        let document = song.into_document(&self.schema);
        let mut writer = self.writer.write().unwrap();
        writer.add_document(document)?;
        writer.commit()
    }

    pub(crate) fn search(&self, query: String, offset: usize, limit: usize) -> tantivy::Result<()> {
        let title = self
            .schema
            .get_field(PartitionFields::Title.field_name())
            .unwrap();
        let album = self
            .schema
            .get_field(PartitionFields::Album.field_name())
            .unwrap();
        let artist = self
            .schema
            .get_field(PartitionFields::Artist.field_name())
            .unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![title, album, artist]);
        let query = query_parser.parse_query(&query)?;

        let top_doc = TopDocs::with_limit(limit).and_offset(offset);

        let searcher = self.index.reader()?.searcher();
        let result = searcher.search(&query, &top_doc)?;

        for (score, doc_address) in result {
            let retrieved_doc = searcher.doc(doc_address)?;
            debug!("{score} : {}", self.schema.to_json(&retrieved_doc));
        }

        Ok(())
    }
}

pub(crate) fn init_index<P: AsRef<Path>>(path: P) -> Result<TantivyIndex> {
    if let Err(error) = fs::create_dir_all(&path) {
        warn!("{error:?}");
    }

    let mut builder = Schema::builder();
    builder.add_text_field(
        PartitionFields::Title.field_name(),
        PartitionFields::Title.text_options(),
    );
    builder.add_text_field(
        PartitionFields::Album.field_name(),
        PartitionFields::Album.text_options(),
    );
    builder.add_text_field(
        PartitionFields::Artist.field_name(),
        PartitionFields::Artist.text_options(),
    );
    builder.add_i64_field(
        PartitionFields::Id.field_name(),
        NumericOptions::default().set_stored(),
    );
    let schema = builder.build();

    let mmap_directory = MmapDirectory::open(path)?;
    let wrapper = ManagedDirectory::wrap(Box::new(mmap_directory))?;
    let index = Index::open_or_create(wrapper, schema)?;

    let transform = ICUTransformTokenFilter {
        compound_id: "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
        rules: None,
        direction: Direction::Forward,
    };
    let icu_analyzer = TextAnalyzer::from(SimpleTokenizer)
        .filter(LengthTokenFilter::new(Some(3), None))
        .filter(transform);
    index.tokenizers().register(
        PartitionFields::Title.index_analysis_name(),
        icu_analyzer.clone(),
    );
    index.tokenizers().register(
        PartitionFields::Album.index_analysis_name(),
        icu_analyzer.clone(),
    );
    index
        .tokenizers()
        .register(PartitionFields::Artist.index_analysis_name(), icu_analyzer);

    let writer = RwLock::new(index.writer(5_000_000)?);
    let schema = index.schema();

    Ok(TantivyIndex {
        index,
        schema,
        writer,
    })
}
