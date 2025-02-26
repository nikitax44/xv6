use crate::kalloc::region::Region;
use crate::kalloc::thin_box::ThinBox;
use crate::kalloc::Page;
use crate::memlayout::{PGSHIFT, PGSIZE};
use crate::vm::mode::Mode;
use crate::vm::pt_inner::GigaPT;
use crate::vm::pte::{GigaPtEntry, KiloPtEntry, MegaPtEntry};
use crate::vm::PTError;
use alloc::boxed::Box;

#[derive(Debug)]
pub struct Pagetable<'inner> {
    inner: Inner<'inner>,
}

#[derive(Debug)]
enum Inner<'inner> {
    Owned(ThinBox<GigaPT>),
    Ref(&'inner GigaPT),
    Mut(&'inner mut GigaPT),
}

impl<'inner> Pagetable<'inner> {
    pub const KPGSIZE: usize = PGSIZE;
    pub const MPGSIZE: usize = Self::KPGSIZE * Self::PT_ENTRIES;
    pub const GPGSIZE: usize = Self::MPGSIZE * Self::PT_ENTRIES;

    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn new() -> Result<Self, PTError> {
        Ok(Self {
            inner: Inner::Owned(ThinBox::alloc()?.zeroed()),
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
        }
        if addr % PGSIZE != 0 {
            return Err(PTError::UnalignedVirtualAddress(addr));
        }
        Ok(())
    }

    pub(super) const fn inner_ref(&self) -> &GigaPT {
        match &self.inner {
            Inner::Owned(inner) => inner.const_deref(),
            Inner::Ref(inner) => inner,
            Inner::Mut(inner) => inner,
        }
    }

    pub(super) const fn inner_mut(&mut self) -> Result<&mut GigaPT, PTError> {
        match &mut self.inner {
            Inner::Owned(inner) => Ok(inner.const_deref_mut()),
            Inner::Ref(_inner) => Err(PTError::ROPagetable),
            Inner::Mut(inner) => Ok(inner),
        }
    }

    pub(super) const fn from_ref(inner: &'inner GigaPT) -> Self {
        Self {
            inner: Inner::Ref(inner),
        }
    }

    pub(super) const fn from_mut(inner: &'inner mut GigaPT) -> Self {
        Self {
            inner: Inner::Mut(inner),
        }
    }

    /// # Errors
    /// see `MMapError`
    pub fn walk_giga(&self, virtual_address: usize) -> Result<&GigaPtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        let pt = self.inner_ref();
        Ok(&pt[Self::get_idx(2, virtual_address)])
    }

    /// # Errors
    /// pagetable is readonly
    pub fn walk_giga_mut(&mut self, virtual_address: usize) -> Result<&mut GigaPtEntry, PTError> {
        Self::verify_va(virtual_address)?;

        let pt = self.inner_mut()?;
        Ok(&mut pt[Self::get_idx(2, virtual_address)])
    }

    /// # Errors
    /// page is not mapped
    /// self's invariants aren't held
    //#[attr_wrapper::time_me(0)]
    pub fn translate(&self, va: usize) -> Result<usize, PTError> {
        let page = va / PGSIZE * PGSIZE;
        Ok(self.walk_kilo(page)?.get()?.0 + (va - page))
    }

    ///# Errors
    /// see `MMapError`
    /// # Safety
    /// mapped page must be allocated
    pub unsafe fn unmap_page_and_free(&mut self, virtual_address: usize) -> Result<(), PTError> {
        let pte = self.walk_kilo_mut(virtual_address)?;
        let addr = pte.get()?.0;
        pte.unset();

        // SAFETY: precondition
        unsafe {
            // use physical address because virtual just got unmapped
            let _ = Box::from_raw(addr as *mut Page);
        }
        Ok(())
    }

    /// # Errors
    /// `size` is not page-aligned
    /// see `map_page`
    //#[attr_wrapper::time_me(0)]
    pub fn map_pages(
        &mut self,
        virtual_address: usize,
        physical_address: usize,
        size: usize,
        mode: Mode,
    ) -> Result<(), PTError> {
        if size % Self::KPGSIZE != 0 {
            return Err(PTError::UnalignedSize(size));
        }
        let mut pos = 0;
        while pos < size {
            if (virtual_address + pos) % Self::GPGSIZE == 0 && (pos + Self::GPGSIZE) <= size {
                self.map_giga(virtual_address + pos, physical_address + pos, mode)?;
                pos += Self::GPGSIZE;
                continue;
            }
            if (virtual_address + pos) % Self::MPGSIZE == 0 && (pos + Self::MPGSIZE) <= size {
                self.map_mega(virtual_address + pos, physical_address + pos, mode)?;
                pos += Self::MPGSIZE;
                continue;
            }
            self.map_kilo(virtual_address + pos, physical_address + pos, mode)?;
            pos += Self::KPGSIZE;
        }
        Ok(())
    }

    /// # Errors
    /// region is not page-aligned or is empty
    /// see `map_page`
    //#[attr_wrapper::time_me(0)]
    pub fn map_reg(&mut self, reg: Region, mode: Mode) -> Result<(), PTError> {
        self.map_pages(reg.start(), reg.start(), reg.size(), mode)
    }
}

impl Drop for Inner<'_> {
    fn drop(&mut self) {
        if let Inner::Owned(_) = self {
            panic!("owned pagetable got dropped");
        }
    }
}

macro_rules! walking {
    ($walk:ident, $walk_mut:ident, $map:ident $(; $prev_walk:ident, $prev_walk_mut:ident, $ret:ty, $id:literal)?) => {
        impl<'inner> Pagetable<'inner> {
            /// # Errors
            /// see `MMapError`
            #[track_caller]
            pub fn $map(
                &mut self,
                virtual_address: usize,
                physical_address: usize,
                perm: Mode,
            ) -> Result<(), PTError> {
                let pte = self.$walk_mut(virtual_address)?;
                if pte.is_set() {
                    return Err(PTError::Remap(virtual_address));
                }
                pte.set(physical_address, perm)?;
                Ok(())
            }

            $(
            /// # Errors
            /// see `MMapError`
            pub fn $walk(&self, virtual_address: usize) -> Result<&$ret, PTError> {
                Self::verify_va(virtual_address)?;

                let pt = self.$prev_walk(virtual_address)?.as_pt()?;
                Ok(&pt[Self::get_idx($id, virtual_address)])
            }
            )?

            $(
            /// # Errors
            /// see `MMapError`
            pub fn $walk_mut(&mut self, virtual_address: usize) -> Result<&mut $ret, PTError> {
                Self::verify_va(virtual_address)?;

                let pt = self.$prev_walk_mut(virtual_address)?;
                if !pt.is_set() {
                    pt.set_pt(ThinBox::alloc()?.zeroed());
                }
                let pt = pt.as_pt_mut()?;
                Ok(&mut pt[Self::get_idx($id, virtual_address)])
            }
            )?
        }
    };
}

walking!(walk_giga, walk_giga_mut, map_giga);
walking!(
    walk_mega,
    walk_mega_mut,
    map_mega;
    walk_giga,
    walk_giga_mut,
    MegaPtEntry,
    1
);
walking!(
    walk_kilo,
    walk_kilo_mut,
    map_kilo;
    walk_mega,
    walk_mega_mut,
    KiloPtEntry,
    0
);
