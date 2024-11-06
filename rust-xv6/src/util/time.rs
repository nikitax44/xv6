use crate::asm::{cpuid, ticks};
use alloc::vec::Vec;
use core::num::NonZeroU64;
use core::ops::Sub;
use core::time::Duration;
use spin::RwLock;

#[derive(Copy, Clone)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    #[must_use]
    pub fn now() -> Self {
        Self { ticks: ticks() }
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }
}

static FREQ: RwLock<Vec<Option<NonZeroU64>>> = RwLock::new(Vec::new());

#[allow(clippy::large_stack_frames, reason = "no way to deal with it")]
fn get_freq() -> u64 {
    let read = FREQ.upgradeable_read();
    let cpuid = cpuid() as usize;
    if let Some(Some(freq)) = read.get(cpuid) {
        return freq.get();
    }

    let Some((dtb, _)) = crate::dtb::DTB.get() else {
        return 10_000_000;
    };

    let mut write = read.upgrade();
    let cpu = dtb
        .cpus()
        .find(|cpu| cpu.ids().all().any(|id| id == cpuid))
        .expect("invalid cpuid");
    let old_sz = write.len();
    write.resize(usize::max(old_sz, cpuid + 1), None);
    let freq = NonZeroU64::new(cpu.timebase_frequency() as u64).expect("cpu frequency is 0");
    write[cpuid] = Some(freq);
    freq.get()
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let freq = get_freq().try_into().expect("cpu frequency overflows u32");
        Duration::from_secs(self.ticks - rhs.ticks) / freq
    }
}
