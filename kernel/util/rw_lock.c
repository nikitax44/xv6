#include <kernel/defs.h>
#include <kernel/util/rw_lock.h>

void init_rw_lock(struct rw_lock* lk) {
  lk->readers = (AtomicU32){.value = 0};
  lk->writers = (AtomicU32){.value = 0};
  lk->lock    = (AtomicU32){.value = 0};
}

void acquire_read(struct rw_lock* lk) {
  push_off(); // disable interrupts to avoid deadlock.
  while (atomic_lock_test_and_set(&lk->lock, 1) == 1) {
  }
  while (atomic_load(&lk->writers) != 0) {
  }
  atomic_fetch_and_add(&lk->readers, 1);
  atomic_lock_release(&lk->lock);

  // full memory barrier
  __sync_synchronize();
}

void acquire_write(struct rw_lock* lk) {
  push_off(); // disable interrupts to avoid deadlock.
  while (atomic_lock_test_and_set(&lk->lock, 1) == 1) {
  }
  while (atomic_load(&lk->writers) != 0 || atomic_load(&lk->readers) != 0) {
  }
  atomic_fetch_and_add(&lk->writers, 1);
  atomic_lock_release(&lk->lock);

  // full memory barrier
  __sync_synchronize();
}

void release_read(struct rw_lock* lk) {
  if (atomic_load(&lk->writers) != 0) {
    panic("release read");
  }

  atomic_fetch_and_add(&lk->readers, -1);

  __sync_synchronize();
  pop_off();
}

void release_write(struct rw_lock* lk) {
  if (atomic_load(&lk->writers) != 1 || atomic_load(&lk->readers) != 0) {
    panic("release write");
  }

  atomic_fetch_and_add(&lk->writers, -1);

  __sync_synchronize();
  pop_off();
}
