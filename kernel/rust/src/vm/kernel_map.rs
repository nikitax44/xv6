use crate::kalloc::pages::KMEM;
use crate::memlayout::{
    end_text, FW_CFG, KERNBASE, KSTACK, PHYSTOP, PLIC, TEST0, TRAMPOLINE, UART0, VIRTIO0,
};
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::PTError;
use crate::{dbg, println};

const NPROC: usize = 64;

pub fn make_kernel_map() -> Result<Pagetable, PTError> {
    let mut pt = Pagetable::new()?;

    // uart registers
    pt.map_page(UART0, dbg!(UART0), Mode::PTE_RW)?;
    println!("UART0: {:?}", pt.walk_mut(UART0));

    // sifive test0/test1
    pt.map_page(TEST0, TEST0, Mode::PTE_RW)?;

    // qemu fw-cfg-mmio
    pt.map_page(FW_CFG, FW_CFG, Mode::PTE_RW)?;

    // virtio mmio disk interface
    pt.map_page(VIRTIO0, VIRTIO0, Mode::PTE_RW)?;

    // PLIC
    pt.map_pages(PLIC, PLIC, 0x400_0000, Mode::PTE_RW)?;

    // map kernel text executable and read-only.
    pt.map_pages(
        KERNBASE,
        KERNBASE,
        end_text() - KERNBASE,
        Mode::PTE_R | Mode::PTE_X,
    )?;

    // map kernel data and the physical RAM we'll make use of.
    pt.map_pages(end_text(), end_text(), PHYSTOP - end_text(), Mode::PTE_RW)?;

    // map the trampoline for trap entry/exit to
    // the highest virtual address in the kernel.
    pt.map_page(TRAMPOLINE, end_text(), Mode::PTE_X)?;

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
extern "C" fn kvmmake() -> Pagetable {
    make_kernel_map().expect("failed to create kernel map")
    // unsafe { kvmmake_() }
}

#[no_mangle]
extern "C" fn kvmdebug(mut pt: Pagetable) -> Pagetable {
    println!("UART0: {:?}", pt.walk_mut(UART0));
    println!("KERNBASE: {:?}", pt.walk_mut(KERNBASE));
    pt
}

extern "C" {
    fn kvmmake_() -> Pagetable;
}
