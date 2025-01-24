#pragma once
#include "kernel/param.h"
#include "kernel/types.h"
#include "kernel/util/spinlock.h"
#include "list.h"

// Saved registers for kernel context switches.
struct context {
  u64 ra;
  u64 sp;

  // callee-saved
  u64 s0;
  u64 s1;
  u64 s2;
  u64 s3;
  u64 s4;
  u64 s5;
  u64 s6;
  u64 s7;
  u64 s8;
  u64 s9;
  u64 s10;
  u64 s11;
};

// Per-CPU state.
struct cpu {
  struct proc*   proc;    // The process running on this cpu, or null.
  struct context context; // swtch() here to enter scheduler().
  int            noff;    // Depth of push_off() nesting.
  int            intena;  // Were interrupts enabled before push_off()?
};

extern struct cpu cpus[NCPU];

// per-process data for the trap handling code in trampoline.S.
// sits in a page by itself just under the trampoline page in the
// user page table. not specially mapped in the kernel page table.
// uservec in trampoline.S saves user registers in the trapframe,
// then initializes registers from the trapframe's
// kernel_sp, kernel_hartid, kernel_satp, and jumps to kernel_trap.
// usertrapret() and userret in trampoline.S set up
// the trapframe's kernel_*, restore user registers from the
// trapframe, switch to the user page table, and enter user space.
// the trapframe includes callee-saved user registers like s0-s11 because the
// return-to-user path via usertrapret() doesn't return through
// the entire kernel call stack.
struct trapframe {
  /*   0 */ u64 kernel_satp;   // kernel page table
  /*   8 */ u64 kernel_sp;     // top of process's kernel stack
  /*  16 */ u64 kernel_trap;   // usertrap()
  /*  24 */ u64 epc;           // saved user program counter
  /*  32 */ u64 kernel_hartid; // saved kernel tp
  /*  40 */ u64 ra;
  /*  48 */ u64 sp;
  /*  56 */ u64 gp;
  /*  64 */ u64 tp;
  /*  72 */ u64 t0;
  /*  80 */ u64 t1;
  /*  88 */ u64 t2;
  /*  96 */ u64 s0;
  /* 104 */ u64 s1;
  /* 112 */ u64 a0;
  /* 120 */ u64 a1;
  /* 128 */ u64 a2;
  /* 136 */ u64 a3;
  /* 144 */ u64 a4;
  /* 152 */ u64 a5;
  /* 160 */ u64 a6;
  /* 168 */ u64 a7;
  /* 176 */ u64 s2;
  /* 184 */ u64 s3;
  /* 192 */ u64 s4;
  /* 200 */ u64 s5;
  /* 208 */ u64 s6;
  /* 216 */ u64 s7;
  /* 224 */ u64 s8;
  /* 232 */ u64 s9;
  /* 240 */ u64 s10;
  /* 248 */ u64 s11;
  /* 256 */ u64 t3;
  /* 264 */ u64 t4;
  /* 272 */ u64 t5;
  /* 280 */ u64 t6;
};

enum procstate {
  UNUSED,
  USED,
  SLEEPING,
  RUNNABLE,
  RUNNING,
  WIP_ZOMBIE,
  ZOMBIE
};

// Per-process state
struct proc {

  struct spinlock lock;

  struct list sched;
  struct list sib;

  // p->lock must be held when using these:
  enum procstate state;  // Process state
  void*          chan;   // If non-zero, sleeping on chan
  int            killed; // If non-zero, have been killed
  int            xstate; // Exit status to be returned to parent's wait
  int            pid;    // Process ID

  // wait_lock must be held when using this:
  struct proc* parent; // Parent process
  struct list  children;

  // these are private to the process, so p->lock need not be held.
  u64               kstack;        // Virtual address of kernel stack
  u64               sz;            // Size of process memory (bytes)
  pagetable_t       pagetable;     // User page table
  struct trapframe* trapframe;     // data page for trampoline.S
  struct context    context;       // swtch() here to run process
  struct file*      ofile[NOFILE]; // Open files
  struct inode*     cwd;           // Current directory
  char              name[16];      // Process name (debugging)
};
