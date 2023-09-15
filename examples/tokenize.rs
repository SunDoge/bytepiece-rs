use bytepiece::tokenizer::{parse_pieces_from_slice, Pieces, Tokenizer};

fn main() {
    let buf = std::fs::read("bytepiece_80k.model").unwrap();
    let pieces = parse_pieces_from_slice(&buf);

    let tokenizer = Tokenizer::from_dict(&pieces);

    let text = "hi what is your name";
    dbg!(text);

    let ids = tokenizer.encode(text, false, false, -1.0);

    dbg!(&ids);

    let sentence = tokenizer.decode(&ids);

    dbg!(&sentence);
}
