use crate::bindings::{
    exit, fork, growproc, kill, killed, pid_t, sleep, ticks, tickslock, timeval, wait,
};
use crate::errno::ErrNo;
use crate::kalloc::Xv6Alloc;
use crate::uapi::copyout;
use crate::{syscall, with_lock};
use macro_rules_attribute::apply;

#[apply(syscall)]
fn sys_uptime() -> i64 {
    // SAFETY: we acquired the lock required to access it
    with_lock!(tickslock, || i64::from(unsafe { ticks }))
}

#[apply(syscall)]
fn sys_sysinfo(outaddr: u64) -> i64 {
    let meminfo = crate::kalloc::get_kalloc().get_info().into();
    let sysinfo = crate::bindings::sysinfo {
        uptime: sys_uptime(),
        loads: [0, 0, 0],
        totalram: meminfo.total_bytes as u64,
        freeram: meminfo.free_bytes as u64,
        sharedram: 0,
        bufferram: 0,
        totalswap: 0,
        freeswap: 0,
        procs: 1,
        totalhigh: 0,
        freehigh: 0,
        mem_unit: 1,
    };
    let status = copyout(outaddr, &sysinfo);

    if status != 0 {
        -ErrNo::EFAULT
    } else {
        0
    }
}

#[apply(syscall)]
fn sys_futimesat() -> i64 {
    -ErrNo::ENOSYS
}

#[apply(syscall)]
fn sys_kill(pid: u64) -> i64 {
    let Ok(pid) = pid_t::try_from(pid) else {
        return -ErrNo::EINVAL;
    };

    // SAFETY: it is safe
    i64::from(unsafe { kill(pid) })
}

#[apply(syscall)]
fn sys_exit(code: u64) -> i64 {
    let Ok(code) = i32::try_from(code.cast_signed()) else {
        return -ErrNo::EINVAL;
    };
    // SAFETY: it is safe
    unsafe { exit(code) }
}

#[apply(syscall)]
fn sys_getpid(proc: Proc) -> i64 {
    // SAFETY: it is read-only
    i64::from(unsafe { (*proc).pid })
}

#[apply(syscall)]
fn sys_fork() -> i64 {
    // SAFETY: it is safe
    i64::from(unsafe { fork() })
}

#[apply(syscall)]
fn sys_wait(addr: u64) -> i64 {
    // SAFETY: safe
    i64::from(unsafe { wait(addr) })
}

#[apply(syscall)]
fn sys_sbrk(proc: Proc, n: u64) -> i64 {
    // SAFETY: may be read only by us
    let prev_size = unsafe { (*proc).sz };
    // SAFETY: safe
    if unsafe { growproc(n.cast_signed()) } < 0 {
        return -1;
    }
    prev_size.cast_signed()
}

#[apply(syscall)]
fn sys_sleep(proc: Proc, n: u64) -> i64 {
    let Ok(n) = u32::try_from(n) else {
        return -ErrNo::EINVAL;
    };
    with_lock!(tickslock, || {
        // SAFETY: tickslick is acquired
        let ticks0 = unsafe { ticks };
        // SAFETY: tickslick is acquired
        while unsafe { ticks } - ticks0 < n {
            // SAFETY: proc is valid ptr
            if unsafe { killed(proc) } != 0 {
                return -ErrNo::ECANCELED;
            }

            // NOTE: rust plugin highlights this as an error, but it is, in fact, not.
            let ticks_ptr = &raw mut ticks as *mut _;
            let lock_ptr = &raw mut tickslock;

            // SAFETY: lock is acquired
            unsafe {
                sleep(ticks_ptr, lock_ptr);
            }
        }

        0
    })
}

#[apply(syscall)]
fn sys_gettimeofday(outaddr: u64, _tzinfo: u64) -> i64 {
    let time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if copyout(outaddr, &time) < 0 {
        return -ErrNo::EFAULT;
    }
    0
}
