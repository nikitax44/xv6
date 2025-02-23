#include "kernel/scheduler/proc.h"
#include "kernel/defs.h"
#include "kernel/errno.h"
#include "kernel/hardware/memlayout.h"
#include "kernel/hardware/riscv.h"
#include "kernel/initcode.h"

struct cpu cpus[NCPU];

struct proc* initproc;

// list for procces that may has arbitrary state, mostly it has runnable state
struct list sentinel_sched;

// list for procces that has state different from runnable
// if process switch your status to runnable it must add yourself to
// sentinel_sched list and delete yourself from current list
struct list sentinel_other;

pid_t                                        nextpid = 1;
struct spinlock __attribute__((aligned(64))) pid_lock;

extern void forkret(void);
static void freeproc(struct proc* p);
void        wakeup_base(void*, int);
void        wakeup_process(struct proc*);

extern char trampoline[]; // trampoline.S

// We must acquire this lock if we want to iterate sentinel_sched list or
// sentinel_other list. Must be acquired before any p->lock.
struct spinlock __attribute__((aligned(64))) wait_lock;

// We must take this lock if we want to use children list or p->parent of some
// process. Must be acquired before wait_lock and any p->lock.
struct spinlock __attribute__((aligned(64))) parent_lock;

// wait_lock must be aquired
// Iterate all proccesses in sentinel_sched list, and sentinel_other list.
// It returns &sentinel_other if iter wal last element
struct list* iterate_next(struct list* iter) {
  if (iter->next == &sentinel_sched) {
    return sentinel_other.next;
  } else {
    return iter->next;
  }
}

// wait_lock must be aquired.
// Used to get first process in merged list
struct list* iterate_begin(void) {
  if (lst_empty(&sentinel_sched)) {
    return sentinel_other.next;
  }
  return sentinel_sched.next;
}

// Initialize the proc table.
void procinit(void) {
  lst_init(&sentinel_sched);
  lst_init(&sentinel_other);

  initlock(&pid_lock, "nextpid");
  initlock(&wait_lock, "wait_lock");
  initlock(&parent_lock, "parent_lock");
}

// Must be called with interrupts disabled,
// to prevent race with process being moved
// to a different CPU.
u32 cpuid(void) {
  u64 id = r_tp();
  return id;
}

// Return this CPU's cpu struct.
// Interrupts must be disabled.
struct cpu* mycpu(void) {
  u32         id = cpuid();
  struct cpu* c  = &cpus[id];
  return c;
}

// Return the current struct proc *, or zero if none.
struct proc* myproc(void) {
  push_off();
  struct cpu*  c = mycpu();
  struct proc* p = c->proc;
  pop_off();
  return p;
}

pid_t allocpid(void) {
  pid_t pid;

  acquire(&pid_lock);
  pid     = nextpid;
  nextpid = nextpid + 1;
  release(&pid_lock);

  return pid;
}

// Initialize process with state required to run in the kernel,
// and return with p->lock held.
// If memory allocation fails, return 0.
static int allocproc(struct proc** proc_out) {
  struct proc* p = kalloc();
  if (p == 0) {
    return ENOMEM;
  }
  memset(p, 0, sizeof(struct proc));
  lst_init(&p->sched);
  lst_init(&p->children);
  lst_init(&p->sib);

  // Allocate a trapframe page.
  if ((p->trapframe = (struct trapframe*)kalloc()) == 0) {
    freeproc(p);
    return ENOMEM;
  }
  memset(p->trapframe, 0, sizeof(struct trapframe));

  // An empty user page table.
  p->pagetable = proc_pagetable(p);
  if (p->pagetable == 0) {
    freeproc(p);
    return ENOMEM;
  }

  p->kstack = request_stack();
  if (p->kstack == NULL) {
    freeproc(p);
    return ENOMEM;
  }

  initlock(&p->lock, "proc");
  p->pid   = allocpid();
  p->state = USED;

  // Set up new context to start executing at forkret,
  // which returns to user space.
  p->context.ra = (u64)forkret;
  p->context.sp = (u64)p->kstack;

  acquire(&wait_lock);
  acquire(&p->lock);
  lst_push(sentinel_other.prev, &p->sched);
  release(&wait_lock);
  *proc_out = p;
  return 0;
}

