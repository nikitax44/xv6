use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::{PGSHIFT, PGSIZE};
use crate::vm::mode::Mode;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use core::ops::IndexMut;

#[repr(transparent)]
pub struct Pagetable {
    pub(super) inner: ThinBox<IPagetable>,
}

impl Pagetable {
    /// # Errors
    /// malloc failed
    pub fn new() -> Result<Self, PTError> {
        Ok(Self {
            inner: IPagetable::new().ok_or(PTError::AllocFail)?,
        })
    }

    const PTELVLS: usize = 3;
    const PTE_LVL_SHIFT: usize = 9;
    pub(crate) const PT_ENTRIES: usize = 1 << Self::PTE_LVL_SHIFT;
    const PT_ENTRY_MASK: usize = (1 << Self::PTE_LVL_SHIFT) - 1;
    pub(crate) const MAX_VA: usize = 1 << (Self::PTELVLS * Self::PTE_LVL_SHIFT + PGSHIFT - 1);

    fn get_idx(level: usize, virtual_address: usize) -> usize {
        (virtual_address >> (PGSHIFT + Self::PTE_LVL_SHIFT * level)) & Self::PT_ENTRY_MASK
    }

    fn verify_va(addr: usize) -> Result<(), PTError> {
        if addr >= Self::MAX_VA {
            return Err(PTError::InvalidVirtualAddress);
        };
        if addr % PGSIZE != 0 {
            return Err(PTError::InvalidVirtualAddress);
        }
        Ok(())
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk(&self, virtual_address: usize) -> Result<PtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        let pt2 = &self.inner;
        // SAFETY: statically known to contain either Null or ptr to IPagetable
        let pt1: &IPagetable =
            unsafe { pt2[Self::get_idx(2, virtual_address)].as_pt() }.ok_or(PTError::NotMapped)?;

        // SAFETY: statically known to contain either Null or ptr to IPagetable
        let pt0: &IPagetable =
            unsafe { pt1[Self::get_idx(1, virtual_address)].as_pt() }.ok_or(PTError::NotMapped)?;

        Ok(pt0[Self::get_idx(0, virtual_address)])
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk_mut(&mut self, virtual_address: usize) -> Result<&mut PtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        // SAFETY: statically known to contain either Null or ptr to IPagetable
        let pt_entry2: &mut PtEntry = &mut self.inner[Self::get_idx(2, virtual_address)];
        if !pt_entry2.is_set() {
            pt_entry2.set_pt(IPagetable::new().ok_or(PTError::AllocFail)?);
        }

        let pt1: &mut IPagetable = unsafe { pt_entry2.as_pt_mut() }.unwrap();

        let pt_entry1: &mut PtEntry = &mut pt1[Self::get_idx(1, virtual_address)];
        if !pt_entry1.is_set() {
            pt_entry1.set_pt(IPagetable::new().ok_or(PTError::AllocFail)?);
        }

        let pt0: &mut IPagetable = unsafe { pt_entry1.as_pt_mut() }.unwrap();

        // SAFETY: we know that inner will outlive 'self
        Ok(pt0.index_mut(Self::get_idx(0, virtual_address)))
    }

    /// # Errors
    /// see `MMapError`
    #[track_caller]
    pub fn map_page(
        &mut self,
        virtual_address: usize,
        physycal_address: usize,
        perm: Mode,
    ) -> Result<(), PTError> {
        let pte = self.walk_mut(virtual_address)?;
        if pte.flags().contains(Mode::PTE_V) {
            return Err(PTError::Remap);
        };
        pte.set(physycal_address, perm)
    }

    /// # Errors
    /// this address it not mapped
    pub fn unmap_page(&mut self, virtual_address: usize) -> Result<(), PTError> {
        if self.walk(virtual_address).is_err() {
            return Err(PTError::NotMapped);
        }
        self.walk_mut(virtual_address)?.unset();
        Ok(())
    }

    /// # Errors
    /// `size` is not page-aligned or zero
    /// see `map_page`
    pub fn map_pages(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        size: usize,
        mode: Mode,
    ) -> Result<(), PTError> {
        if size % PGSIZE != 0 || size == 0 {
            return Err(PTError::InvalidSize);
        }
        (0usize..size).step_by(PGSIZE).try_for_each(|offset| {
            self.map_page(virtual_address + offset, physical_address + offset, mode)
        })
    }
}
