use crate::kalloc::pages::{KMEMError, KMEM};
use crate::kalloc::thin_box::ThinBox;
use crate::vm::pagetable::Pagetable;
use crate::vm::pte::PtEntry;
use core::fmt::{Debug, Formatter};
use core::ops::{Index, IndexMut};
use zerocopy::{FromZeros, KnownLayout, TryFromBytes};

#[repr(transparent)]
#[derive(FromZeros, KnownLayout)]
pub struct IPagetable {
    pub entries: [PtEntry; Pagetable::PT_ENTRIES],
}

impl IPagetable {
    #[track_caller]
    pub fn alloc() -> Result<ThinBox<Self>, KMEMError> {
        KMEM.lock()
            .alloc("IPagetable::alloc()")
            .map(|page| page.zeroed().into_box().leak_ref())
            .map(|page| &mut page.0[..])
            .map(Self::try_mut_from_bytes)
            .map(Result::unwrap)
            .map(ThinBox::from)
    }
}

impl Index<usize> for IPagetable {
    type Output = PtEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for IPagetable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Debug for IPagetable {
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
