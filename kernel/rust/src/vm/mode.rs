use bitflags::bitflags;
bitflags! {
    #[derive(Debug, Eq, PartialEq,Copy, Clone)]
    pub struct Mode: usize {
        const PTE_V = 0b00001; // valid
        const PTE_R = 0b00010;
        const PTE_RW = 0b00110; // includes R
        const PTE_X = 0b01000;
        const PTE_U = 0b10000; // user can access

        const MASK=0x3ff;
    }
}
impl Mode {
    pub(crate) const SHIFT: u32 = Self::MASK.bits().trailing_ones();
}
