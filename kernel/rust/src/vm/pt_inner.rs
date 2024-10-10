use crate::kalloc::pages::KMEM;
use crate::kalloc::thin_box::ThinBox;
use crate::vm::pagetable::Pagetable;
use crate::vm::pte::PtEntry;
use core::ops::{Index, IndexMut};

#[repr(transparent)]
pub struct IPagetable {
    pub entries: [PtEntry; Pagetable::PT_ENTRIES],
}

impl IPagetable {
    #[track_caller]
    pub fn alloc() -> Option<ThinBox<Self>> {
        KMEM.lock()
            .alloc("IPagetable::alloc()")
            .map(|page| page.zeroed().leak_uninit::<Self>())
            // SAFETY: page was zeroed
            .map(|uninit| unsafe { uninit.assume_init_mut() }.into())
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
