mod virtio_blk;

use crate::memlayout::VIRTIO0;
use crate::util::Rounding;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut, Range};
use core::ptr::NonNull;
use core::str::from_utf8;
use efs::celled::Celled;
use efs::dev::sector::Address;
use efs::dev::size::Size;
use efs::dev::{Commit, Device, Slice};
use efs::{dev::error::DevError, error::Error};
use log::{error, trace};
use spin::Lazy;
use virtio_drivers::device::blk::SECTOR_SIZE;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use zerocopy::IntoBytes;

use crate::hw::disk::virtio_blk::{SectorData, SectorID, VirtIOBlkError};
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

pub struct Disk {
    dev: VirtIOBlk,
    buffer: Vec<SectorData>,
}

impl Disk {
    /// # Errors
    /// driver returned error
    /// # Panics
    /// disk returned invalid utf8
    pub fn get_name(&mut self) -> Result<String, virtio_drivers::Error> {
        let buf = &mut [0; 20];
        let name = self.device_id(buf)?;
        let name = from_utf8(name).expect("invalid disk name");
        Ok(name.to_owned())
    }

    fn resize(&mut self, size: SectorID) -> Result<(), DevError> {
        // trace!("reserving {size:?} for Disk IO");

        self.buffer.clear();
        self.buffer
            .try_reserve(size.in_sectors())
            .map_err(|err| {
                error!(
                    "failed to allocate buffer of size {:#x} sectors: {err}",
                    size.in_sectors()
                );
            })
            .map_err(|()| DevError::WriteZero)?;
        self.buffer.resize(size.in_sectors(), SectorData::DUMMY);
        Ok(())
    }
}

impl<FSE: core::error::Error> Device<u8, FSE> for Disk {
    fn size(&mut self) -> Size {
        Size(self.dev.capacity().in_bytes())
    }

    fn slice(&mut self, addr_range: Range<Address>) -> Result<Slice<'_, u8>, Error<FSE>> {
        // trace!("reading disk at {addr_range:#x?}");
        let start = addr_range.start.index().round_down2(SECTOR_SIZE) as u64;
        let end = addr_range.end.index().round_up2(SECTOR_SIZE) as u64;

        let start_sector = SectorID::from_bytes(start);
        let sectors = SectorID::from_bytes(end - start);
        self.resize(sectors)?;

        self.dev.read_blocks(start_sector, &mut self.buffer)?;

        let size0 = *(addr_range.end - addr_range.start);
        Ok(Slice::new(
            &self.buffer.as_bytes()[addr_range.start.index().modulo2(SECTOR_SIZE)..][..size0],
            addr_range.start,
        ))
    }

    fn commit(&mut self, commit: Commit<u8>) -> Result<(), Error<FSE>> {
        let start = *commit.addr();
        let offset = commit.addr().modulo2(SECTOR_SIZE);
        let bytes = commit.as_ref();
        let sector = SectorID::from_bytes(start.round_down2(SECTOR_SIZE) as u64);
        self.dev.write_data(sector, offset, bytes)?;
        Ok(())
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

#[allow(clippy::large_stack_frames)]
pub static MAIN_DISK: Lazy<Celled<Disk>> = Lazy::new(|| {
    // TODO: use dtb info
    const DEFAULT_DISK: NonNull<VirtIOHeader> = NonNull::new(VIRTIO0 as *mut _).unwrap();

    trace!("MAIN_DISK init");
    // SAFETY: in default qemu configuration `DEFAULT_DISK` points to disk's MMIO region
    Celled::new(unsafe { init_disk(DEFAULT_DISK) }.unwrap().into())
});

#[no_mangle]
extern "C" fn rs_disk_intr() {
    MAIN_DISK
        .try_lock()
        .as_mut()
        .map(|disk| disk.ack_interrupt());
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
