pub mod virtio_blk;

use crate::memlayout::VIRTIO0;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use block_device::BlockDevice;
use core::ptr::NonNull;
use core::str::from_utf8;
use log::trace;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::hw::disk::virtio_blk::{SectorData, SectorID, VirtIOBlkError};
use crate::util::lazy::Lazy;
use crate::util::mutex::Mutex;
use virtio_blk::VirtIOBlk;

/// # Safety
/// `disk` must point to the valid MMIO region
/// # Panics
/// invalid pointer
/// # Errors
/// failed to initialize disk
/// failed to read disk id
#[allow(clippy::large_stack_frames)]
pub unsafe fn init_disk(disk: NonNull<VirtIOHeader>) -> Result<VirtIOBlk, VirtIOBlkError> {
    trace!("init_disk({disk:?})");
    // SAFETY: aligned and valid for the lifetime of this function by precondition
    let transport = unsafe { MmioTransport::new(disk).expect("failed to create transport") };
    trace!("MMIO transport initialized");
    let disk = VirtIOBlk::new(transport)?;
    trace!("disk initialized");

    Ok(disk)
}

#[derive(Copy, Clone)]
pub struct Disk {
    dev: &'static Mutex<VirtIOBlk>,
}

impl Disk {
    /// # Errors
    /// driver returned error
    /// # Panics
    /// disk returned invalid utf8
    pub fn get_name(&self) -> Result<String, virtio_drivers::Error> {
        let buf = &mut [0; 20];
        let name = self.dev.lock().device_id(buf)?;
        let name = from_utf8(name).expect("invalid disk name");
        Ok(name.to_owned())
    }

    pub const fn new(dev: &'static Mutex<VirtIOBlk>) -> Self {
        Self { dev }
    }

    pub const fn inner(&self) -> &Mutex<VirtIOBlk> {
        self.dev
    }
}

impl BlockDevice for Disk {
    type Error = VirtIOBlkError;

    fn read(
        &self,
        buf: &mut [u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        assert_eq!(
            buf.len(),
            number_of_blocks * Self::BLOCK_SIZE as usize,
            "size mismatch"
        );

        assert_eq!(
            address % Self::BLOCK_SIZE as usize,
            0,
            "address is not BLOCK-aligned"
        );

        if let Ok(slice) = bytemuck::try_cast_slice_mut(buf) {
            self.dev
                .lock()
                .read_blocks(SectorID::from_bytes(address as u64), slice)
        } else {
            let mut buffer = Vec::try_with_capacity(number_of_blocks)?;
            buffer.resize(number_of_blocks, SectorData::DUMMY);
            self.dev
                .lock()
                .read_blocks(SectorID::from_bytes(address as u64), &mut buffer)?;
            buf.copy_from_slice(bytemuck::cast_slice(&buffer));
            Ok(())
        }
    }

    fn write(
        &self,
        buf: &[u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        assert_eq!(
            buf.len(),
            number_of_blocks * Self::BLOCK_SIZE as usize,
            "size mismatch"
        );

        assert_eq!(
            address % Self::BLOCK_SIZE as usize,
            0,
            "address is not BLOCK-aligned"
        );

        if let Ok(slice) = bytemuck::try_cast_slice(buf) {
            self.dev
                .lock()
                .write_blocks(SectorID::from_bytes(address as u64), slice)
        } else {
            let mut buffer = Vec::try_with_capacity(number_of_blocks)?;
            buffer.resize(number_of_blocks, SectorData::DUMMY);
            bytemuck::cast_slice_mut(&mut buffer).copy_from_slice(buf);
            self.dev
                .lock()
                .write_blocks(SectorID::from_bytes(address as u64), &buffer)?;
            Ok(())
        }
    }
}

pub static MAIN_DISK: Lazy<Disk> = Lazy::new(|| {
    #[allow(clippy::large_stack_frames)]
    static MAIN_DISK_: Lazy<Mutex<VirtIOBlk>> = Lazy::new(|| {
        // TODO: use dtb info
        const DEFAULT_DISK: NonNull<VirtIOHeader> = NonNull::new(VIRTIO0 as *mut _).unwrap();

        trace!("MAIN_DISK init");
        // SAFETY: in default qemu configuration `DEFAULT_DISK` points to disk's MMIO region
        let disk = unsafe { init_disk(DEFAULT_DISK) }.unwrap();
        Mutex::new(disk)
    });
    Disk::new(&MAIN_DISK_)
});

#[no_mangle]
extern "C" fn rs_disk_intr() {
    MAIN_DISK
        .inner()
        .try_lock()
        .map(|mut disk| disk.ack_interrupt());
}
