use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use crate::vm::mode::Mode;
use crate::vm::pt_inner::IPagetable;
use crate::vm::PTError;
use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PtEntry(usize);

impl PtEntry {
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.flags().contains(Mode::PTE_V)
    }

    #[must_use]
    pub fn get(&self) -> Option<(usize, Mode)> {
        Some(self.0)
            .map(|val| ((val >> Mode::SHIFT) * PGSIZE, Mode::from_bits_truncate(val)))
            .filter(|(_, m)| m.contains(Mode::PTE_V))
    }

    /// # Panics
    /// if `is_set` would return false
    #[must_use]
    pub fn flags(&self) -> Mode {
        self.get().map_or(Mode::empty(), |(_, mode)| mode)
    }
    pub fn unset(&mut self) {
        self.0 = 0;
    }

    /// # Errors
    /// addr and mode must be valid
    pub fn set(&mut self, addr: usize, mode: Mode) -> Result<(), PTError> {
        if !Mode::MASK.contains(mode) {
            return Err(PTError::InvalidMode);
        }
        if addr % PGSIZE != 0 {
            return Err(PTError::InvalidPhysicalAddress);
        }
        self.0 = ((addr / PGSIZE) << Mode::SHIFT) | (Mode::PTE_V | mode).bits();
        Ok(())
    }

    pub(super) fn set_pt(&mut self, pt: ThinBox<IPagetable>) {
        self.set(pt.leak().as_ptr() as usize, Mode::empty()).ok();
    }

    /// # Safety
    /// `PtEntry` must contain valid IPageTable
    pub(super) unsafe fn as_pt(&self) -> Option<&IPagetable> {
        self.get()
            .filter(|(_, m)| m.contains(Mode::PTE_V))
            .map(|(addr, _)| addr as *const IPagetable)
            // SAFETY: precondition
            .map(|ptr| unsafe { &*ptr })
    }

    /// # Safety
    /// `PtEntry` must contain valid IPageTable
    pub(super) unsafe fn as_pt_mut(&mut self) -> Option<&mut IPagetable> {
        self.get()
            .filter(|(_, m)| m.contains(Mode::PTE_V))
            .map(|(addr, _)| addr as *mut IPagetable)
            // SAFETY: precondition
            .map(|ptr| unsafe { &mut *ptr })
    }
}

impl Debug for PtEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (addr, mode) = self.get().unwrap_or((0, Mode::empty()));
        write!(f, "PtEntry({:#x}, {:?})", addr, mode)
    }
}
