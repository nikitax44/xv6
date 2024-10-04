#include "defs.h"
#include "elf.h"
#include "errno.h"
#include "hardware/riscv.h"
#include "param.h"
#include "proc.h"
#include "types.h"
#include <string.h>

static int loadseg(pde_t*, u64, struct inode*, u32, u32);
static int loaddata(pde_t*, u64, struct inode*, u32, u32);
static u64 putargs(pde_t*, str* argv, u64* sp, u64 stackbase, u64* ustack,
                   u64* base);

int flags2perm(int flags) {
  int perm = 0;
  if (flags & 0x1) {
    perm = PTE_X;
  }
  if (flags & 0x2) {
    perm |= PTE_W;
  }
  return perm;
}

int execve(str path, str* argv, str* envp) {
  const char *   s, *last;
  u64            i, off;
  u64            argc, envc, sz = 0, sp, ustack[MAXARG], stackbase, base;
  struct elfhdr  elf;
  struct inode*  ip;
  struct proghdr ph;
  pagetable_t    pagetable = 0, oldpagetable;
  struct proc*   p         = myproc();
  int            ret;

  begin_op();

  if ((ip = namei(path)) == 0) {
    end_op();
    return ENOENT;
  }
  ilock(ip);

  // Check ELF header
  if (readi(ip, 0, (u64)&elf, 0, sizeof(elf)) != sizeof(elf)) {
    ret = ENOEXEC;
    goto bad;
  }

  if (elf.magic != ELF_MAGIC) {
    ret = ENOEXEC;
    goto bad;
  }

  if ((pagetable = proc_pagetable(p)) == 0) {
    ret = -1; // TODO: set valid errno
    goto bad;
  }

  // Load program into memory.
  for (i = 0, off = elf.phoff; i < elf.phnum; i++, off += sizeof(ph)) {
    if (readi(ip, 0, (u64)&ph, off, sizeof(ph)) != sizeof(ph)) {
      ret = ENOEXEC;
      goto bad;
    }
    if (ph.type != ELF_PROG_LOAD) {
      continue;
    }
    if (ph.memsz < ph.filesz) {
      ret = ENOEXEC;
      goto bad;
    }
    if (ph.vaddr + ph.memsz < ph.vaddr) {
      ret = ENOEXEC;
      goto bad;
    }
    u64 sz1;
    if ((sz1 = uvmalloc(pagetable, sz, ph.vaddr + ph.memsz,
                        flags2perm((int)ph.flags))) == 0) {
      ret = ENOMEM;
      goto bad;
    }
    sz = sz1;
    if ((ret = loaddata(pagetable, ph.vaddr, ip, ph.off, ph.filesz)) != 0) {
      goto bad;
    }
  }
  iunlockput(ip);
  end_op();
  ip = 0;

  p         = myproc();
  u64 oldsz = p->sz;

  // Allocate some pages at the next page boundary.
  // Make the first inaccessible as a stack guard.
  // Use the rest as the user stack.
  sz = PGROUNDUP(sz);
  u64 sz1;
  if ((sz1 = uvmalloc(pagetable, sz, sz + (USERSTACK + 1) * PGSIZE, PTE_W)) ==
      0) {
    ret = ENOMEM;
    goto bad;
  }
  sz = sz1;
  uvmclear(pagetable, sz - (USERSTACK + 1) * PGSIZE);
  sp        = sz;
  stackbase = sp - USERSTACK * PGSIZE;

  base = 1;
  sp -= sizeof(u64);

  argc = putargs(pagetable, argv, &sp, stackbase, ustack, &base);
  if (argc == (u64)-1) {
    ret = E2BIG;
    goto bad;
  }

  envc = putargs(pagetable, envp, &sp, stackbase, ustack, &base);
  if (envc == (u64)-1) {
    ret = E2BIG;
    goto bad;
  }

  // push the arrays of argv[] and envp[] pointers.
  sp -= base * sizeof(u64);
  sp -= sp % 16;
  if (sp < stackbase) {
    ret = E2BIG;
    goto bad;
  }

  ustack[0] = argc;

  if (copyout(pagetable, sp, (const u8*)ustack, base * sizeof(u64)) < 0) {
    ret = -1;
    goto bad;
  }

  // arguments to user main(argc, argv, envp)
  // argc is returned via the system call return
  // value, which goes in a0.
  p->trapframe->a1 = sp + sizeof(u64);
  p->trapframe->a2 = sp + (1 + argc + 1) * sizeof(u64);

  // Save program name for debugging.
  for (last = s = path; *s; s++) {
    if (*s == '/') {
      last = s + 1;
    }
  }
  safestrcpy(p->name, last, sizeof(p->name));

  // Commit to the user image.
  oldpagetable      = p->pagetable;
  p->pagetable      = pagetable;
  p->sz             = sz;
  p->trapframe->epc = elf.entry; // initial program counter = main
  p->trapframe->sp  = sp;        // initial stack pointer
  proc_freepagetable(oldpagetable, oldsz);

  return (int)argc; // this ends up in a0, the first argument to main(argc, argv,
               // envp)

bad:
  if (pagetable) {
    proc_freepagetable(pagetable, sz);
  }
  if (ip) {
    iunlockput(ip);
    end_op();
  }
  return ret;
}

// allows unaligned loading
static int loaddata(pagetable_t pagetable, u64 va, struct inode* ip, u32 offset,
                    u32 sz) {
  u64 bt = PGROUNDUP(va);

  u64 diff = bt - va; // >=0
  if (diff != 0) {
    u32 n;
    u64 pa = walkaddr(pagetable, PGROUNDDOWN(va));
    if (pa == 0) {
      panic("loaddata: address should exist");
    }
    if (sz > diff) {
      n = diff;
    } else {
      n = sz;
    }
    if (readi(ip, 0, (u64)pa + (va % PGSIZE), offset, n) != n) {
      return ENOEXEC;
    }
  }

  if (sz > diff) {
    return loadseg(pagetable, bt, ip, offset + diff, sz - diff);
  }
  return 0;
}

// Load a program segment into pagetable at virtual address va.
// va must be page-aligned
// and the pages from va to va+sz must already be mapped.
// Returns 0 on success, -1 on failure.
static int loadseg(pagetable_t pagetable, u64 va, struct inode* ip, u32 offset,
                   u32 sz) {
  u32 i, n;
  u64 pa;

  for (i = 0; i < sz; i += PGSIZE) {
    pa = walkaddr(pagetable, va + i);
    if (pa == 0) {
      panic("loadseg: address should exist");
    }
    if (sz - i < PGSIZE) {
      n = sz - i;
    } else {
      n = PGSIZE;
    }
    if (readi(ip, 0, (u64)pa, offset + i, n) != n) {
      return ENOEXEC;
    }
  }

  return 0;
}

static u64 putargs(pagetable_t pagetable, str* argv, u64* sp, u64 stackbase,
                   u64* ustack, u64* base) {
  u64 argc;
  // Push argument strings, prepare rest of stack in ustack.
  for (argc = 0; argv[argc]; argc++) {
    if (*base + argc >= MAXARG) {
      return -1;
    }
    *sp -= strlen(argv[argc]) + 1;
    *sp -= *sp % 16; // riscv sp must be 16-byte aligned
    if (*sp < stackbase) {
      return -1;
    }
    if (copyout(pagetable, *sp, (const u8*)argv[argc], strlen(argv[argc]) + 1) <
        0) {
      return -1;
    }
    ustack[*base + argc] = *sp;
  }
  ustack[*base + argc] = 0;
  *base += argc + 1;
  return argc;
}
