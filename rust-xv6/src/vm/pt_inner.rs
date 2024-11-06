use crate::kalloc::pages::KMEMError;
use crate::kalloc::thin_box::ThinBox;
use crate::vm::pagetable::Pagetable;
use crate::vm::pte::PtEntry;
use core::fmt::{Debug, Formatter};
use core::ops::{Index, IndexMut};

#[repr(transparent)]
pub(super) struct IPagetable {
    pub entries: [PtEntry; Pagetable::PT_ENTRIES],
}

impl IPagetable {
    #[track_caller]
    pub fn alloc() -> Result<ThinBox<Self>, KMEMError> {
        ThinBox::alloc()
            .map(ThinBox::zero)
            // SAFETY: 0 is valid state for IPagetable
            .map(|ipt| unsafe { ipt.assume_init() })
            .map_err(KMEMError::AllocFail)
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
