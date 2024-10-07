use crate::kalloc::pages::KMEM;
use crate::memlayout::{
    addrof_end_text, addrof_trampoline, FW_CFG, KERNBASE, KSTACK, PHYSTOP, PLIC, TEST0, TRAMPOLINE,
    UART0, VIRTIO0,
};
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::PTError;

const NPROC: usize = 64;

pub fn make_kernel_map() -> Result<Pagetable<'static>, PTError> {
    let mut pt = Pagetable::alloc()?;

    // uart registers
    pt.map_page(UART0, UART0, Mode::PTE_RW)?;

    // sifive test0/test1
    pt.map_page(TEST0, TEST0, Mode::PTE_RW)?;

    // qemu fw-cfg-mmio
    pt.map_page(FW_CFG, FW_CFG, Mode::PTE_RW)?;

    // virtio mmio disk interface
    pt.map_page(VIRTIO0, VIRTIO0, Mode::PTE_RW)?;

    // PLIC
    pt.map_pages(PLIC, PLIC, 0x0400_0000, Mode::PTE_RW)?;

    // map kernel text executable and read-only.
    pt.map_pages(
        KERNBASE,
        KERNBASE,
        addrof_end_text() - KERNBASE,
        Mode::PTE_R | Mode::PTE_X,
    )?;

    // map kernel data and the physical RAM we'll make use of.
    pt.map_pages(
        addrof_end_text(),
        addrof_end_text(),
        PHYSTOP - addrof_end_text(),
        Mode::PTE_RW,
    )?;

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    pt.map_page(TRAMPOLINE, addrof_trampoline(), Mode::PTE_X)?;

    // allocate and map a kernel stack for each process.
    proc_mapstacks(&mut pt)?;

    // done mapping
    Ok(pt)
}

fn proc_mapstacks(pt: &mut Pagetable) -> Result<(), PTError> {
    for i in 0..NPROC {
        let page = KMEM.lock().alloc().ok_or(PTError::AllocFail)?;
        let va = KSTACK(i);
        pt.map_page(va, page.leak().as_ptr() as usize, Mode::PTE_RW)?;
    }
    Ok(())
}

#[no_mangle]
extern "C" fn kvmmake() -> Pagetable<'static> {
    make_kernel_map().expect("failed to create kernel map")
}
