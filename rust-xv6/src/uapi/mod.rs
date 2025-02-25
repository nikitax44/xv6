use crate::bindings::pid_t;
use crate::errno::ErrNo;
use crate::kalloc::Xv6Alloc;
use crate::with_lock;
use macro_rules_attribute::apply;

#[macro_export]
macro_rules! syscall {
    () => {};
    (fn $name:ident($($arg:ident : $tp:ident $([$sz:literal])?),* $(,)?) -> i64 $body:block) => {
        #[no_mangle]
        extern "C" fn $name () -> i64 {
            #[allow(unused_mut)]
            let mut argidx = 0;
            $(syscall!(@arg argidx $arg $tp$([$sz])?);)*
            let _ = argidx;

            $body
        }
    };

    (@arg $argidx:ident $arg:ident $tp:ident $([$sz:literal])?) => {
        let $arg = syscall!(@tp $argidx $tp$([$sz])?);
    };

    (@tp $argidx:ident u64) => {{
        // SAFETY: it is save
        let value = unsafe {
            $crate::bindings::argraw($argidx)
        };
        $argidx += 1;
        value
    }};

    (@tp $argidx:ident Proc) => {
        // SAFETY: it is save
        unsafe { $crate::bindings::myproc() }
    };

    (@tp $argidx:ident CStr[$sz:literal]) => {{
        let mut buf = $crate::util::string::InlineCString {
            buffer: [0;$sz],
            size_with_null:0
        };
        // SAFETY: buffer was specified correctly
        let size = unsafe {
            $crate::bindings::argstr($argidx, buf.buffer.as_mut_ptr(), buf.buffer.len() as u64)
        };
        if (size == -1) {
            return -1;
        }
        assert!(size>=0);
        buf.size_with_null=size as usize + 1;

        $argidx += 1;
        buf
    }};
}

#[apply(syscall)]
fn sys_uptime() -> i64 {
    // SAFETY: we acquired the lock required to access it
    with_lock!(crate::bindings::tickslock, || i64::from(unsafe {
        crate::bindings::ticks
    }))
}

#[apply(syscall)]
fn sys_sysinfo(proc: Proc, outaddr: u64) -> i64 {
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
    // SAFETY: it is read-only
    let pagetable = unsafe { (*proc).pagetable };
    // SAFETY: sizeof of sysinfo was passed correctly
    let status = unsafe {
        crate::bindings::copyout(
            pagetable,
            outaddr,
            &raw const sysinfo as *const u8,
            size_of_val(&sysinfo) as u64,
        )
    };

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
    i64::from(unsafe { crate::bindings::kill(pid) })
}

#[apply(syscall)]
fn sys_exit(code: u64) -> i64 {
    let Ok(code) = i32::try_from(code.cast_signed()) else {
        return -ErrNo::EINVAL;
    };
    // SAFETY: it is safe
    unsafe { crate::bindings::exit(code) }
}

#[apply(syscall)]
fn sys_getpid(proc: Proc) -> i64 {
    // SAFETY: it is read-only
    i64::from(unsafe { (*proc).pid })
}

#[apply(syscall)]
fn sys_fork() -> i64 {
    // SAFETY: it is safe
    i64::from(unsafe { crate::bindings::fork() })
}
