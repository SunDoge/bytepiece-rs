use bytepiece::tokenizer::{parse_pieces_from_slice, Tokenize, Tokenizer};
use std::path::PathBuf;

fn get_model_path(path: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dbg!(&root.display());
    root.join(path)
}

fn main() -> bytepiece::Result<()> {
    let buf = std::fs::read(get_model_path("../models/bytepiece_80k.model")).unwrap();
    let pieces = parse_pieces_from_slice(&buf)?;
    let tokenizer = Tokenizer::from_pieces(&pieces)?;

    let text = "hi what is your name";
    dbg!(text);

    let tokens = tokenizer.tokenize(&text, -1.0);
    dbg!(&tokens);

    let ids = tokenizer.encode(text, false, false, -1.0);
    dbg!(&ids);

    let sentence = tokenizer.decode(&ids);
    dbg!(&sentence);

    Ok(())
}
