#pragma once
#include "atomic.h"
#include "kernel/types.h"

// Mutual exclusion lock.
struct spinlock {
  AtomicU32 waiters;  // Is the lock held?
  AtomicU32 released; // Is the lock held?

  // For debugging:
  const char*       name; // Name of lock.
  const struct cpu* cpu;  // The cpu holding the lock.
};
