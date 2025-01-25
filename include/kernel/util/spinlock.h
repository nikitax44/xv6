#pragma once
#include "atomic.h"
#include "kernel/types.h"

// Mutual exclusion lock.
struct spinlock {
  AtomicU32 waiters;
  AtomicU32 released;

  // For debugging:
  const char*       name; // Name of lock.
  const struct cpu* cpu;  // The cpu holding the lock.
};
