use bytepiece::tokenizer::{parse_pieces_from_slice, Tokenize, Tokenizer};

fn main() {
    let buf = std::fs::read("../bytepiece_80k.model").unwrap();
    let pieces = parse_pieces_from_slice(&buf);

    let tokenizer = Tokenizer::from_pieces(&pieces);

    let text = "hi what is your name";
    dbg!(text);

    let tokens = tokenizer.tokenize(text, -1.0);
    dbg!(&tokens);

    let ids = tokenizer.encode(text, false, false, -1.0);

    dbg!(&ids);

    let sentence = tokenizer.decode(&ids);

    dbg!(&sentence);
}
