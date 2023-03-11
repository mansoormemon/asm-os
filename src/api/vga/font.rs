use alloc::vec::Vec;

////////////
/// Font ///
////////////
pub struct Font {
    pub height: u8,
    pub size: u16,
    pub data: Vec<u8>,
}
