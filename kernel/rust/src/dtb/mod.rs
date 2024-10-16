use crate::kalloc::region::Region;
use fdt::Fdt;
use spin::Once;

pub static DTB: Once<(Fdt<'static>, Region)> = Once::new();

/// # Safety
/// ptr must point to 'static valid FDT
/// ptr need not be aligned
#[no_mangle]
unsafe extern "C" fn parse_dtb(ptr: *const u8) {
    DTB.try_call_once(|| {
        // SAFETY: this ptr is valid by precondition
        unsafe { Fdt::from_ptr(ptr) }.map(|fdt| {
            (
                fdt,
                Region::new(ptr as usize, ptr as usize + fdt.total_size()),
            )
        })
    })
    .expect("failed to parse dtb");
}

#[no_mangle]
extern "C" fn init_harts(init_hart: extern "C" fn(usize)) {
    let (fdt, _) = DTB.get().expect("static DTB must be set");

    #[expect(
        clippy::redundant_closure,
        reason = "`init_hart` is `extern \"C\", so not impl FnMut"
    )]
    fdt.cpus()
        .flat_map(|cpu| cpu.ids().all())
        .for_each(|id| init_hart(id));
}
