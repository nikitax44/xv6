use crate::kalloc::pages::KMEM;
use crate::kalloc::thin_box::ThinBox;
use crate::util::{assert_page_aligned, PGSHIFT, PGSIZE};
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

#[derive(Copy, Clone)]
pub struct PtEntry(usize);

pub enum MMapError {
    Remap,
    ZeroSize,
    MallocFail,
    NotMapped,
    InvalidAddress,
}

pub struct Pagetable0 {
    pte0: [PtEntry; PTESIZE],
}

pub struct Pagetable1 {
    pte1: [Option<ThinBox<Pagetable0>>; PTESIZE],
}

pub struct Pagetable2 {
    pte2: [Option<ThinBox<Pagetable1>>; PTESIZE],
}

impl Pagetable2 {
    fn get_idx(level: usize, virtual_address: usize) -> usize {
        (virtual_address >> (PGSHIFT + PTESHIFT * level)) & PTEMASK
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk(&self, virtual_address: usize) -> Result<PtEntry, MMapError> {
        if virtual_address >= MAXVA {
            return Err(MMapError::InvalidAddress);
        };

        let pt1: &Pagetable1 = self.pte2[Self::get_idx(2, virtual_address)]
            .as_ref()
            .ok_or(MMapError::NotMapped)?;

        let pt0: &Pagetable0 = pt1.pte1[Self::get_idx(1, virtual_address)]
            .as_ref()
            .ok_or(MMapError::NotMapped)?;

        Ok(pt0.pte0[Self::get_idx(0, virtual_address)])
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk_mut(&mut self, virtual_address: usize) -> Result<&mut PtEntry, MMapError> {
        if virtual_address >= MAXVA {
            return Err(MMapError::InvalidAddress);
        };

        let pt_entry2: &mut Option<ThinBox<Pagetable1>> =
            &mut self.pte2[Self::get_idx(2, virtual_address)];
        if pt_entry2.is_none() {
            *pt_entry2 = KMEM
                .lock()
                .alloc()
                .map(|page| page.zeroed().leak().cast::<Pagetable1>())
                // SAFETY:
                // page points to valid Pagetable1
                .map(|page| unsafe { ThinBox::new(page) });
        }

        let pt1: &mut Pagetable1 = pt_entry2.as_mut().ok_or(MMapError::MallocFail)?;

        let pt_entry1: &mut Option<ThinBox<Pagetable0>> =
            &mut pt1.pte1[Self::get_idx(1, virtual_address)];
        if pt_entry1.is_none() {
            *pt_entry1 = KMEM
                .lock()
                .alloc()
                .map(|page| page.zeroed().leak().cast::<Pagetable0>())
                // SAFETY:
                // page points to valid Pagetable0
                .map(|page| unsafe { ThinBox::new(page) });
        }

        let pt0: &mut Pagetable0 = pt_entry1.as_mut().ok_or(MMapError::MallocFail)?;

        Ok(&mut pt0.pte0[Self::get_idx(0, virtual_address)])
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
