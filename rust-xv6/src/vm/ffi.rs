use crate::errno::ErrNo;
use crate::kalloc::thin_box::ThinBox;
use crate::kalloc::Xv6Alloc;
use crate::kalloc::{get_kalloc, Page};
use crate::memlayout::{FROM_KSTACK_BOTTOM, PGSIZE, STACK_SIZE, TO_KSTACK_BOTTOM};
use crate::util::Mutex;
use crate::vm::kernel_map::make_kernel_map;
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use alloc::vec::Vec;
use core::ptr::NonNull;
use log::error;
use spin::rwlock::RwLock;

static KERNEL_PAGETABLE: RwLock<Option<Pagetable>> = RwLock::new(None);

#[derive(Copy, Clone)]
struct KernelStack(pub usize);

impl KernelStack {
    fn bottom(self) -> *mut Page {
        TO_KSTACK_BOTTOM(self.0)
    }
    fn top(self) -> *mut Page {
        (self.bottom() as usize + STACK_SIZE * PGSIZE) as *mut _
    }

    fn from_bottom(ptr: *mut Page) -> Self {
        Self(FROM_KSTACK_BOTTOM(ptr))
    }

    fn pages(self) -> impl Iterator<Item = *mut Page> {
        let bot = self.bottom();
        (0..STACK_SIZE).map(move |i| (bot as usize + i * PGSIZE) as *mut _)
    }

    fn map(self) -> Result<(), PTError> {
        for ptr in self.pages() {
            let page = ThinBox::alloc_page()?;
            KERNEL_PAGETABLE
                .write()
                .as_mut()
                .expect("map_page on None")
                .map_page(ptr as usize, page.leak().as_ptr() as usize, Mode::_RW_)?;
        }
        Ok(())
    }

    unsafe fn unmap(self) -> Result<(), PTError> {
        for ptr in self.pages() {
            // SAFETY: precondition
            unsafe {
                KERNEL_PAGETABLE
                    .write()
                    .as_mut()
                    .expect("unmap_page on None")
                    .unmap_page_and_free(ptr as usize)?;
            }
        }
        Ok(())
    }
}

struct FreeKernelStacks {
    free_stack_slots: Vec<KernelStack>,
    next_stack_slot: usize,
}

static UNUSED_KERNEL_STACKS: Mutex<FreeKernelStacks> = Mutex::new(FreeKernelStacks {
    free_stack_slots: Vec::new(),
    next_stack_slot: 0,
});

#[no_mangle]
extern "C" fn request_stack() -> Option<NonNull<Page>> {
    let mut guard = UNUSED_KERNEL_STACKS.lock();
    let slot = guard.free_stack_slots.pop().unwrap_or_else(|| {
        let slot = KernelStack(guard.next_stack_slot);
        guard.next_stack_slot += 1;
        slot
    });
    slot.map()
        .inspect_err(|err| {
            let info = get_kalloc().try_get_info();
            error!("failed to map kernel stack: {err}; {info:?}");
        })
        .ok()?;
    NonNull::new(slot.top())
}

#[no_mangle]
unsafe extern "C" fn release_stack(top: Option<NonNull<Page>>) {
    let top = top.expect("release_stack(NULL)");
    let bottom = top.as_ptr() as usize - PGSIZE * STACK_SIZE;
    let kstack = KernelStack::from_bottom(bottom as *mut Page);
    // SAFETY: precondition
    unsafe { kstack.unmap().expect("failed to unmap kernel stack") };

    let mut guard = UNUSED_KERNEL_STACKS.lock();

    // worst-case scenario is address space overflow
    guard.free_stack_slots.try_reserve(1).ok();
    guard.free_stack_slots.push_within_capacity(kstack).ok();
}

/// # Safety
/// no one owns memory outside of kernel and bios regions,
/// or it is declared in dtb's reserved regions
#[no_mangle]
unsafe extern "C" fn kvminit() {
    // SAFETY: precondition
    let pt = unsafe { make_kernel_map() }.expect("failed to create kernel map");

    let mut kpt = KERNEL_PAGETABLE.write();
    if let Some(_old_pt) = kpt.replace(pt) {
        panic!("KERNEL_PAGETABLE overwrite")
    }
}

#[no_mangle]
extern "C" fn with_kernel_pagetable(op: extern "C" fn(&mut IPagetable)) {
    let mut kpt = KERNEL_PAGETABLE.write();
    let kpt = kpt
        .as_mut()
        .expect("kernel pagetable was expected to be present");
    op(kpt
        .inner_mut()
        .expect("kernel pagetable is the Pagetable::Ref variant"));
}

//int mappages(pagetable_t pagetable, u64 va, u64 size, u64 pa, int perm)
#[no_mangle]
extern "C" fn mappages(
    pt: &mut IPagetable,
    virtual_address: usize,
    size: usize,
    physical_address: usize,
    perm: usize,
) -> ErrNo {
    /*warn!(
        "rwarn {:#x}, {:#x}",
        pt as *const _ as usize, virtual_address
    );*/
    let perm = perm.try_into().expect("invalid access mode");
    let mut pt = Pagetable::from_mut(pt);
    let result = pt.map_pages(virtual_address, physical_address, size, perm);
    if let Err(err) = result {
        return match err {
            PTError::AllocFail(_err) => ErrNo::ENOMEM,
            _ => panic!("ffi::mappages: {:?}", err),
        };
    }
    ErrNo::SUCCESS
}

// Look up a virtual address, return the physical address,
// or 0 if not mapped.
// Can only be used to look up user pages.
// u64 walkaddr(pagetable_t pagetable, u64 va)
#[no_mangle]
extern "C" fn walkaddr(pagetable: &IPagetable, virtual_address: usize) -> usize {
    Pagetable::from_ref(pagetable)
        .walk(virtual_address)
        .ok()
        .as_ref()
        .and_then(PtEntry::get)
        .filter(|(_addr, mode)| mode.get_u())
        .map_or(0, |(addr, _mode)| addr)
}
