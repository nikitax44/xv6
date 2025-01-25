// Mutual exclusion spin locks.

#include "kernel/util/spinlock.h"
#include "kernel/defs.h"
#include "kernel/hardware/riscv.h"
#include "kernel/scheduler/proc.h"

void initlock(struct spinlock* lk, char* name) {
  lk->name     = name;
  lk->waiters  = (AtomicU32){.value = 0};
  lk->released = (AtomicU32){.value = 0};
  lk->cpu      = 0;
}

extern struct spinlock wait_lock;

// Acquire the lock.
// Loops (spins) until the lock is acquired.
void acquire(struct spinlock* lk) {
  push_off(); // disable interrupts to avoid deadlock.
  if (holding(lk)) {
    panic("acquire");
  }

  u32 tiket = atomic_fetch_and_add(&lk->waiters, 1);

  while (tiket != atomic_load(&lk->released)) {
  }

  // Tell the C compiler and the processor to not move loads or stores
  // past this point, to ensure that the critical section's memory
  // references happen strictly after the lock is acquired.
  // On RISC-V, this emits a fence instruction.
  __sync_synchronize();

  // Record info about lock acquisition for holding() and debugging.
  lk->cpu = mycpu();
}

// Acquire the lock.
// Loops (spins) until the lock is acquired.
bool try_acquire(struct spinlock* lk) {
  push_off(); // disable interrupts to avoid deadlock.
  if (holding(lk)) {
    panic("try_acquire");
  }
  u32 waiters = atomic_load(&lk->waiters);
  // On RISC-V, sync_lock_test_and_set turns into an atomic swap:
  //   a5 = 1
  //   s1 = &lk->locked
  //   amoswap.w.aq a5, a5, (s1)
  if (atomic_compare_exchange(&lk->released, waiters, waiters + 1) == 0) {
    pop_off();
    return false;
  }

  // Tell the C compiler and the processor to not move loads or stores
  // past this point, to ensure that the critical section's memory
  // references happen strictly after the lock is acquired.
  // On RISC-V, this emits a fence instruction.
  __sync_synchronize();

  // Record info about lock acquisition for holding() and debugging.
  lk->cpu = mycpu();
  return true;
}

// Release the lock.
void release(struct spinlock* lk) {
  if (!holding(lk)) {
    panic("release");
  }

  lk->cpu = 0;

  // Tell the C compiler and the CPU to not move loads or stores
  // past this point, to ensure that all the stores in the critical
  // section are visible to other CPUs before the lock is released,
  // and that loads in the critical section occur strictly before
  // the lock is released.
  // On RISC-V, this emits a fence instruction.
  __sync_synchronize();

  // Release the lock, equivalent to lk->locked = 0.
  // This code doesn't use a C assignment, since the C standard
  // implies that an assignment might be implemented with
  // multiple store instructions.
  // On RISC-V, sync_lock_release turns into an atomic swap:
  //   s1 = &lk->locked
  //   amoswap.w zero, zero, (s1)
  atomic_fetch_and_add(&lk->released, 1);

  pop_off();
}

// Check whether this cpu is holding the lock.
// Interrupts must be off.
bool holding(struct spinlock* lk) {
  bool r;
  r = (atomic_load(&lk->waiters) != atomic_load(&lk->released) &&
       lk->cpu == mycpu());
  return r;
}

// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
// are initially off, then push_off, pop_off leaves them off.

void push_off(void) {
  int old = intr_get();

  intr_off();
  if (mycpu()->noff == 0) {
    mycpu()->intena = old;
  }
  mycpu()->noff += 1;
}

void pop_off(void) {
  struct cpu* c = mycpu();
  if (intr_get()) {
    panic("pop_off - interruptible");
  }
  if (c->noff < 1) {
    panic("pop_off");
  }
  c->noff -= 1;
  if (c->noff == 0 && c->intena) {
    intr_on();
  }
}
