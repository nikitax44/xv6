// Physical memory allocator, for user processes,
// kernel stacks, page-table pages,
// and pipe buffers. Allocates whole 4096-byte pages.

#include "defs.h"
#include "hardware/memlayout.h"
#include "hardware/riscv.h"
#include "param.h"
#include "types.h"
#include "util/spinlock.h"
#include <string.h>

void freerange(void* pa_start, void* pa_end);

extern char end[]; // first address after kernel.
                   // defined by kernel.ld.

struct run {
  struct run* next;
};

struct {
  struct spinlock lock;
  // requires lock
  struct run* freelist;
  // read-only if lock is not taken
  u64 free_pages;
} kmem;

void kinit(void) {
  initlock(&kmem.lock, "kmem");
  kmem.free_pages = 0;
  kmem.freelist   = NULL;
}

void freerange(void* pa_start, void* pa_end) {
  char* p;
  p = (char*)PGROUNDUP((u64)pa_start);
  for (; p + PGSIZE <= (char*)pa_end; p += PGSIZE) {
    kfree(p);
  }
}

extern char _entry[];

// Free the page of physical memory pointed at by pa,
// which normally should have been returned by a
// call to kalloc().  (The exception is when
// initializing the allocator; see kinit above.)
void kfree(void* pa) {
  struct run* r;

  if (((u64)pa % PGSIZE) != 0 || ((char*)pa < end && (char*)pa >= _entry) ||
      (char*)pa < (char*)0x80000000L) {
    panic("kfree");
  }

  // Fill with junk to catch dangling refs.
  memset(pa, 1, PGSIZE);

  r = (struct run*)pa;

  acquire(&kmem.lock);
  r->next       = kmem.freelist;
  kmem.freelist = r;
  kmem.free_pages++;
  release(&kmem.lock);
}

// Allocate one 4096-byte page of physical memory.
// Returns a pointer that the kernel can use.
// Returns 0 if the memory cannot be allocated.
void* kalloc(void) {
  struct run* r;

  acquire(&kmem.lock);
  r = kmem.freelist;
  if (r) {
    kmem.freelist = r->next;
    kmem.free_pages--;
  }
  release(&kmem.lock);

  if (r) {
    memset((char*)r, 0, PGSIZE); // fill with zeroes
  }
  return (void*)r;
}

// free memory size
u64 free_pages(void) { return kmem.free_pages; }