// Free a proc structure and the data hanging from it,
// including user pages. Unmap stack.
// p->lock must be held.
static void freeproc(struct proc* p) {
  if (p->trapframe) {
    kfree((void*)p->trapframe);
  }
  if (p->pagetable) {
    proc_freepagetable(p->pagetable, p->sz);
  }
  if (p->kstack) {
    sfence_vma_address((u64)p->kstack - PGSIZE);
    release_stack(p->kstack);
  }
  if (!lst_empty(&p->children)) {
    panic("free children");
  }
  kfree(p);
}

// Create a user page table for a given process, with no user memory,
// but with trampoline and trapframe pages.
pagetable_t proc_pagetable(struct proc* p) {
  pagetable_t pagetable;

  // An empty page table.
  pagetable = uvmcreate();
  if (pagetable == 0) {
    return 0;
  }

  // map the trampoline code (for system call return)
  // at the highest user virtual address.
  // only the supervisor uses it, on the way
  // to/from user space, so not PTE_U.
  if (mappages(pagetable, TRAMPOLINE, PGSIZE, (u64)trampoline, PTE_R | PTE_X) !=
      0) {
    uvmfree(pagetable, 0);
    return 0;
  }

  // map the trapframe page just below the trampoline page, for
  // trampoline.S.
  if (mappages(pagetable, TRAPFRAME, PGSIZE, (u64)(p->trapframe),
               PTE_R | PTE_W) != 0) {
    uvmunmap(pagetable, TRAMPOLINE, 1, 0);
    uvmfree(pagetable, 0);
    return 0;
  }

  return pagetable;
}

// Free a process's page table, and free the
// physical memory it refers to.
void proc_freepagetable(pagetable_t pagetable, u64 sz) {
  uvmunmap(pagetable, TRAMPOLINE, 1, 0);
  uvmunmap(pagetable, TRAPFRAME, 1, 0);
  uvmfree(pagetable, sz);
}

// Set up first user process.
void userinit(void) {
  struct proc* p;

  allocproc(&p);
  initproc = p;

  // allocate one user page and copy initcode's instructions
  // and data into it.
  uvmfirst(p->pagetable, initcode_start, initcode_end - initcode_start);
  p->sz = PGSIZE;

  // prepare for the very first "return" from kernel to user.
  p->trapframe->epc = 0;      // user program counter
  p->trapframe->sp  = PGSIZE; // user stack pointer

  safestrcpy(p->name, "initcode", sizeof(p->name));
  p->cwd = namei("/");
  release(&p->lock);
  acquire(&wait_lock);
  acquire(&p->lock);
  p->state = RUNNABLE;
  lst_remove(&p->sched);
  lst_push(sentinel_sched.prev, &p->sched);
  release(&wait_lock);

  release(&p->lock);
}

// Grow or shrink user memory by n bytes.
// Return 0 on success, -1 on failure.
int growproc(int n) {
  u64          sz;
  struct proc* p = myproc();

  sz = p->sz;
  if (n > 0) {
    if ((sz = uvmalloc(p->pagetable, sz, sz + n, PTE_W)) == 0) {
      return -1;
    }
  } else if (n < 0) {
    sz = uvmdealloc(p->pagetable, sz, sz + n);
  }
  p->sz = sz;
  return 0;
}

// Create a new process, copying the parent.
// Sets up child kernel stack to return as if from fork() system call.
pid_t fork(void) {
  int          i;
  pid_t        pid;
  struct proc* np;
  struct proc* p   = myproc();
  int          res = 0;

  // Allocate process.
  if ((res = allocproc(&np)) != 0) {
    return -res;
  }

  // Copy user memory from parent to child.
  if ((res = uvmcopy(p->pagetable, np->pagetable, p->sz)) != 0) {
    release(&np->lock);
    acquire(&wait_lock);
    lst_remove(&np->sched);
    freeproc(np);
    release(&wait_lock);
    return -res;
  }
  np->sz = p->sz;

  // copy saved user registers.
  *(np->trapframe) = *(p->trapframe);

  // Cause fork to return 0 in the child.
  np->trapframe->a0 = 0;

  // increment reference counts on open file descriptors.
  for (i = 0; i < NOFILE; i++) {
    if (p->ofile[i]) {
      np->ofile[i] = filedup(p->ofile[i]);
    }
  }
  np->cwd = idup(p->cwd);

  safestrcpy(np->name, p->name, sizeof(p->name));

  pid = np->pid;

  release(&np->lock);

  acquire(&parent_lock);
  np->parent = p;
  lst_push(&p->children, &np->sib);
  release(&parent_lock);

  acquire(&wait_lock);
  acquire(&np->lock);
  np->state = RUNNABLE;
  lst_remove(&np->sched);
  lst_push(sentinel_sched.prev, &np->sched);

  release(&np->lock);
  release(&wait_lock);

  return pid;
}

