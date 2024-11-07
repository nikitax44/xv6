use crate::kalloc::region::Region;
use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::{PGSHIFT, PGSIZE};
use crate::vm::mode::Mode;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use core::ops::IndexMut;

#[derive(Debug)]
pub struct Pagetable<'inner> {
    inner: Inner<'inner>,
}

#[derive(Debug)]
enum Inner<'inner> {
    Owned(ThinBox<IPagetable>),
    Ref(&'inner IPagetable),
    Mut(&'inner mut IPagetable),
}

// TODO: add support for megapages and gigapages
impl<'inner> Pagetable<'inner> {
    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn new() -> Result<Self, PTError> {
        Ok(Self {
            inner: Inner::Owned(IPagetable::alloc().map_err(PTError::AllocFail)?),
        })
    }

    const PTELVLS: usize = 3;
    const PTE_LVL_SHIFT: usize = 9;
    pub(crate) const PT_ENTRIES: usize = 1 << Self::PTE_LVL_SHIFT;
    const PT_ENTRY_MASK: usize = (1 << Self::PTE_LVL_SHIFT) - 1;
    pub(crate) const MAX_VA: usize = 1 << (Self::PTELVLS * Self::PTE_LVL_SHIFT + PGSHIFT - 1);

    const fn get_idx(level: usize, virtual_address: usize) -> usize {
        (virtual_address >> (PGSHIFT + Self::PTE_LVL_SHIFT * level)) & Self::PT_ENTRY_MASK
    }

    const fn verify_va(addr: usize) -> Result<(), PTError> {
        if addr >= Self::MAX_VA {
            return Err(PTError::InvalidVirtualAddress(addr));
        };
        if addr % PGSIZE != 0 {
            return Err(PTError::UnalignedVirtualAddress(addr));
        }
        Ok(())
    }

    pub(super) fn inner_ref(&self) -> &IPagetable {
        match &self.inner {
            Inner::Owned(inner) => inner,
            Inner::Ref(inner) => inner,
            Inner::Mut(inner) => inner,
        }
    }

    pub(super) fn inner_mut(&mut self) -> Option<&mut IPagetable> {
        match &mut self.inner {
            Inner::Owned(inner) => Some(inner),
            Inner::Ref(_inner) => None,
            Inner::Mut(inner) => Some(inner),
        }
    }

    pub(super) const fn from_ref(inner: &'inner IPagetable) -> Self {
        Self {
            inner: Inner::Ref(inner),
        }
    }

    pub(super) fn from_mut(inner: &'inner mut IPagetable) -> Self {
        Self {
            inner: Inner::Mut(inner),
        }
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk(&self, virtual_address: usize) -> Result<PtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        let pt2 = self.inner_ref();
        let pt1: &IPagetable = pt2[Self::get_idx(2, virtual_address)]
            .as_pt()
            .ok_or(PTError::NotMapped)?;

        let pt0: &IPagetable = pt1[Self::get_idx(1, virtual_address)]
            .as_pt()
            .ok_or(PTError::NotMapped)?;

        Ok(pt0[Self::get_idx(0, virtual_address)])
    }

    /// # Errors
    /// see `MMapError`
    /// # Panics
    /// never
    pub fn walk_mut(&mut self, virtual_address: usize) -> Result<&mut PtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        let inner = self.inner_mut().ok_or(PTError::ROPagetable)?;
        let pt_entry2: &mut PtEntry = &mut inner[Self::get_idx(2, virtual_address)];
        if !pt_entry2.is_set() {
            pt_entry2.set_pt(IPagetable::alloc().map_err(PTError::AllocFail)?);
        }

        let pt1: &mut IPagetable = pt_entry2.as_pt_mut().unwrap();

        let pt_entry1: &mut PtEntry = &mut pt1[Self::get_idx(1, virtual_address)];
        if !pt_entry1.is_set() {
            pt_entry1.set_pt(IPagetable::alloc().map_err(PTError::AllocFail)?);
        }

        let pt0: &mut IPagetable = pt_entry1.as_pt_mut().unwrap();

        Ok(pt0.index_mut(Self::get_idx(0, virtual_address)))
    }

    /// # Errors
    /// page is not mapped
    /// self's invariants aren't held
    pub fn translate(&self, va: usize) -> Result<usize, PTError> {
        let page = va / PGSIZE * PGSIZE;
        self.walk(page)
            .and_then(|pte| pte.addr())
            .map(|pa| pa + (va - page))
    }

    /// # Errors
    /// see `MMapError`
    /// # Panics
    /// if contains bugs
    #[track_caller]
    pub fn map_page(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        perm: Mode,
    ) -> Result<(), PTError> {
        let pte = self.walk_mut(virtual_address)?;
        if pte.is_set() {
            return Err(PTError::Remap);
        };
        pte.set(physical_address, perm)?;

        debug_assert_eq!(
            self.walk(virtual_address)
                .expect("failed to properly map")
                .get(),
            Some((physical_address, perm)),
            "vmmap: read different value from one written"
        );
        Ok(())
    }

    /// # Errors
    /// this address it not mapped
    pub fn unmap_page(&mut self, virtual_address: usize) -> Result<(), PTError> {
        // do not create pages to unmapped page
        self.walk(virtual_address)?;

        self.walk_mut(virtual_address)?.unset();
        Ok(())
    }

    /// # Errors
    /// `size` is not page-aligned
    /// see `map_page`
    pub fn map_pages(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        size: usize,
        mode: Mode,
    ) -> Result<(), PTError> {
        if size % PGSIZE != 0 {
            return Err(PTError::UnalignedSize(size));
        }
        (0usize..size).step_by(PGSIZE).try_for_each(|offset| {
            self.map_page(virtual_address + offset, physical_address + offset, mode)
        })
    }

    /// # Errors
    /// region is not page-aligned or is empty
    /// see `map_page`
    pub fn map_reg(&mut self, reg: Region, mode: Mode) -> Result<(), PTError> {
        self.map_pages(reg.start(), reg.start(), reg.size(), mode)
    }
}
