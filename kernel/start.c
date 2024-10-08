#include "defs.h"
#include "dtb.h"
#include "param.h"
#include "riscv.h"
#include "types.h"

// main.c
void kernel_main();
void init_boot();
void init_other();

// entry.S
void _entry_other(u64 hartid);

// implemented below
void        spawn_others(u32 hrts);
static void preallocate_stacks(u32 n);

// entry.S jumps here in supervisor mode on boot hart.
void start_boot(struct fdt_header* dtb) {
  // enable interrupts
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // init timer
  sbi_set_timer(r_time() + 1000000);

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

void start_other() {
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  sbi_set_timer(r_time() + 1000000);

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
    result = sbi_hsm_hart_status(hartid);

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

// entry.S needs one static stack.
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
