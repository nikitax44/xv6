use crate::kalloc::pages::page::Page;
use crate::memlayout::{PGSIZE, VIRTIO0};
use crate::vm::get_physical_address;
use alloc::vec::Vec;
use core::ptr::NonNull;
use log::{error, trace};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct HalImpl;
// SAFETY: see Impl safety blocks in methods
unsafe impl Hal for HalImpl {
    /// # Impl safety
    /// returns valid pointer.
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let buf = match Vec::<Page>::try_with_capacity(pages) {
            Ok(val) => val,
            Err(err) => {
                error!("failed to allocate buffer in HalImpl: {:?}", err);
                return (0, NonNull::dangling());
            }
        };
        let ptr = NonNull::from(buf.leak());
        debug_assert!(
            ptr.cast::<Page>().is_aligned(),
            "invalid allocation alignment"
        );
        let ptr: NonNull<u8> = ptr.cast();

        // SAFETY: ptr is valid for writes
        unsafe {
            ptr.write_bytes(0, pages * PGSIZE);
        }

        trace!("HAL: sharing allocated: ");
        // SAFETY: ptr is valid
        let pa = unsafe {
            Self::share(
                NonNull::slice_from_raw_parts(ptr, pages * PGSIZE),
                direction,
            )
        };

        (pa, ptr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        trace!("HAL: dealloc: {vaddr:?}@{paddr:0x}");

        // SAFETY: we now own the [vaddr, vaddr+pages*PGSIZE)
        unsafe {
            let _ = Vec::from_parts(vaddr.cast::<Page>(), pages, pages);
        }
        0
    }

    /// # Impl safety
    /// returns valid pointer that is not aliased by safety precondition
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        assert!(VIRTIO0 <= paddr, "convert oob address");
        assert!(paddr + size <= VIRTIO0 + 0xff, "convert oob address");
        trace!("HAL: mmio2virt: {paddr:0x}");
        assert_eq!(
            get_physical_address(paddr as *const ()).expect("VIRTIO MMIO is not kvmmap'ed"),
            paddr,
            "invalid kvm ptable"
        );
        NonNull::new(paddr as *mut _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        trace!("HAL: sharing {buffer:?}({:#x} bytes)", buffer.len());
        let ptr = buffer.cast::<()>().as_ptr().cast_const();
        get_physical_address(ptr).expect("page is not mapped")
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, _direction: BufferDirection) {
        trace!("HAL: unsharing {buffer:?}@{paddr:0x}");
        // nothing to do
    }
}
