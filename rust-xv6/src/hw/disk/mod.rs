use crate::hw::hal::HalImpl;
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
use virtio_drivers::device::blk;
use virtio_drivers::device::blk::SECTOR_SIZE;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

pub type VirtIOBlk = blk::VirtIOBlk<HalImpl, MmioTransport>;

/// # Safety
/// `disk` must point to the valid MMIO region
/// # Panics
/// invalid pointer
/// # Errors
/// failed to initialize disk
/// failed to read disk id
#[allow(clippy::large_stack_frames)]
pub unsafe fn init_disk(disk: NonNull<VirtIOHeader>) -> Result<VirtIOBlk, virtio_drivers::Error> {
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
    buffer: Vec<u8>,
}

impl Disk {
    /// # Errors
    /// driver returned error
    /// # Panics
    /// disk returned invalid utf8
    pub fn get_name(&mut self) -> Result<String, virtio_drivers::Error> {
        let buf = &mut [0; 20];
        let sz = self.device_id(buf)?;
        let name = from_utf8(&buf[..sz]).expect("invalid disk name");
        Ok(name.to_owned())
    }
}

impl<FSE: core::error::Error> Device<u8, FSE> for Disk {
    fn size(&mut self) -> Size {
        Size(self.dev.capacity() * u64::try_from(SECTOR_SIZE).unwrap())
    }

    fn slice(&mut self, addr_range: Range<Address>) -> Result<Slice<'_, u8>, Error<FSE>> {
        // trace!("reading disk at {addr_range:#x?}");
        let start = addr_range.start.index().round_down2(SECTOR_SIZE);
        let end = addr_range.end.index().round_up2(SECTOR_SIZE);

        let size = end - start;
        self.buffer.clear();
        self.buffer
            .try_reserve_exact(size)
            .map_err(|err| error!("failed to allocate buffer of size {size:#x}: {err}"))
            .map_err(|()| DevError::WriteZero)?;
        self.buffer.resize(size, 0x36);

        self.dev
            .read_blocks(start / SECTOR_SIZE, self.buffer.as_mut_slice())
            .map_err(|err| error!("failed to read blocks: {err}"))
            .map_err(|()| DevError::WriteZero)?;

        let size0 = (addr_range.end - addr_range.start).index();
        Ok(Slice::new(
            &self.buffer[addr_range.start.index().modulo2(SECTOR_SIZE)..][..size0],
            addr_range.start,
        ))
    }

    fn commit(&mut self, commit: Commit<u8>) -> Result<(), Error<FSE>> {
        let (chunks, rest) = commit.as_ref().as_chunks::<SECTOR_SIZE>();
        let index = commit.addr().index();
        for (i, chunk) in chunks.iter().enumerate() {
            self.dev
                .write_blocks(index + i, chunk)
                .map_err(|err| error!("failed to write blocks: {err}"))
                .map_err(|()| DevError::WriteZero)?;
        }
        if !rest.is_empty() {
            let buf = &mut [0; SECTOR_SIZE];
            let i = index + chunks.len();
            buf[..rest.len()].copy_from_slice(rest);

            self.dev
                .write_blocks(i, buf)
                .map_err(|err| error!("failed to write last block: {err}"))
                .map_err(|()| DevError::WriteZero)?;
        }
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
