use crate::hw::hal::HalImpl;
use crate::static_assert;
use crate::util::{copy_data, u64_to_usize};
use alloc::collections::TryReserveError;
use bytemuck::{Pod, Zeroable};
use thiserror::Error;
use virtio_drivers::device::blk;
use virtio_drivers::transport::mmio::MmioTransport;
use virtio_drivers::Error;
use zerocopy::{FromBytes, Immutable, IntoBytes};

pub const SECTOR_SIZE: u64 = blk::SECTOR_SIZE as u64;

pub struct VirtIOBlk {
    inner: blk::VirtIOBlk<HalImpl, MmioTransport>,
}

#[derive(Debug, Error)]
pub enum VirtIOBlkError {
    #[error("VirtIOBlkError({0})")]
    VirtIOBlkError(virtio_drivers::Error),
    #[error("AllocError({0})")]
    TryReserveError(#[from] TryReserveError),
}

impl From<virtio_drivers::Error> for VirtIOBlkError {
    fn from(value: Error) -> Self {
        Self::VirtIOBlkError(value)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SectorID(u64);
static_assert!(blk::SECTOR_SIZE == 512);
#[repr(C, align(512))]
#[derive(IntoBytes, FromBytes, Immutable, Copy, Clone, Pod, Zeroable)]
pub struct SectorData(pub [u8; blk::SECTOR_SIZE]);

impl SectorID {
    /// # Panics
    /// `bytes` is not `SECTOR_SIZE`-aligned
    #[must_use]
    pub fn from_bytes(bytes: u64) -> Self {
        assert_eq!(
            bytes % SECTOR_SIZE,
            0,
            "byte offset is not SECTOR_SIZE-aligned"
        );
        Self(bytes / SECTOR_SIZE)
    }

    /// # Panics
    /// never
    #[must_use]
    pub fn parts_from_bytes(bytes: u64) -> (Self, usize) {
        (
            Self(bytes / SECTOR_SIZE),
            (bytes % SECTOR_SIZE).try_into().unwrap(),
        )
    }

    #[must_use]
    pub const fn in_bytes(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn in_sectors(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl SectorData {
    pub const DUMMY: Self = Self([0x36; blk::SECTOR_SIZE]);
}

impl VirtIOBlk {
    #[allow(clippy::large_stack_frames)]
    /// # Errors
    /// failed to create `VirtIOBlk`
    pub fn new(transport: MmioTransport) -> Result<Self, VirtIOBlkError> {
        Ok(Self {
            inner: blk::VirtIOBlk::new(transport)?,
        })
    }

    /// # Errors
    /// failed to read name
    pub fn device_id<'v>(&mut self, id: &'v mut [u8; 20]) -> virtio_drivers::Result<&'v [u8]> {
        let sz = self.inner.device_id(id)?;
        Ok(&id[..sz])
    }

    #[must_use]
    pub fn capacity(&self) -> SectorID {
        SectorID(self.inner.capacity())
    }

    pub fn ack_interrupt(&mut self) -> bool {
        self.inner.ack_interrupt()
    }

    /// # Errors
    /// failed to read disk
    pub fn read_block(
        &mut self,
        block_id: SectorID,
        buf: &mut SectorData,
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .read_blocks(u64_to_usize(block_id.0), &mut buf.0)?)
    }

    /// # Errors
    /// failed to read disk
    pub fn read_blocks(
        &mut self,
        start_block_id: SectorID,
        buf: &mut [SectorData],
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .read_blocks(u64_to_usize(start_block_id.0), buf.as_mut_bytes())?)
    }

    /// # Errors
    /// failed to write to disk
    pub fn write_block(
        &mut self,
        block_id: SectorID,
        buf: &SectorData,
    ) -> Result<(), VirtIOBlkError> {
        Ok(self.inner.write_blocks(u64_to_usize(block_id.0), &buf.0)?)
    }

    /// # Errors
    /// failed to write to disk
    pub fn write_blocks(
        &mut self,
        start_block_id: SectorID,
        buf: &[SectorData],
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .write_blocks(u64_to_usize(start_block_id.0), buf.as_bytes())?)
    }

    /// # Errors
    /// failed to write to disk
    fn write_incomplete(
        &mut self,
        block_id: SectorID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), VirtIOBlkError> {
        assert_ne!(data.len(), 0, "write empty buffer");
        assert!(
            offset + data.len() <= blk::SECTOR_SIZE,
            "overflow in write_incomplete"
        );
        let buf = &mut SectorData::DUMMY.clone();
        self.read_block(block_id, buf)?;
        buf.0[offset..][..data.len()].clone_from_slice(data);
        self.write_block(block_id, buf)
    }

    fn write_prefix<'v>(
        &mut self,
        sector: SectorID,
        offset: usize,
        bytes: &'v [u8],
    ) -> Result<&'v [u8], VirtIOBlkError> {
        let point = blk::SECTOR_SIZE - offset;
        let bytes = if let Some((head, rest)) = bytes.split_at_checked(point) {
            self.write_incomplete(sector, offset, head)?;
            rest
        } else {
            self.write_incomplete(sector, offset, bytes)?;
            &[]
        };
        Ok(bytes)
    }

    /// # Errors
    /// failed to write to disk
    pub fn write_data(&mut self, position: u64, data: &[u8]) -> Result<(), VirtIOBlkError> {
        let (sector, offset) = SectorID::parts_from_bytes(position);

        let (data, mut sector) = if offset != 0 {
            let rest = self.write_prefix(sector, offset, data)?;
            (rest, sector.next())
        } else {
            (data, sector)
        };

        let (chunks, last) = data.as_chunks();
        for chunk in chunks {
            self.write_block(sector, &SectorData(*chunk))?;
            sector = sector.next();
        }

        if !last.is_empty() {
            self.write_incomplete(sector, 0, last)?;
        }

        Ok(())
    }

    /// # Errors
    /// failed to write to disk
    pub fn read_data(&mut self, position: u64, data: &mut [u8]) -> Result<usize, VirtIOBlkError> {
        let (sector, offset) = SectorID::parts_from_bytes(position);
        let buf = &mut SectorData::DUMMY.clone();
        self.read_block(sector, buf)?;

        let buf = &buf.0[offset..];
        let n = copy_data(data, buf);
        Ok(n)
    }
}
