use crate::kalloc::pages::page::Page;
use crate::memlayout::PGSIZE;
use alloc::format;
use alloc::vec::Vec;
use core::cmp::Reverse;
use fdt::node::MemoryReservation;
use fdt::standard_nodes::MemoryRegion;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct Region {
    start: usize,
    end: usize,
}

impl From<MemoryRegion> for Region {
    fn from(value: MemoryRegion) -> Self {
        let start = value.starting_address as usize;
        Self::new(
            start,
            start + value.size.expect("unbounded region into Region"),
        )
    }
}
impl From<MemoryReservation> for Region {
    fn from(value: MemoryReservation) -> Self {
        let start = value.address() as usize;
        Self::new(start, start + value.size())
    }
}
impl Region {
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "invalid region");
        Self { start, end }
    }

    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }
    #[must_use]
    pub const fn size(&self) -> usize {
        self.end - self.start
    }
    #[must_use]
    pub fn split(self, other: Self) -> Option<(Self, Self)> {
        self.contains(other).then(|| {
            (
                Self::new(self.start, other.start),
                Self::new(other.end, self.end),
            )
        })
    }

    /// # Panics
    /// one or more regions in `reserved` are not contained in `self`
    /// some of the regions in `reserved` intersect with each other
    #[must_use]
    pub fn split_multiple(mut self, mut reserved: Vec<Self>) -> Vec<Self> {
        reserved.sort_unstable_by_key(|reg| reg.start);
        for reg in &mut reserved {
            let (beg, rest) = self
                .split(*reg)
                .ok_or_else(|| {
                    format!(
                        "regions are intersecting or out of bounds: {self:0x?} {:0x?}",
                        *reg
                    )
                })
                .unwrap();
            *reg = beg;
            self = rest;
        }
        reserved.push(self);
        reserved
    }

    /// # Panics
    /// self is misaligned
    pub fn pages(self) -> impl Iterator<Item = *const Page> {
        assert_eq!(
            self.align_shrink(),
            self,
            "misaligned region requests pages"
        );
        (self.start..self.end)
            .step_by(PGSIZE)
            .map(|addr| addr as *const Page)
    }

    #[must_use]
    pub const fn join(self, other: Self) -> Option<(Self, Self)> {
        if self.end > other.start {
            None
        } else {
            Some((
                Self::new(self.start, other.end),
                Self::new(self.end, other.start),
            ))
        }
    }

    /// # Panics
    /// `regions` is empty
    pub fn join_multiple(mut regions: Vec<Self>) -> (Self, Vec<Self>) {
        regions.sort_unstable_by_key(|reg| Reverse(reg.start));
        let mut out = regions.pop().expect("join empty set");
        for i in (0..regions.len()).rev() {
            let val;
            (out, val) = out.join(regions[i]).expect("??");
            regions[i] = val;
        }
        (out, regions)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub const fn align_shrink(self) -> Self {
        Self::new(
            self.start.div_ceil(PGSIZE) * PGSIZE,
            self.end / PGSIZE * PGSIZE,
        )
    }
    pub const fn align_grow(self) -> Self {
        Self::new(
            self.start / PGSIZE * PGSIZE,
            self.end.div_ceil(PGSIZE) * PGSIZE,
        )
    }
}
