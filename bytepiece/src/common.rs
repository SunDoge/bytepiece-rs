pub const PAD: &[u8] = b"<pad>";
pub const BOS: &[u8] = b"<bos>";
pub const EOS: &[u8] = b"<eos>";

#[derive(Debug)]
#[repr(usize)]
pub enum SpatialToken {
    Pad = 0,
    Bos,
    Eos,
}