// Pass p's abandoned children to init.
void reparent(struct proc* p) {
  struct proc* pp;
  struct list* sib;

  acquire(&parent_lock);
  for (sib = p->children.next; sib != &p->children; sib = sib->next) {
    pp = (struct proc*)((char*)sib - offsetof(struct proc, sib));
    if (pp->parent != p) {
      panic("reparent: wrong parent");
    }
    pp->parent = initproc;
  }
  lst_extend_move(&initproc->children, &p->children);
  release(&parent_lock);
  acquire(&wait_lock);
  wakeup_process(initproc);
  release(&wait_lock);
}

// Mark the current process as exited. *Does* return.
// the usertrap must be in the caller chain
// An exited process remains in the zombie state
// until its parent calls wait().
void mark_exit(int status) {
  struct proc* p = myproc();

  if (p == initproc) {
    panic("init exiting");
  }

  // Close all open files.
  for (int fd = 0; fd < NOFILE; fd++) {
    if (p->ofile[fd]) {
      struct file* f = p->ofile[fd];
      fileclose(f);
      p->ofile[fd] = 0;
    }
  }

  begin_op();
  iput(p->cwd);
  end_op();
  p->cwd = 0;

  // Give any children to init.
  reparent(p);

  acquire(&p->lock);
  p->xstate = status;
  p->state  = WIP_ZOMBIE;
  release(&p->lock);
}

// Wait for a child process to exit and return its pid.
// Return -1 if this process has no children.
int wait(u64 addr) {
  struct proc* pp;
  int          pid;
  struct proc* p = myproc();
  struct list* sib;

  acquire(&parent_lock);

  for (;;) {
    // No point waiting if we don't have any children.
    if (lst_empty(&p->children)) {
      release(&parent_lock);
      return -1;
    }

    // Scan through table looking for exited children.
    for (sib = p->children.next; sib != &p->children; sib = sib->next) {
      pp = (struct proc*)((char*)sib - offsetof(struct proc, sib));
      if (pp->parent != p) {
        panic("wait: wrong parent");
      }

      // make sure the child isn't still in exit() or swtch().
      acquire(&pp->lock);

      if (pp->state == ZOMBIE) {
        // Found one.
        pid = pp->pid;

        if (!lst_empty(&pp->children)) {
          panic("stale ZOMBIE's children");
        }

        if (addr != 0 && copyout(p->pagetable, addr, (const u8*)&pp->xstate,
                                 sizeof(pp->xstate)) < 0) {
          release(&pp->lock);
          release(&parent_lock);
          return -1;
        }

        release(&pp->lock);
        acquire(&wait_lock);
        acquire(&pp->lock);
        // cut zombie from children list
        lst_remove(&pp->sib);

        lst_remove(&pp->sched);
        release(&pp->lock);
        freeproc(pp);
        release(&wait_lock);
        release(&parent_lock);
        return pid;
      }
      release(&pp->lock);
    }

    if (killed(p)) {
      release(&parent_lock);
      return -1;
    }

    // Wait for a child to exit.
    sleep(p, &parent_lock); // DOC: wait-sleep
  }
}

u64 max_shead_cycles = 100;

