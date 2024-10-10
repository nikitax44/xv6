#include "kernel/defs.h"
#include "kernel/hardware/riscv.h"
#include "kernel/param.h"
#include "kernel/types.h"

#ifdef SBI_ENABLE
#include "dtb.h"

// main.c
void kernel_main(void);
void init_boot(void);
void init_other(void);

// entry.S
void _entry_other(u64 hartid);

// implemented below
void        spawn_others(u32 hrts);
static void preallocate_stacks(u32 n);

// entry.S jumps here in supervisor mode on boot hart.
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
#else
#define WARL_R (1 << 0)
#define WARL_W (1 << 1)
#define WARL_X (1 << 2)

#define WARL_LOCK (1 << 7)

#define WARL_OFF   (0x00 << 3)
#define WARL_TOR   (0x01 << 3)
#define WARL_NA4   (0x10 << 3)
#define WARL_NAPOT (0x11 << 3)

void dispatch(void);
void timerinit(void);

// entry.S needs one stack per CPU.
__attribute__((aligned(16))) char stack0[4096 * NCPU];

// entry.S jumps here in machine mode on stack0.
void start(void) {
  // set M Previous Privilege mode to Supervisor, for mret.
  u64 x = r_mstatus();
  x &= ~MSTATUS_MPP_MASK;
  x |= MSTATUS_MPP_S;
  w_mstatus(x);

  // set M Exception Program Counter to main, for mret.
  // requires gcc -mcmodel=medany
  w_mepc((u64)dispatch);

  // disable paging for now.
  w_satp(0);

  // delegate all interrupts and exceptions to supervisor mode.
  w_medeleg(0xffff);
  w_mideleg(0xffff);
  w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

  // configure Physical Memory Protection to give supervisor mode
  // access to all of physical memory.
  w_pmpaddr0(0x3fffffffffffffull);
  w_pmpcfg0(WARL_TOR | WARL_R | WARL_W | WARL_X);

  // ask for clock interrupts.
  timerinit();

  // switch to supervisor mode and jump to main().
  asm volatile("mret");
}

// ask each hart to generate timer interrupts.
void timerinit(void) {
  // enable supervisor-mode timer interrupts.
  w_mie(r_mie() | MIE_STIE);

  // enable the sstc extension (i.e. stimecmp).
  w_menvcfg(r_menvcfg() | (1L << 63));

  // allow supervisor to use stimecmp and time.
  w_mcounteren(r_mcounteren() | 2);

  // ask for the very first timer interrupt.
  w_stimecmp(r_time() + 1000000);
}

static volatile bool started = 0;

void init_boot(void);
void init_other(void);
void kernel_main(void);

void dispatch(void) {
  if (cpuid() == 0) {
    init_boot();
    __sync_synchronize();
    started = true;
  } else {
    while (!started)
      ;
    __sync_synchronize();
    init_other();
  }

  kernel_main();
}

#endif