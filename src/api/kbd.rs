use core::str::FromStr;

use crate::drv::input;

///////////////
/// Default
///////////////
pub struct Default;

impl Default {
    pub const LAYOUT: Layout = Layout::QWERTY;
}

//////////////
/// Layout
//////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Layout {
    AZERTY = 0x0,
    Dvorak = 0x1,
    QWERTY = 0x2,
}

impl Layout {
    /// Creates a new object from enum index.
    pub fn from_index(idx: u8) -> Result<Self, ()> {
        match idx {
            0x0 => Ok(Self::AZERTY),
            0x1 => Ok(Self::Dvorak),
            0x2 => Ok(Self::QWERTY),
            _ => Err(()),
        }
    }

    /// Returns the object as an enum index.
    pub fn as_u8(&self) -> u8 { (*self) as u8 }

    /// Returns the object as a primitive string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AZERTY => "azerty",
            Self::Dvorak => "dvorak",
            Self::QWERTY => "qwerty",
        }
    }
}

impl FromStr for Layout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "azerty" => Ok(Self::AZERTY),
            "dvorak" => Ok(Self::Dvorak),
            "qwerty" => Ok(Self::QWERTY),
            _ => Err(())
        }
    }
}

/// Returns the layout.
pub fn get_layout() -> Layout { input::kbd::get_layout() }

/// Sets the layout.
pub fn set_layout(lyt: Layout) { input::kbd::set_layout(lyt); }

/// Resets the layout.
pub fn reset_layout() { input::kbd::reset_layout(); }
