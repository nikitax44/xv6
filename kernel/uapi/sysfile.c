//
// File-system system calls.
// Mostly argument checking, since we don't trust
// user code, and filesystem calls.
//

#include "kernel/defs.h"
#include "kernel/fcntl.h"
#include "kernel/file/stat.h"
#include "kernel/mman.h"
#include "kernel/param.h"
#include "kernel/proc.h"

// Fetch the nth word-sized system call argument as a file descriptor
// and return both the descriptor and the corresponding struct file.
static int argfd(int n, int* pfd, struct file** pf) {
  int          fd;
  struct file* f;

  argint(n, &fd);
  if (fd < 0 || fd >= NOFILE || (f = myproc()->ofile[fd]) == 0) {
    return -1;
  }
  if (pfd) {
    *pfd = fd;
  }
  if (pf) {
    *pf = f;
  }
  return 0;
}

// Allocate a file descriptor for the given file.
// Takes over file reference from caller on success.
static int fdalloc(struct file* f) {
  int          fd;
  struct proc* p = myproc();

  for (fd = 0; fd < NOFILE; fd++) {
    if (p->ofile[fd] == 0) {
      p->ofile[fd] = f;
      return fd;
    }
  }
  return -1;
}

u64 sys_dup(void) {
  struct file* f;
  int          fd;

  if (argfd(0, 0, &f) < 0) {
    return -1;
  }
  if ((fd = fdalloc(f)) < 0) {
    return -1;
  }
  rs_file_dup(f);
  return fd;
}

u64 sys_read(void) {
  struct file* f;
  int          n;
  u64          p;
  u8           buf[512];

  argaddr(1, &p);
  argint(2, &n);
  if (argfd(0, 0, &f) < 0) {
    return -1;
  }

  usize r = rs_file_read(f, buf, MIN((usize)n, sizeof(buf)));
  copyout(myproc()->pagetable, p, buf, r);

  return r;
}

u64 sys_seek(void) {
  struct file* f;
  u64          offset;
  int          whence;

  argaddr(1, &offset);
  argint(2, &whence);
  if (argfd(0, 0, &f) < 0) {
    return -1;
  }
  return rs_file_seek(f, (i64)offset, (WHENCE)whence);
}

u64 sys_write(void) {
  struct file* f;
  int          n;
  u64          p;
  char         buf[512];

  argaddr(1, &p);
  argint(2, &n);
  if (argfd(0, 0, &f) < 0) {
    return -1;
  }

  int r = copyin(myproc()->pagetable, buf, p, MIN((usize)n, sizeof(buf)));

  usize o = rs_file_write(f, (u8*)buf, r);

  return o;
}

u64 sys_close(void) {
  int          fd;
  struct file* f;

  if (argfd(0, &fd, &f) < 0) {
    return -1;
  }
  myproc()->ofile[fd] = 0;
  rs_file_close(f);
  return 0;
}

u64 sys_fstat(void) {
  struct file* f;
  u64          st; // user pointer to struct stat

  argaddr(1, &st);
  if (argfd(0, 0, &f) < 0) {
    return -1;
  }
  struct stat stat = rs_file_stat(f);

  if (copyout(myproc()->pagetable, st, (const u8*)&stat, sizeof(stat)) < 0) {
    return -1;
  }
  return 0;
}

// Create the path new as a link to the same inode as old.
u64 sys_link(void) {
  char new[MAXPATH], old[MAXPATH];

  if (argstr(0, old, MAXPATH) < 0 || argstr(1, new, MAXPATH) < 0) {
    return -1;
  }

  return -rs_fs_hard_link(old, new);
}

u64 sys_unlink(void) {
  char path[MAXPATH];

  if (argstr(0, path, MAXPATH) < 0) {
    return -1;
  }

  return -rs_fs_unlink(path);
}

u64 sys_open(void) {
  char         path[MAXPATH];
  int          fd, omode;
  struct file* f;

  argint(1, &omode);
  if (argstr(0, path, MAXPATH) < 0) {
    return -1;
  }

  if (omode & O_CREATE) {
    rs_fs_create(path);
  }
  if ((f = rs_file_open(path)) == 0) {
    return -1;
  }

  fd = fdalloc(f);
  if (fd < 0) {
    if (f) {
      rs_file_close(f);
    }
    return -1;
  }

  return fd;
}

u64 sys_mkdir(void) {
  char path[MAXPATH];

  if (argstr(0, path, MAXPATH) < 0) {
    return -1;
  }

  return -rs_fs_mkdir(path);
}

u64 sys_mknod(void) {
  char path[MAXPATH];
  int  major, minor;

  argint(1, &major);
  argint(2, &minor);
  if (argstr(0, path, MAXPATH) < 0) {
    return -1;
  }
  return -rs_fs_mknod(path, major, minor);
}

