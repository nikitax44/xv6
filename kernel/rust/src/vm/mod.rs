use crate::kalloc::pages::KMEM;
use crate::util::{assert_page_aligned, PGSHIFT, PGSIZE};
use core::ptr::NonNull;
const PTE_V: usize = 1 << 0; // valid
const PTE_R: usize = 1 << 1;
const PTE_W: usize = 1 << 2;
const PTE_X: usize = 1 << 3;
const PTE_U: usize = 1 << 4; // user can access
                             // PTE_FLAGS(pte) ((pte) & 0x3FF)
const PTELVLS: usize = 3;
const PTESHIFT: usize = 9;
const PTESIZE: usize = 1 << PTESHIFT;
const PTEMASK: usize = (1 << PTESHIFT) - 1;
const MAXVA: usize = 1 << (PTELVLS * PTESHIFT + PGSHIFT - 1);

union PtInner {
    addr: Option<NonNull<Pagetable>>,
    pte: PtEntry,
}

#[derive(Copy, Clone)]
pub struct PtEntry(usize);

pub enum MMapError {
    Remap,
    ZeroSize,
    MallocFail,
    NotMapped,
    InvalidAddress,
}

pub struct Pagetable {
    data: [PtInner; PTESIZE],
}

impl Pagetable {
    fn get_idx(level: usize, virtual_address: usize) -> usize {
        (virtual_address >> (PGSHIFT + PTESHIFT * level)) & PTEMASK
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk(&self, virtual_address: usize) -> Result<PtEntry, MMapError> {
        if virtual_address >= MAXVA {
            return Err(MMapError::InvalidAddress);
        };

        let pt2: &Self = self;

        // SAFETY:
        // statically known
        let pt1_addr = unsafe { pt2.data[Self::get_idx(2, virtual_address)].addr };
        // SAFETY:
        // we own this ptr, it is safe
        let pt1: &Self = unsafe { pt1_addr.ok_or(MMapError::NotMapped)?.as_ref() };

        // SAFETY:
        // statically known
        let pt0_addr = unsafe { pt1.data[Self::get_idx(1, virtual_address)].addr };
        // SAFETY:
        // we own this ptr, it is safe
        let pt0: &Self = unsafe { pt0_addr.ok_or(MMapError::NotMapped)?.as_ref() };

        // SAFETY:
        // statically known
        let final_pte = unsafe { pt0.data[Self::get_idx(0, virtual_address)].pte };
        if final_pte.0 == 0 {
            return Err(MMapError::NotMapped);
        }
        Ok(final_pte)
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk_mut(&mut self, virtual_address: usize) -> Result<&mut PtEntry, MMapError> {
        if virtual_address >= MAXVA {
            return Err(MMapError::InvalidAddress);
        };

        let pt2: &mut Self = self;

        // SAFETY:
        // statically known
        let pt_entry2 = unsafe { &mut pt2.data[Self::get_idx(2, virtual_address)].addr };
        if pt_entry2.is_none() {
            *pt_entry2 = KMEM.lock().alloc().map(|page| page.zeroed().leak().cast());
        }

        // SAFETY:
        // we own this ptr, it is safe
        let pt1: &mut Self = unsafe { pt_entry2.ok_or(MMapError::MallocFail)?.as_mut() };

        // SAFETY:
        // statically known
        let pt_entry1 = unsafe { &mut pt1.data[Self::get_idx(1, virtual_address)].addr };
        if pt_entry1.is_none() {
            *pt_entry1 = KMEM.lock().alloc().map(|page| page.zeroed().leak().cast());
        }

        // SAFETY:
        // we own this ptr, it is safe
        let pt0: &mut Self = unsafe { pt_entry1.ok_or(MMapError::MallocFail)?.as_mut() };

        // SAFETY:
        // statically known
        Ok(unsafe { &mut pt0.data[Self::get_idx(0, virtual_address)].pte })
    }

    /// # Errors
    /// see `MMapError`
    pub fn mappages(
        &mut self,
        virtual_address: usize,
        size: usize,
        mut physycal_address: usize,
        perm: usize,
    ) -> Result<(), MMapError> {
        assert_page_aligned(virtual_address);
        assert_page_aligned(size);
        if size == 0 {
            return Err(MMapError::ZeroSize);
        }

        let mut a = virtual_address;
        let last = virtual_address + size - PGSIZE;
        loop {
            let pte = self.walk_mut(a)?;
            if pte.0 & PTE_V != 0 {
                return Err(MMapError::Remap);
            };
            *pte = PtEntry(((physycal_address >> 12) << 10) | PTE_V | perm);
            if a == last {
                break;
            }
            a += PGSIZE;
            physycal_address += PGSIZE;
        }

        Ok(())
    }
}