// Per-CPU process scheduler.
// Each CPU calls scheduler() after setting itself up.
// Scheduler never returns.  It loops, doing:
//  - choose a process to run.
//  - swtch to start running that process.
//  - eventually that process transfers control
//    via swtch back to the scheduler.
void scheduler(void) {
  struct proc* p;
  struct cpu*  c = mycpu();
  struct list* lst_ptr;

  c->proc     = 0;
  u64 count   = 0;
  int has_job = 0;
  for (;;) {
    count++;
    if (count == max_shead_cycles) {
      if (!has_job) {
        intr_on();
        asm volatile("wfi");
      }
      count   = 0;
      has_job = 0;
    }
    // The most recent process to run may have had interrupts
    // turned off; enable them to avoid a deadlock if all
    // processes are waiting.
    intr_on();
    acquire(&wait_lock);
    if (sentinel_sched.next == &sentinel_sched) {
      release(&wait_lock);
      continue;
    }
    lst_ptr = sentinel_sched.next;
    lst_remove(lst_ptr);
    lst_push(sentinel_other.prev, lst_ptr);
    p = (struct proc*)((char*)lst_ptr - offsetof(struct proc, sched));
    acquire(&p->lock);
    if (p->state != RUNNABLE) {
      release(&p->lock);
      release(&wait_lock);
      continue;
    }
    release(&wait_lock);
    has_job = 1;
    //  Switch to chosen process.  It is the process's job
    //  to release its lock and then reacquire it
    //  before jumping back to us.
    p->state = RUNNING;
    c->proc  = p;
    // flush TLB for kstack
    sfence_vma_address((u64)p->kstack - PGSIZE);
    swtch(&c->context, &p->context);

    // Process is done running for now.
    // It should have changed its p->state before coming back.
    c->proc = 0;
    release(&p->lock);
  }
}

// Switch to scheduler.  Must hold only p->lock
// and have changed proc->state. Saves and restores
// intena because intena is a property of this
// kernel thread, not this CPU. It should
// be proc->intena and proc->noff, but that would
// break in the few places where a lock is held but
// there's no process.
void sched(void) {
  int          intena;
  struct proc* p = myproc();

  if (!holding(&p->lock)) {
    panic("sched p->lock");
  }
  if (mycpu()->noff != 1) {
    panic("sched locks");
  }
  if (p->state == RUNNING) {
    panic("sched running");
  }
  if (intr_get()) {
    panic("sched interruptible");
  }

  intena = mycpu()->intena;
  swtch(&p->context, &mycpu()->context);
  mycpu()->intena = intena;
}

// Give up the CPU for one scheduling round.
void yield(void) {
  struct proc* p = myproc();
  acquire(&wait_lock);
  acquire(&p->lock);
  p->state = RUNNABLE;
  lst_remove(&p->sched);
  lst_push(sentinel_sched.prev, &p->sched);
  release(&wait_lock);
  sched();
  release(&p->lock);
}

// A fork child's very first scheduling by scheduler()
// will swtch to forkret.
void forkret(void) {
  static int first = 1;

  // Still holding p->lock from scheduler.
  release(&myproc()->lock);

  if (first) {
    // File system initialization must be run in the context of a
    // regular process (e.g., because it calls sleep), and thus cannot
    // be run from main().
    fsinit(ROOTDEV);

    first = 0;
    // ensure other cores see first=0.
    __sync_synchronize();
  }

  usertrapret();
}

// Atomically release lock and sleep on chan.
// Reacquires lock when awakened.
void sleep(void* chan, struct spinlock* lk) {
  struct proc* p = myproc();

  // Must acquire p->lock in order to
  // change p->state and then call sched.
  // Once we hold p->lock, we can be
  // guaranteed that we won't miss any wakeup
  // (wakeup locks p->lock),
  // so it's okay to release lk.
  acquire(&p->lock); // DOC: sleeplock1
  release(lk);
  // Go to sleep.
  p->chan  = chan;
  p->state = SLEEPING;

  sched();

  // Tidy up.
  p->chan = 0;

  // Reacquire original lock.
  release(&p->lock);
  acquire(lk);
}

// Wake up all processes sleeping on chan.
// Must be called without any p->lock.
// has_wait_lock = 0 if don't have wait_lock
void wakeup_base(void* chan, int has_wait_lock) {
  struct proc* p;
  struct list* it;
  if (!has_wait_lock) {
    acquire(&wait_lock);
  }
  for (it = iterate_begin(); it != &sentinel_other; it = iterate_next(it)) {
    p = (struct proc*)((char*)it - offsetof(struct proc, sched));
    if (p != myproc()) {
      acquire(&p->lock);
      if (p->state == SLEEPING && p->chan == chan) {
        p->state = RUNNABLE;
        lst_remove(&p->sched);
        lst_push(sentinel_sched.prev, &p->sched);
      }
      release(&p->lock);
    }
  }
  if (!has_wait_lock) {
    release(&wait_lock);
  }
}

// wakeup one given process
// user must acquire p->lock and wait_lock
void wakeup_process(struct proc* p) {
  acquire(&p->lock);
  if (p->state == SLEEPING) {
    p->state = RUNNABLE;
    lst_remove(&p->sched);
    lst_push(sentinel_sched.prev, &p->sched);
  }
  release(&p->lock);
}

