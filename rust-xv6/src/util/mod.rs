pub use efs::arch::u64_to_usize;

pub mod lazy;
pub mod string;
pub mod time;

pub trait Rounding: Copy {
    #[must_use]
    fn round_down2(self, n: Self) -> Self;
    #[must_use]
    fn round_up2(self, n: Self) -> Self;
    #[must_use]
    fn modulo2(self, n: Self) -> Self;
}

impl Rounding for usize {
    fn round_down2(self, n: Self) -> Self {
        assert!(n.is_power_of_two(), "modulus must be a power of two");
        self & !(n - 1)
    }

    fn round_up2(self, n: Self) -> Self {
        assert!(n.is_power_of_two(), "modulus must be a power of two");
        (self + n - 1) & !(n - 1)
    }

    fn modulo2(self, n: Self) -> Self {
        self - self.round_down2(n)
    }
}
