use anyhow::Result;
use base64::engine::general_purpose;
use server_lib::Client;
use server_lib::{ApiNoContext, ContextWrapperExt};
use std::io::Write;
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

fn main() -> Result<()> {
    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    let rt = tokio::runtime::Runtime::new()?;

    let client: Box<dyn ApiNoContext<ClientContext>> = {
        // Using HTTP
        let client = Box::new(
            Client::try_new_http("http://127.0.0.1:8000").expect("Failed to create HTTP client"),
        );
        Box::new(client.with_context(context))
    };

    let mut enc = base64::write::EncoderWriter::new(Vec::new(), &general_purpose::STANDARD);

    //let file = std::fs::read("../../Musique/Agnès Bihl/La Terre Blonde/01 L'enceinte vierge.mp3")?;
    let file = std::fs::read("./tests-resources/RossBugden-Notturno.mp3")?;
    enc.write_all(&file)?;
    let encoded = enc.finish()?;
    let encoded = String::from_utf8(encoded)?;
    let result = rt.block_on(client.songs_post("RossBugden-Notturno.mp3".to_owned(), encoded));
    println!(
        "{:?} (X-Span-ID: {:?})",
        result,
        (client.context() as &dyn Has<XSpanIdString>).get().clone()
    );

    Ok(())
}