// Wake up all processes sleeping on chan.
// Must be called without any p->lock.
void wakeup(void* chan) { wakeup_base(chan, 0); }

// Kill the process with the given pid.
// The victim won't exit until it tries to return
// to user space (see usertrap() in trap.c).
int kill(pid_t pid) {
  struct proc* p;
  struct list* it;

  acquire(&wait_lock);
  for (it = iterate_begin(); it != &sentinel_other; it = iterate_next(it)) {
    p = (struct proc*)((char*)it - offsetof(struct proc, sched));
    acquire(&p->lock);
    if (p->pid == pid) {
      p->killed = 1;
      if (p->state == SLEEPING) {
        // Wake process from sleep().
        p->state = RUNNABLE;
        lst_remove(&p->sched);
        lst_push(sentinel_sched.prev, &p->sched);
      }
      release(&p->lock);
      release(&wait_lock);
      return 0;
    }
    release(&p->lock);
  }
  release(&wait_lock);
  return -1;
}

// Kill all the user processed except for init
void kill_all(void) {
  struct proc* p;
  struct list* it;

  acquire(&wait_lock);
  for (it = iterate_begin(); it != &sentinel_other; it = iterate_next(it)) {
    p = (struct proc*)((char*)it - offsetof(struct proc, sched));
    acquire(&p->lock);
    if (p->pid > 1) {
      p->killed = 1;
      if (p->state == SLEEPING) {
        // Wake process from sleep().
        p->state = RUNNABLE;
        lst_remove(&p->sched);
        lst_push(sentinel_sched.prev, &p->sched);
      }
    }
    release(&p->lock);
  }
  release(&wait_lock);
}

void setkilled(struct proc* p) {
  acquire(&p->lock);
  p->killed = 1;
  release(&p->lock);
}

int killed(struct proc* p) {
  int k;

  acquire(&p->lock);
  k = p->killed;
  release(&p->lock);
  return k;
}

void do_exit_if_needed(struct proc* p) {
  acquire((&parent_lock));
  acquire(&wait_lock);
  acquire(&p->lock);
  if (p->state == WIP_ZOMBIE) {
    p->state = ZOMBIE;

    // Parent might be sleeping in wait().
    wakeup_process(p->parent);

    release(&wait_lock);
    release(&parent_lock);

    // Jump into the scheduler, never to return.
    sched();
    panic("zombie exit");
  }
  release(&p->lock);
  release(&wait_lock);
  release(&parent_lock);
}

// Copy to either a user address, or kernel address,
// depending on usr_dst.
// Returns 0 on success, -1 on error.
int either_copyout(int user_dst, u64 dst, void* src, u64 len) {
  struct proc* p = myproc();
  if (user_dst) {
    return copyout(p->pagetable, dst, src, len);
  } else {
    memmove((char*)dst, src, len);
    return 0;
  }
}

// Copy from either a user address, or kernel address,
// depending on usr_src.
// Returns 0 on success, -1 on error.
int either_copyin(void* dst, int user_src, u64 src, u64 len) {
  struct proc* p = myproc();
  if (user_src) {
    return copyin(p->pagetable, dst, src, len);
  } else {
    memmove(dst, (char*)src, len);
    return 0;
  }
}

// Print a process listing to console.  For debugging.
// Runs when user types ^P on console.
// No lock to avoid wedging a stuck machine further.
void procdump(void) {
  static char* states[] = {
      [UNUSED] = "unused",    [USED] = "used",      [SLEEPING] = "sleep ",
      [RUNNABLE] = "runble",  [RUNNING] = "run   ", [ZOMBIE] = "zombie",
      [WIP_ZOMBIE] = "dying "};
  struct proc* p;
  char*        state;
  struct list* it;

  printf("\n");
  acquire(&wait_lock);
  for (it = iterate_begin(); it != &sentinel_other; it = iterate_next(it)) {
    p = (struct proc*)((char*)it - offsetof(struct proc, sched));
    if (p->state == UNUSED) {
      continue;
    }
    if (p->state >= 0 && p->state < NELEM(states) && states[p->state]) {
      state = states[p->state];
    } else {
      state = "???";
    }
    printf("%d %s %s", p->pid, state, p->name);
    printf("\n");
  }
  release(&wait_lock);
}
