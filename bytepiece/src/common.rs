pub const PAD: &str = "<pad>";
pub const BOS: &str = "<bos>";
pub const EOS: &str = "<eos>";

#[derive(Debug)]
#[repr(usize)]
pub enum SpatialToken {
    Pad = 0,
    Bos,
    Eos,
}
