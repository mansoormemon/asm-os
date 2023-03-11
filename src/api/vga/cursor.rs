/////////////
/// Style ///
/////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    Underscore = 0x0,
    Block = 0x1,
}

impl Style {
    /// Creates a new object from the given index.
    pub fn from_index(index: usize) -> Self {
        match index {
            0x0 => Style::Underscore,
            0x1 => Style::Block,
            _ => Style::Underscore,
        }
    }

    /// Returns the scanline bounds.
    pub fn get_scanline_bounds(&self) -> (u8, u8) {
        match self {
            Style::Underscore => (0xD, 0xE),
            Style::Block => (0x1, 0xE),
        }
    }
}
