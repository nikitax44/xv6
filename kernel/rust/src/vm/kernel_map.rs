use crate::addrof_symbol;
use crate::dtb::DTB;
use crate::kalloc::pages::page::PageHandle;
use crate::kalloc::pages::KMEM;
use crate::kalloc::region::Region;
use crate::memlayout::{
    addrof_end_kernel, addrof_end_text, addrof_kernel, FW_CFG, KSTACK, PGSIZE, PLIC, TEST0,
    TRAMPOLINE, UART0, VIRTIO0,
};
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::PTError;
use alloc::vec;
use alloc::vec::Vec;

const NPROC: usize = 64;

/// # Safety
/// no one owns memory outside of kernel and bios regions,
/// or it is declared in dtb's reserved regions
#[allow(clippy::large_stack_frames, reason = "stack is reused")]
pub(super) unsafe fn make_kernel_map() -> Result<Pagetable<'static>, PTError> {
    let xv6_mem = xv6_memory();

    let mem: Region;
    let mut reserved;
    if let Some((dtb, dtb_reg)) = DTB.get() {
        let mems: Vec<Region> = dtb.memory().regions().map(Region::from).collect();
        (mem, reserved) = Region::join_multiple(mems);
        dtb.memory_reservations()
            .map(Region::from)
            .chain([Region::new(0x8000_0000, addrof_kernel()), *dtb_reg])
            .collect_into(&mut reserved);
    } else {
        mem = Region::new(addrof_kernel(), 0x8800_0000);
        reserved = vec![];
    }
    xv6_mem
        .iter()
        .map(|(reg, _)| *reg)
        .filter(|&reg| mem.contains(reg))
        .collect_into(&mut reserved);
    let mut free = mem.split_multiple(reserved);
    free.retain_mut(|reg| {
        *reg = reg.align_shrink();
        reg.size() != 0
    });

    {
        let mut kmem = KMEM.lock();
        for reg in &free {
            reg.pages().for_each(|addr| {
                kmem.free(PageHandle::new(
                    // SAFETY: no one owns that memory by precondition
                    unsafe { addr.cast_mut().as_mut().unwrap() },
                    None,
                ));
            });
        }
    }

    // must be called after kfree's
    let mut pt = Pagetable::alloc()?;

    for (reg, mode) in xv6_mem {
        pt.map_reg(reg, mode)?;
    }

    for &reg in &free {
        pt.map_reg(reg, Mode::_RW_)?;
    }

    if let Some((_, dtb_reg)) = DTB.get() {
        pt.map_reg(dtb_reg.align_grow(), Mode::_R__)?;
    }

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    pt.map_page(TRAMPOLINE, addrof_symbol!(trampoline), Mode::___X)?;

    // allocate and map a kernel stack for each process.
    proc_mapstacks(&mut pt)?;

    // done mapping
    Ok(pt)
}

fn xv6_memory() -> Vec<(Region, Mode)> {
    const fn reg(start: usize, size: usize, mode: Mode) -> (Region, Mode) {
        (Region::new(start, start + size), mode)
    }
    const fn page(page: usize, mode: Mode) -> (Region, Mode) {
        reg(page, PGSIZE, mode)
    }
    vec![
        page(UART0, Mode::_RW_),
        // sifive test0/test1
        page(TEST0, Mode::_RW_),
        // qemu fw-cfg-mmio
        page(FW_CFG, Mode::_RW_),
        // virtio mmio disk interface
        page(VIRTIO0, Mode::_RW_),
        // PLIC
        reg(PLIC, 0x0400_0000, Mode::_RW_),
        // map kernel text executable and read-only.
        reg(
            addrof_kernel(),
            addrof_end_text() - addrof_kernel(),
            Mode::_R_X,
        ),
        // map kernel data and the physical RAM we'll make use of.
        reg(
            addrof_end_text(),
            addrof_end_kernel() - addrof_end_text(),
            Mode::_RW_,
        ),
    ]
}

#[allow(clippy::large_stack_frames, reason = "it is fine")]
fn proc_mapstacks(pt: &mut Pagetable) -> Result<(), PTError> {
    for i in 0..NPROC {
        let page1 = KMEM
            .lock()
            .alloc("proc stack")
            .map_err(PTError::AllocFail)?;
        let page2 = KMEM
            .lock()
            .alloc("proc stack")
            .map_err(PTError::AllocFail)?;
        let va = KSTACK(i);
        pt.map_page(va, page1.into_box().leak().as_ptr() as usize, Mode::_RW_)?;
        pt.map_page(
            va + PGSIZE,
            page2.into_box().leak().as_ptr() as usize,
            Mode::_RW_,
        )?;
    }
    Ok(())
}
