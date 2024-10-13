#include "kernel/defs.h"
#include "kernel/hardware/riscv.h"
#include "kernel/param.h"
#include "kernel/types.h"

#include "dtb.h"

// main.c
void kernel_main(void);
void init_boot(void);
void init_other(void);

// sbi_entry.S
void _entry_other(u64 hartid);

// implemented below
void        spawn_others(u32 hrts);
static void preallocate_stacks(u32 n);

// sbi_entry.S jumps here in supervisor mode on boot hart.
void start_boot(struct fdt_header* dtb) {
  // enable interrupts
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // needs to be parsed before kvm init. TODO: map me
  u32 hrts = harts(dtb);

  init_boot();

  __sync_synchronize();

  if (hrts > 1) {
    // needs to be run after everything is initialized
    spawn_others(hrts);
  }

  kernel_main();
  panic("main exited on boot hart");
}

void start_other(void) {
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  init_other();

  __sync_synchronize();

  kernel_main();
  panic("main exited");
}

void spawn_others(u32 hrts) {
  struct sbiret result;

  preallocate_stacks(hrts);
  __sync_synchronize();

  for (u32 hartid = 0; hartid < hrts; hartid++) {
    if (hartid == cpuid()) {
      continue;
    }

    result = sbi_hsm_hart_start(hartid, _entry_other, MODE_S);
    if (result.error != 0) {
      printf("failed to start hart %u: %ld\n", hartid, result.error);
    }
  }
}

typedef __attribute__((aligned(PGSIZE))) struct {
  u8 _[PGSIZE];
} page_t;

// sbi_entry.S needs one static stack.
__attribute__((aligned(16))) u8 stack0[PGSIZE];
// and one more stack for each hart except for the boot one.
page_t** other_stack_arr;

static void preallocate_stacks(u32 n) {
  if (n * sizeof(u8*) > PGSIZE) {
    panic("preallocate_stacks");
  }
  page_t** ptr = kalloc();
  for (u32 i = 0; i < n; i++) {
    if (i != r_tp()) {
      ptr[i] = (page_t*)kalloc();
    }
  }
  other_stack_arr = ptr;
}
