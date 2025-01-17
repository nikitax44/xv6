pub mod lazy;
pub mod mutex;
pub mod once;
pub mod rw_lock;
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

macro_rules! impl_rounding {
    () => {};
    ($tp:ty $(,$tps:ty)* $(,)?) => {
        impl Rounding for $tp {
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
        impl_rounding!($($tps),*);
    };
}

impl_rounding!(usize, u64);

/// # Panics
/// on 32-bit systems
#[must_use]
pub fn u64_to_usize(value: u64) -> usize {
    if cfg!(target_pointer_width = "64") {
        usize::try_from(value).unwrap()
    } else {
        panic!("u64_to_usize called on non-64-bit platform");
    }
}

pub fn copy_data(data: &mut [u8], buf: &[u8]) -> usize {
    let n = core::cmp::min(buf.len(), data.len());
    let buf = &buf[..n];
    let data = &mut data[..n];
    data.copy_from_slice(buf);
    n
}
