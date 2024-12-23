use crate::hw::hal::HalImpl;
use crate::static_assert;
use crate::util::u64_to_usize;
use efs::dev::error::DevError;
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
#[error("VirtIOBlkError({0})")]
pub struct VirtIOBlkError(virtio_drivers::Error);

impl From<virtio_drivers::Error> for VirtIOBlkError {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl<FSE: core::error::Error> From<VirtIOBlkError> for efs::error::Error<FSE> {
    fn from(value: VirtIOBlkError) -> Self {
        log::error!("{}", value);
        Self::Device(DevError::WriteZero)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SectorID(u64);
static_assert!(blk::SECTOR_SIZE == 512);
#[repr(C, align(512))]
#[derive(IntoBytes, FromBytes, Immutable, Copy, Clone)]
pub struct SectorData(pub [u8; blk::SECTOR_SIZE]);

impl SectorID {
    pub fn from_bytes(bytes: u64) -> Self {
        assert_eq!(
            bytes % SECTOR_SIZE,
            0,
            "byte offset is not SECTOR_SIZE-aligned"
        );
        Self(bytes / SECTOR_SIZE)
    }

    pub const fn in_bytes(self) -> u64 {
        self.0
    }

    pub fn in_sectors(self) -> usize {
        u64_to_usize(self.0)
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl SectorData {
    pub const DUMMY: Self = Self([0x36; blk::SECTOR_SIZE]);
}

impl VirtIOBlk {
    #[allow(clippy::large_stack_frames)]
    pub fn new(transport: MmioTransport) -> Result<Self, VirtIOBlkError> {
        Ok(Self {
            inner: blk::VirtIOBlk::new(transport)?,
        })
    }

    pub fn device_id<'v>(&mut self, id: &'v mut [u8; 20]) -> virtio_drivers::Result<&'v mut [u8]> {
        let sz = self.inner.device_id(id)?;
        Ok(&mut id[..sz])
    }

    pub fn capacity(&self) -> SectorID {
        SectorID(self.inner.capacity())
    }

    pub fn ack_interrupt(&mut self) -> bool {
        self.inner.ack_interrupt()
    }

    pub fn read_block(
        &mut self,
        block_id: SectorID,
        buf: &mut SectorData,
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .read_blocks(u64_to_usize(block_id.0), &mut buf.0)?)
    }

    pub fn read_blocks(
        &mut self,
        start_block_id: SectorID,
        buf: &mut [SectorData],
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .read_blocks(u64_to_usize(start_block_id.0), buf.as_mut_bytes())?)
    }

    pub fn write_block(
        &mut self,
        block_id: SectorID,
        buf: &SectorData,
    ) -> Result<(), VirtIOBlkError> {
        Ok(self.inner.write_blocks(u64_to_usize(block_id.0), &buf.0)?)
    }

    pub fn write_blocks(
        &mut self,
        start_block_id: SectorID,
        buf: &[SectorData],
    ) -> Result<(), VirtIOBlkError> {
        Ok(self
            .inner
            .write_blocks(u64_to_usize(start_block_id.0), buf.as_bytes())?)
    }

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

    pub fn write_data(
        &mut self,
        sector: SectorID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), VirtIOBlkError> {
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
}
