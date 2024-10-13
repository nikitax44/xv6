use crate::addrof_symbol;
use crate::kalloc::pages::KMEM;
use crate::memlayout::{
    addrof_end_text, addrof_kernel, FW_CFG, KSTACK, PHYSTOP, PLIC, TEST0, TRAMPOLINE, UART0,
    VIRTIO0,
};
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::PTError;

const NPROC: usize = 64;

pub fn make_kernel_map() -> Result<Pagetable<'static>, PTError> {
    let mut pt = Pagetable::alloc()?;

    // uart registers
    pt.map_page(UART0, UART0, Mode::_RW_)?;

    // sifive test0/test1
    pt.map_page(TEST0, TEST0, Mode::_RW_)?;

    // qemu fw-cfg-mmio
    pt.map_page(FW_CFG, FW_CFG, Mode::_RW_)?;

    // virtio mmio disk interface
    pt.map_page(VIRTIO0, VIRTIO0, Mode::_RW_)?;

    // PLIC
    pt.map_pages(PLIC, PLIC, 0x0400_0000, Mode::_RW_)?;

    // map kernel text executable and read-only.
    pt.map_pages(
        addrof_kernel(),
        addrof_kernel(),
        addrof_end_text() - addrof_kernel(),
        Mode::_R_X,
    )?;

    // map kernel data and the physical RAM we'll make use of.
    pt.map_pages(
        addrof_end_text(),
        addrof_end_text(),
        PHYSTOP - addrof_end_text(),
        Mode::_RW_,
    )?;

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    pt.map_page(TRAMPOLINE, addrof_symbol!(trampoline), Mode::___X)?;

    // allocate and map a kernel stack for each process.
    proc_mapstacks(&mut pt)?;

    // done mapping
    Ok(pt)
}

fn proc_mapstacks(pt: &mut Pagetable) -> Result<(), PTError> {
    for i in 0..NPROC {
        let page = KMEM
            .lock()
            .alloc("proc stack")
            .map_err(PTError::AllocFail)?;
        let va = KSTACK(i);
        pt.map_page(va, page.into_box().leak().as_ptr() as usize, Mode::_RW_)?;
    }
    Ok(())
}
