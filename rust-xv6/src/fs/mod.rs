use crate::hw::hal::HalImpl;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::ops::Range;
use efs::dev::sector::Address;
use efs::dev::size::Size;
use efs::dev::{Commit, Device, Slice};
use efs::fs::error::FsError;
use thiserror::Error;
use virtio_drivers::device::blk::{VirtIOBlk, SECTOR_SIZE};
use virtio_drivers::transport::mmio::MmioTransport;
use zerocopy::{FromBytes, Immutable, IntoBytes};

#[derive(Debug, Error)]
enum Error {
    #[error("failed to allocate slice")]
    Alloc(#[from] TryReserveError),
    #[error("failed to read data from VirtIO backend")]
    VirtIORead(virtio_drivers::Error),
    #[error("failed to write data to VirtIO backend")]
    VirtIOWrite(virtio_drivers::Error),
}

impl From<Error> for efs::error::Error<Error> {
    fn from(value: Error) -> Self {
        Self::Fs(FsError::Implementation(value))
    }
}

pub struct VirtioDevice {
    dev: VirtIOBlk<HalImpl, MmioTransport>,
    buffer: Vec<Blk>,
}

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
struct Blk([u8; SECTOR_SIZE]);

impl Device<Blk, Error> for VirtioDevice {
    fn size(&mut self) -> Size {
        Size(self.dev.capacity() * u64::try_from(SECTOR_SIZE).unwrap())
    }

    fn slice(
        &mut self,
        addr_range: Range<Address>,
    ) -> Result<Slice<'_, Blk>, efs::error::Error<Error>> {
        self.buffer.clear();
        self.buffer
            .try_reserve_exact(addr_range.end.index() - addr_range.start.index())
            .map_err(Error::Alloc)?;

        self.dev
            .read_blocks(addr_range.start.index(), self.buffer.as_mut_bytes())
            .map_err(Error::VirtIORead)?;

        Ok(Slice::new(&self.buffer, addr_range.start))
    }

    fn commit(&mut self, commit: Commit<Blk>) -> Result<(), efs::error::Error<Error>> {
        self.dev
            .write_blocks(commit.addr().index(), commit.as_ref().as_bytes())
            .map_err(Error::VirtIOWrite)
            .map_err(Into::into)
    }
}
