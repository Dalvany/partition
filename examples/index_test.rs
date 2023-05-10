use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};
use tantivy_analysis_contrib::commons::LengthTokenFilter;
use tantivy_analysis_contrib::icu::{Direction, ICUTransformTokenFilter};

fn main() -> tantivy::Result<()> {
    let args = std::env::args().skip(1);

    let transform = ICUTransformTokenFilter {
        compound_id: "Any-Latin; NFD; [:Nonspacing Mark:] Remove; Lower;  NFC".to_string(),
        rules: None,
        direction: Direction::Forward,
    };
    let icu_analyzer = TextAnalyzer::from(SimpleTokenizer)
        .filter(LengthTokenFilter::new(Some(3), None))
        .filter(transform);

    for arg in args {
        println!("{arg}");

        let mut token_stream = icu_analyzer.token_stream(&arg);
        while token_stream.advance() {
            println!("{:?}", token_stream.token());
        }
    }

    Ok(())
}
