pub const PAD: &'static str = "<pad>";
pub const BOS: &'static str = "<bos>";
pub const EOS: &'static str = "<eos>";

#[derive(Debug)]
#[repr(usize)]
pub enum SpatialToken {
    Pad = 0,
    Bos,
    Eos,
}
