use crate::memlayout::{PGSHIFT, PGSIZE};
use crate::vm::mode::Mode;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use core::ops::IndexMut;

#[repr(transparent)]
pub struct Pagetable<'inner> {
    pub(super) inner: &'inner mut IPagetable,
}

impl Pagetable<'static> {
    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn alloc() -> Result<Self, PTError> {
        Ok(Self::new(
            IPagetable::alloc().ok_or(PTError::AllocFail)?.leak_ref(),
        ))
    }
}

impl<'inner> Pagetable<'inner> {
    /// # Errors
    /// malloc failed
    pub fn new(inner: &'inner mut IPagetable) -> Self {
        Self { inner }
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
        let pt1: &IPagetable =
        // SAFETY: statically known to contain either Null or ptr to IPagetable
            unsafe { pt2[Self::get_idx(2, virtual_address)].as_pt() }.ok_or(PTError::NotMapped)?;

        let pt0: &IPagetable =
        // SAFETY: statically known to contain either Null or ptr to IPagetable
            unsafe { pt1[Self::get_idx(1, virtual_address)].as_pt() }.ok_or(PTError::NotMapped)?;

        Ok(pt0[Self::get_idx(0, virtual_address)])
    }

    /// # Errors
    /// see `MMapError`
    /// # Panics
    /// never
    pub fn walk_mut(&mut self, virtual_address: usize) -> Result<&mut PtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        // SAFETY: statically known to contain either Null or ptr to IPagetable
        let pt_entry2: &mut PtEntry = &mut self.inner[Self::get_idx(2, virtual_address)];
        if !pt_entry2.is_set() {
            pt_entry2.set_pt(IPagetable::alloc().ok_or(PTError::AllocFail)?);
        }

        // SAFETY: level2 ptentries may contain only `IPagetable`s
        let pt1: &mut IPagetable = unsafe { pt_entry2.as_pt_mut() }.unwrap();

        let pt_entry1: &mut PtEntry = &mut pt1[Self::get_idx(1, virtual_address)];
        if !pt_entry1.is_set() {
            pt_entry1.set_pt(IPagetable::alloc().ok_or(PTError::AllocFail)?);
        }

        // SAFETY: level1 ptentries may contain only `IPagetable`s
        let pt0: &mut IPagetable = unsafe { pt_entry1.as_pt_mut() }.unwrap();

        // SAFETY: we know that inner will outlive 'self
        Ok(pt0.index_mut(Self::get_idx(0, virtual_address)))
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

mod ffi {
    use crate::errno::ErrNo;
    use crate::errno::ErrNo::{ENOMEM, SUCCESS};
    use crate::vm::pagetable::Pagetable;
    use crate::vm::PTError;

    //int mappages(pagetable_t pagetable, u64 va, u64 size, u64 pa, int perm)
    #[no_mangle]
    extern "C" fn mappages(
        mut pt: Pagetable,
        virtual_address: usize,
        size: usize,
        physical_address: usize,
        perm: usize,
    ) -> ErrNo {
        let perm = perm.try_into().expect("invalid access mode");
        if let Err(err) = pt.map_pages(virtual_address, physical_address, size, perm) {
            return match err {
                PTError::AllocFail => ENOMEM,

                _ => panic!("ffi::mappages: {:?}", err),
            };
        }
        SUCCESS
    }
}
