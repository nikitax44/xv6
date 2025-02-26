use crate::vm::pagetable::Pagetable;
use crate::vm::pte::{GigaPtEntry, KiloPtEntry, MegaPtEntry};
use core::fmt::{Debug, Formatter};
use core::ops::{Index, IndexMut};
use zerocopy::FromZeros;

macro_rules! pt_impl {
    ($struc:ident, $pte:ty) => {
        #[repr(C, align(4096))]
        #[derive(FromZeros)]
        pub(super) struct $struc {
            pub entries: [$pte; Pagetable::PT_ENTRIES],
        }

        impl Index<usize> for $struc {
            type Output = $pte;

            fn index(&self, index: usize) -> &Self::Output {
                &self.entries[index]
            }
        }

        impl IndexMut<usize> for $struc {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.entries[index]
            }
        }

        impl Debug for $struc {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.debug_map()
                    .entries(
                        self.entries
                            .iter()
                            .enumerate()
                            .filter(|(_idx, entry)| entry.is_set()),
                    )
                    .finish()
            }
        }
    };
}

pt_impl!(GigaPT, GigaPtEntry);
pt_impl!(MegaPT, MegaPtEntry);
pt_impl!(KiloPT, KiloPtEntry);
