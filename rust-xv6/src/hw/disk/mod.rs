use crate::hw::hal::HalImpl;
use crate::memlayout::VIRTIO0;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut, Range};
use core::ptr::NonNull;
use core::str::from_utf8;
use efs::celled::Celled;
use efs::dev::sector::Address;
use efs::dev::size::Size;
use efs::dev::{Commit, Device, Slice};
use efs::{dev::error::DevError, error::Error};
use log::{error, info};
use spin::Lazy;
use virtio_drivers::device::blk;
use virtio_drivers::device::blk::SECTOR_SIZE;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use zerocopy::IntoBytes;

pub type VirtIOBlk = blk::VirtIOBlk<HalImpl, MmioTransport>;

/// # Safety
/// `disk` must point to the valid MMIO region
/// # Panics
/// invalid pointer
/// # Errors
/// failed to initialize disk
/// failed to read disk id
#[allow(clippy::large_stack_frames, reason = "no way to deal with it")]
pub unsafe fn init_disk(disk: NonNull<VirtIOHeader>) -> Result<VirtIOBlk, virtio_drivers::Error> {
    // SAFETY: aligned and valid for the lifetime of this function by precondition
    let transport = unsafe { MmioTransport::new(disk).expect("failed to create transport") };

    let mut disk = VirtIOBlk::new(transport)?;

    let buf = &mut [0; 20];
    let sz = disk.device_id(buf)?;
    let name = from_utf8(&buf[..sz]).expect("invalid disk name");

    info!(
        "VirtIO block device {}: {} kB",
        name,
        disk.capacity() * SECTOR_SIZE as u64 / 1024
    );

    Ok(disk)
}

pub struct Disk {
    dev: VirtIOBlk,
    buffer: Vec<Blk>,
}

type Blk = [u8; SECTOR_SIZE];

impl<FSE: core::error::Error> Device<u8, FSE> for Disk {
    fn size(&mut self) -> Size {
        Size(self.dev.capacity() * u64::try_from(SECTOR_SIZE).unwrap())
    }

    fn slice(&mut self, addr_range: Range<Address>) -> Result<Slice<'_, u8>, Error<FSE>> {
        self.buffer.clear();
        self.buffer
            .try_reserve_exact(addr_range.end.index() - addr_range.start.index())
            .map_err(|err| error!("failed to allocate buffer: {err}"))
            .map_err(|()| DevError::WriteZero)?;

        self.dev
            .read_blocks(
                addr_range.start.index(),
                self.buffer.as_mut_slice().as_mut_bytes(),
            )
            .map_err(|err| error!("failed to read blocks: {err}"))
            .map_err(|()| DevError::WriteZero)?;

        Ok(Slice::new(self.buffer.as_bytes(), addr_range.start))
    }

    fn commit(&mut self, commit: Commit<u8>) -> Result<(), Error<FSE>> {
        self.dev
            .write_blocks(commit.addr().index(), commit.as_ref())
            .map_err(|err| error!("failed to write blocks: {err}"))
            .map_err(|()| DevError::WriteZero.into())
    }
}

impl From<VirtIOBlk> for Disk {
    fn from(value: VirtIOBlk) -> Self {
        Self {
            dev: value,
            buffer: Vec::new(),
        }
    }
}

pub static MAIN_DISK: Lazy<Celled<Disk>> = Lazy::new(|| {
    // TODO: use dtb info
    const DEFAULT_DISK: NonNull<VirtIOHeader> = NonNull::new(VIRTIO0 as *mut _).unwrap();

    // SAFETY: in default qemu configuration `DEFAULT_DISK` points to disk's MMIO region
    Celled::new(unsafe { init_disk(DEFAULT_DISK) }.unwrap().into())
});

#[no_mangle]
extern "C" fn rs_disk_intr() {
    let mut guard = MAIN_DISK.lock();
    guard.ack_interrupt();
}

impl Deref for Disk {
    type Target = VirtIOBlk;

    fn deref(&self) -> &Self::Target {
        &self.dev
    }
}

impl DerefMut for Disk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dev
    }
}
