/// Unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Unit {
    KiB = 0x400,
    MiB = 0x100000,
    GiB = 0x40000000,
    TiB = 0x10000000000,
}
