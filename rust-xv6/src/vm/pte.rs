use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use crate::vm::mode::Mode;
use crate::vm::pt_inner::{KiloPT, MegaPT};
use crate::vm::PTError;
use core::fmt::{Debug, Formatter};
use zerocopy::FromZeros;

macro_rules! pte_impl {
    ($pte:ident) => {
        #[derive(Copy, Clone)]
        #[repr(transparent)]
        #[derive(FromZeros)]
        pub struct $pte(usize);

        impl $pte {
            #[must_use]
            pub fn is_set(&self) -> bool {
                self.get().is_ok()
            }

            /// # Panics
            /// if got invalid PTE
            /// # Errors
            /// page is not mapped
            pub fn get(&self) -> Result<(usize, Mode), PTError> {
                if self.0 & Mode::VALID == 0 {
                    return Err(PTError::NotMapped);
                }
                Ok((
                    (self.0 >> Mode::SHIFT) * PGSIZE,
                    (self.0 & Mode::ACCESS_MASK)
                        .try_into()
                        .expect("invalid PTE"),
                ))
            }

            pub const fn unset(&mut self) {
                self.0 = 0;
            }

            /// # Errors
            /// addr and mode must be valid. mode is not Table
            pub fn set(&mut self, addr: usize, mode: Mode) -> Result<(), PTError> {
                if mode == Mode::Table {
                    return Err(PTError::UnexpectedTable);
                }
                self.set_raw(addr, mode)
            }

            const fn set_raw(&mut self, addr: usize, mode: Mode) -> Result<(), PTError> {
                if addr % PGSIZE != 0 {
                    return Err(PTError::UnalignedPhysicalAddress(addr));
                }
                self.0 = ((addr / PGSIZE) << Mode::SHIFT) | (mode as usize) | Mode::VALID;
                Ok(())
            }
        }
    };
    ($pte:ident, $next_pt:ty) => {
        pte_impl!($pte);
        impl $pte {
            pub(super) fn set_pt(&mut self, pt: ThinBox<$next_pt>) {
                self.set_raw(pt.leak().as_ptr() as usize, Mode::Table)
                    .unwrap();
            }

            /// # Errors
            /// page is not mapped
            /// mode is not `Mode::Table`
            pub(super) fn as_pt(&self) -> Result<&$next_pt, PTError> {
                let (addr, mode) = self.get()?;
                if mode != Mode::Table {
                    return Err(PTError::NotTable);
                }
                // SAFETY: contains reference to Pagetable by invariant
                Ok(unsafe { &*(addr as *const $next_pt) })
            }

            /// # Errors
            /// page is not mapped
            /// mode is not `Mode::Table`
            pub(super) fn as_pt_mut(&mut self) -> Result<&mut $next_pt, PTError> {
                let (addr, mode) = self.get()?;
                if mode != Mode::Table {
                    return Err(PTError::NotTable);
                }
                // SAFETY: contains reference to Pagetable by invariant
                Ok(unsafe { &mut *(addr as *mut $next_pt) })
            }
        }

        impl Debug for $pte {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                if let Ok(inner) = self.as_pt() {
                    return write!(f, "{}({:#?})", stringify!($pte), inner);
                }

                if let Ok((addr, mode)) = self.get() {
                    write!(f, "{}({addr:#x}, {mode:?})", stringify!($pte))
                } else {
                    write!(f, "{}(None)", stringify!($pte))
                }
            }
        }
    };
}

pte_impl!(GigaPtEntry, MegaPT);
pte_impl!(MegaPtEntry, KiloPT);
pte_impl!(KiloPtEntry);

impl Debug for KiloPtEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Ok((addr, mode)) = self.get() {
            write!(f, "KiloPtEntry({addr:#x}, {mode:?})")
        } else {
            write!(f, "KiloPtEntry(None)")
        }
    }
}