u64 sys_chdir(void) {
  struct proc* p = myproc();

  if (argstr(0, p->cwd, MAXPATH) < 0) {
    return -1;
  }
  return 0;
}

static u64 fetchargs(u64 uargv, char* (*argv)[MAXARG]) {
  u64 i, uarg;
  memset(argv, 0, sizeof((argv[0])));
  if (uargv == 0) {
    (*argv)[0] = 0;
    return 0;
  }
  for (i = 0;; i++) {
    if (i >= NELEM((*argv))) {
      return E2BIG;
    }
    if (fetchaddr(uargv + sizeof(u64) * i, &uarg) < 0) {
      return EFAULT;
    }
    if (uarg == 0) {
      (*argv)[i] = 0;
      break;
    }
    (*argv)[i] = kalloc();
    if ((*argv)[i] == 0) {
      return ENOMEM;
    }
    if (fetchstr(uarg, (*argv)[i], PGSIZE) < 0) {
      return E2BIG;
    }
  }
  return 0;
}

u64 sys_execve(void) {
  char path[MAXPATH], *argv[MAXARG], *envp[MAXARG];
  u64  i;
  u64  uargv, uenvp;
  int  err;

  argaddr(1, &uargv);
  argaddr(2, &uenvp);
  if (argstr(0, path, MAXPATH) < 0) {
    return E2BIG;
  }

  err = fetchargs(uargv, &argv);
  if (err != 0) {
    envp[0] = 0;
    goto end;
  }

  err = fetchargs(uenvp, &envp);
  if (err != 0) {
    goto end;
  }

  err = execve(path, (str*)argv, (str*)envp);

end:
  for (i = 0; i < NELEM(argv) && argv[i] != 0; i++) {
    kfree(argv[i]);
  }

  for (i = 0; i < NELEM(envp) && envp[i] != 0; i++) {
    kfree(envp[i]);
  }

  return err;
}

u64 sys_mmap(void) {
  u64 va, sz, size, offset;
  int prot, flags, fd = -1;
  argaddr(0, &va);
  argaddr(1, &size);
  argint(2, &prot);
  argint(3, &flags);
  argfd(4, &fd, 0);
  argaddr(1, &offset);

  printf("va=0x%lx size=0x%lx offset=0x%lx prot=0x%x flags=0x%x fd=%d\n", va,
         size, offset, prot, flags, fd);
  if (flags != (MAP_ANON | MAP_PRIVATE)) {
    return MAP_FAILED_EADDR;
  }
  if (prot != (PROT_READ | PROT_WRITE)) {
    return MAP_FAILED_EADDR;
  }
  // so MAP_FIXED is not set

  struct proc* proc = myproc();

  sz = va = proc->sz;
  if (va + size < va) {
    return MAP_FAILED_EADDR;
  }

  if ((sz = uvmalloc(proc->pagetable, sz, sz + size, PTE_W)) == 0) {
    return -1;
  }
  proc->sz = sz;

  return va;
}

u64 sys_pipe(void) {
  u64          fdarray; // user pointer to array of two integers
  struct file *rf, *wf;
  int          fd0, fd1;
  struct proc* p = myproc();

  argaddr(0, &fdarray);
  if (rs_pipe_alloc(&rf, &wf) != 0) {
    return -1;
  }
  fd0 = -1;
  if ((fd0 = fdalloc(rf)) < 0 || (fd1 = fdalloc(wf)) < 0) {
    if (fd0 >= 0) {
      p->ofile[fd0] = 0;
    }
    rs_file_close(rf);
    rs_file_close(wf);
    return -1;
  }
  if (copyout(p->pagetable, fdarray, (const u8*)&fd0, sizeof(fd0)) < 0 ||
      copyout(p->pagetable, fdarray + sizeof(fd0), (const u8*)&fd1,
              sizeof(fd1)) < 0) {
    p->ofile[fd0] = 0;
    p->ofile[fd1] = 0;
    rs_file_close(rf);
    rs_file_close(wf);
    return -1;
  }
  return 0;
}

u64 sys_getdents(void) {
  struct file* f;
  u64          output, size;
  u32          sz;
  if (argfd(0, 0, &f) < 0) {
    return -1;
  }
  argaddr(0, &output);
  argaddr(0, &size);

  u8* buf = kalloc();
  if (buf == 0) {
    return ENOMEM;
  }
  if ((sz = rs_file_read(f, buf, PGSIZE)) == 0) {
    kfree(buf);
    return ENOSYS;
  }

  for (u32 pos = 0; pos < sz; pos++) {
    // process((char*)buf+pos);
    pos += strlen((char*)buf + pos);
  }
  kfree(buf);
  return ENOSYS;
  TODO
}
