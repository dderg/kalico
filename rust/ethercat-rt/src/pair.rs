//! The slot pair every differential feature (damper, trim, strain comp) is
//! configured against.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotPair {
    pub a: u8,
    pub b: u8,
}
