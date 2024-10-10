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
        self.get().is_some()
    }

    #[must_use]
    /// # Panics
    /// if got invalid PTE
    pub fn get(&self) -> Option<(usize, Mode)> {
        Some(self.0)
            .filter(|val| val & Mode::VALID != 0)
            .map(|val| {
                (
                    (val >> Mode::SHIFT) * PGSIZE,
                    (val & Mode::ACCESS_MASK).try_into().expect("invalid PTE"),
                )
            })
    }

    pub fn unset(&mut self) {
        self.0 = 0;
    }

    /// # Errors
    /// addr and mode must be valid
    pub fn set(&mut self, addr: usize, mode: Mode) -> Result<(), PTError> {
        if mode == Mode::Table {
            return Err(PTError::InvalidMode);
        }
        self.set_raw(addr, mode)
    }

    fn set_raw(&mut self, addr: usize, mode: Mode) -> Result<(), PTError> {
        if addr % PGSIZE != 0 {
            return Err(PTError::InvalidPhysicalAddress);
        }
        self.0 = ((addr / PGSIZE) << Mode::SHIFT) | (mode as usize) | Mode::VALID;
        Ok(())
    }

    pub(super) fn set_pt(&mut self, pt: ThinBox<IPagetable>) {
        self.set_raw(pt.leak().as_ptr() as usize, Mode::Table)
            .unwrap();
    }

    pub(super) fn as_pt(&self) -> Option<&IPagetable> {
        self.get()
            .filter(|(_, mode)| *mode == Mode::Table)
            .map(|(addr, _)| addr as *const IPagetable)
            // SAFETY: contains reference IPagetable
            .map(|ptr| unsafe { &*ptr })
    }

    /// # Safety
    /// `PtEntry` must contain valid `IPagetable`
    #[expect(clippy::needless_pass_by_ref_mut, reason = "it returns &mut reference")]
    pub(super) fn as_pt_mut(&mut self) -> Option<&mut IPagetable> {
        self.get()
            .filter(|(_, mode)| *mode == Mode::Table)
            .map(|(addr, _)| addr as *mut IPagetable)
            // SAFETY: contains reference IPagetable
            .map(|ptr| unsafe { &mut *ptr })
    }
}

impl Debug for PtEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some((addr, mode)) = self.get() {
            write!(f, "PtEntry({addr:#x}, {mode:?})")
        } else {
            write!(f, "PtEntry(None)")
        }
    }
}
