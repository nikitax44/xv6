#include "kernel/errno.h"
#include "kernel/fcntl.h"
#include "kernel/fs.h"
#include "kernel/memlayout.h"
#include "kernel/param.h"
#include "kernel/riscv.h"
#include "kernel/stat.h"
#include "kernel/syscall.h"
#include "kernel/types.h"
#include "user/user.h"

//
// Tests xv6 system calls.  usertests without arguments runs them all
// and usertests <name> runs <name> test. The test runner creates for
// each test a process and based on the exit status of the process,
// the test runner reports "OK" or "FAILED".  Some tests result in
// kernel printing usertrap messages, which can be ignored if test
// prints "OK".
//

#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wsign-compare"

#define BUFSZ ((MAXOPBLOCKS + 2) * BSIZE)

char buf[BUFSZ];

//
// Section with tests that run fairly quickly.  Use -q if you want to
// run just those.  With -q usertests also runs the ones that take a
// fair of time.
//

// what if you pass ridiculous pointers to system calls
// that read user memory with copyin?
void copyin(char* s) {
  uint64 addrs[] = {0x80000000LL, 0x3fffffe000, 0x3ffffff000, 0x4000000000,
                    0xffffffffffffffff};

  for (int ai = 0; ai < sizeof(addrs) / sizeof(addrs[0]); ai++) {
    uint64 addr = addrs[ai];

    int fd = _open("copyin1", O_CREATE | O_WRONLY);
    if (fd < 0) {
      printf("_open(copyin1) failed\n");
      _exit(1);
    }
    int n = _write(fd, (void*)addr, 8192);
    if (n >= 0) {
      printf("_write(fd, %p, 8192) returned %d, not -1\n", (void*)addr, n);
      _exit(1);
    }
    _close(fd);
    _unlink("copyin1");

    n = _write(1, (char*)addr, 8192);
    if (n > 0) {
      printf("_write(1, %p, 8192) returned %d, not -1 or 0\n", (void*)addr, n);
      _exit(1);
    }

    int fds[2];
    if (_pipe(fds) < 0) {
      printf("_pipe() failed\n");
      _exit(1);
    }
    n = _write(fds[1], (char*)addr, 8192);
    if (n > 0) {
      printf("_write(pipe, %p, 8192) returned %d, not -1 or 0\n", (void*)addr,
             n);
      _exit(1);
    }
    _close(fds[0]);
    _close(fds[1]);
  }
}

// what if you pass ridiculous pointers to system calls
// that write user memory with copyout?
void copyout(char* s) {
  uint64 addrs[] = {0LL,          0x80000000LL, 0x3fffffe000,
                    0x3ffffff000, 0x4000000000, 0xffffffffffffffff};

  for (int ai = 0; ai < sizeof(addrs) / sizeof(addrs[0]); ai++) {
    uint64 addr = addrs[ai];

    int fd = _open("README", 0);
    if (fd < 0) {
      printf("_open(README) failed\n");
      _exit(1);
    }
    int n = _read(fd, (void*)addr, 8192);
    if (n > 0) {
      printf("_read(fd, %p, 8192) returned %d, not -1 or 0\n", (void*)addr, n);
      _exit(1);
    }
    _close(fd);

    int fds[2];
    if (_pipe(fds) < 0) {
      printf("_pipe() failed\n");
      _exit(1);
    }
    n = _write(fds[1], "x", 1);
    if (n != 1) {
      printf("pipe write failed\n");
      _exit(1);
    }
    n = _read(fds[0], (void*)addr, 8192);
    if (n > 0) {
      printf("_read(pipe, %p, 8192) returned %d, not -1 or 0\n", (void*)addr,
             n);
      _exit(1);
    }
    _close(fds[0]);
    _close(fds[1]);
  }
}

// what if you pass ridiculous string pointers to system calls?
void copyinstr1(char* s) {
  uint64 addrs[] = {0x80000000LL, 0x3fffffe000, 0x3ffffff000, 0x4000000000,
                    0xffffffffffffffff};

  for (int ai = 0; ai < sizeof(addrs) / sizeof(addrs[0]); ai++) {
    uint64 addr = addrs[ai];

    int fd = _open((char*)addr, O_CREATE | O_WRONLY);
    if (fd >= 0) {
      printf("_open(%p) returned %d, not -1\n", (void*)addr, fd);
      _exit(1);
    }
  }
}

// what if a string system call argument is exactly the size
// of the kernel buffer it is copied into, so that the null
// would fall just beyond the end of the kernel buffer?
void copyinstr2(char* s) {
  char b[MAXPATH + 1];

  for (int i = 0; i < MAXPATH; i++) {
    b[i] = 'x';
  }
  b[MAXPATH] = '\0';

  int ret = _unlink(b);
  if (ret != -1) {
    printf("_unlink(%s) returned %d, not -1\n", b, ret);
    _exit(1);
  }

  int fd = _open(b, O_CREATE | O_WRONLY);
  if (fd != -1) {
    printf("_open(%s) returned %d, not -1\n", b, fd);
    _exit(1);
  }

  ret = _link(b, b);
  if (ret != -1) {
    printf("_link(%s, %s) returned %d, not -1\n", b, b, ret);
    _exit(1);
  }

  char* args[] = {"xx", 0};
  ret          = _exec(b, args);
  if (ret != E2BIG) {
    printf("_exec(%s) returned %d, not E2BIG\n", b, fd);
    _exit(1);
  }

  int pid = _fork();
  if (pid < 0) {
    printf("fork failed\n");
    _exit(1);
  }
  if (pid == 0) {
    static char big[PGSIZE + 1];
    for (int i = 0; i < PGSIZE; i++) {
      big[i] = 'x';
    }
    big[PGSIZE]   = '\0';
    char* args2[] = {big, big, big, 0};
    ret           = _exec("echo", args2);
    if (ret != E2BIG) {
      printf("_exec(echo, BIG) returned %d, not E2BIG\n", fd);
      _exit(1);
    }
    _exit(747); // OK
  }

  int st = 0;
  _wait(&st);
  if (st != 747) {
    printf("_exec(echo, BIG) succeeded, should have failed\n");
    _exit(1);
  }
}

// what if a string argument crosses over the end of last user page?
void copyinstr3(char* s) {
  _sbrk(8192);
  uint64 top = (uint64)_sbrk(0);
  if ((top % PGSIZE) != 0) {
    _sbrk(PGSIZE - (top % PGSIZE));
  }
  top = (uint64)_sbrk(0);
  if (top % PGSIZE) {
    printf("oops\n");
    _exit(1);
  }

  char* b = (char*)(top - 1);
  *b      = 'x';

  int ret = _unlink(b);
  if (ret != -1) {
    printf("_unlink(%s) returned %d, not -1\n", b, ret);
    _exit(1);
  }

  int fd = _open(b, O_CREATE | O_WRONLY);
  if (fd != -1) {
    printf("_open(%s) returned %d, not -1\n", b, fd);
    _exit(1);
  }

  ret = _link(b, b);
  if (ret != -1) {
    printf("_link(%s, %s) returned %d, not -1\n", b, b, ret);
    _exit(1);
  }

  char* args[] = {"xx", 0};
  ret          = _exec(b, args);
  if (ret != E2BIG) {
    printf("_exec returned %d, not E2BIG\n", ret);
    _exit(1);
  }
}

// See if the kernel refuses to read/write user memory that the
// application doesn't have anymore, because it returned it.
void rwsbrk() {
  int fd, n;

  uint64 a = (uint64)_sbrk(8192);

  if (a == 0xffffffffffffffffLL) {
    printf("_sbrk(rwsbrk) failed\n");
    _exit(1);
  }

  if ((uint64)_sbrk(-8192) == 0xffffffffffffffffLL) {
    printf("_sbrk(rwsbrk) shrink failed\n");
    _exit(1);
  }

  fd = _open("rwsbrk", O_CREATE | O_WRONLY);
  if (fd < 0) {
    printf("_open(rwsbrk) failed\n");
    _exit(1);
  }
  n = _write(fd, (void*)(a + 4096), 1024);
  if (n >= 0) {
    printf("_write(fd, %p, 1024) returned %d, not -1\n", (void*)(a + 4096), n);
    _exit(1);
  }
  _close(fd);
  _unlink("rwsbrk");

  fd = _open("README", O_RDONLY);
  if (fd < 0) {
    printf("_open(rwsbrk) failed\n");
    _exit(1);
  }
  n = _read(fd, (void*)(a + 4096), 10);
  if (n >= 0) {
    printf("_read(fd, %p, 10) returned %d, not -1\n", (void*)(a + 4096), n);
    _exit(1);
  }
  _close(fd);

  _exit(0);
}

// test O_TRUNC.
void truncate1(char* s) {
  char buf[32];

  _unlink("truncfile");
  int fd1 = _open("truncfile", O_CREATE | O_WRONLY | O_TRUNC);
  _write(fd1, "abcd", 4);
  _close(fd1);

  int fd2 = _open("truncfile", O_RDONLY);
  int n   = _read(fd2, buf, sizeof(buf));
  if (n != 4) {
    printf("%s: read %d bytes, wanted 4\n", s, n);
    _exit(1);
  }

  fd1 = _open("truncfile", O_WRONLY | O_TRUNC);

  int fd3 = _open("truncfile", O_RDONLY);
  n       = _read(fd3, buf, sizeof(buf));
  if (n != 0) {
    printf("aaa fd3=%d\n", fd3);
    printf("%s: read %d bytes, wanted 0\n", s, n);
    _exit(1);
  }

  n = _read(fd2, buf, sizeof(buf));
  if (n != 0) {
    printf("bbb fd2=%d\n", fd2);
    printf("%s: read %d bytes, wanted 0\n", s, n);
    _exit(1);
  }

  _write(fd1, "abcdef", 6);

  n = _read(fd3, buf, sizeof(buf));
  if (n != 6) {
    printf("%s: read %d bytes, wanted 6\n", s, n);
    _exit(1);
  }

  n = _read(fd2, buf, sizeof(buf));
  if (n != 2) {
    printf("%s: read %d bytes, wanted 2\n", s, n);
    _exit(1);
  }

  _unlink("truncfile");

  _close(fd1);
  _close(fd2);
  _close(fd3);
}

// write to an open FD whose file has just been truncated.
// this causes a write at an offset beyond the end of the file.
// such writes fail on xv6 (unlike POSIX) but at least
// they don't crash.
void truncate2(char* s) {
  _unlink("truncfile");

  int fd1 = _open("truncfile", O_CREATE | O_TRUNC | O_WRONLY);
  _write(fd1, "abcd", 4);

  int fd2 = _open("truncfile", O_TRUNC | O_WRONLY);

  int n = _write(fd1, "x", 1);
  if (n != -1) {
    printf("%s: write returned %d, expected -1\n", s, n);
    _exit(1);
  }

  _unlink("truncfile");
  _close(fd1);
  _close(fd2);
}

void truncate3(char* s) {
  int pid, xstatus;

  _close(_open("truncfile", O_CREATE | O_TRUNC | O_WRONLY));

  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }

  if (pid == 0) {
    for (int i = 0; i < 100; i++) {
      char buf[32];
      int  fd = _open("truncfile", O_WRONLY);
      if (fd < 0) {
        printf("%s: open failed\n", s);
        _exit(1);
      }
      int n = _write(fd, "1234567890", 10);
      if (n != 10) {
        printf("%s: write got %d, expected 10\n", s, n);
        _exit(1);
      }
      _close(fd);
      fd = _open("truncfile", O_RDONLY);
      _read(fd, buf, sizeof(buf));
      _close(fd);
    }
    _exit(0);
  }

  for (int i = 0; i < 150; i++) {
    int fd = _open("truncfile", O_CREATE | O_WRONLY | O_TRUNC);
    if (fd < 0) {
      printf("%s: open failed\n", s);
      _exit(1);
    }
    int n = _write(fd, "xxx", 3);
    if (n != 3) {
      printf("%s: write got %d, expected 3\n", s, n);
      _exit(1);
    }
    _close(fd);
  }

  _wait(&xstatus);
  _unlink("truncfile");
  _exit(xstatus);
}

// does _chdir() call iput(p->cwd) in a transaction?
void iputtest(char* s) {
  if (_mkdir("iputdir") < 0) {
    printf("%s: mkdir failed\n", s);
    _exit(1);
  }
  if (_chdir("iputdir") < 0) {
    printf("%s: chdir iputdir failed\n", s);
    _exit(1);
  }
  if (_unlink("../iputdir") < 0) {
    printf("%s: unlink ../iputdir failed\n", s);
    _exit(1);
  }
  if (_chdir("/") < 0) {
    printf("%s: chdir / failed\n", s);
    _exit(1);
  }
}

// does _exit() call iput(p->cwd) in a transaction?
void exitiputtest(char* s) {
  int pid, xstatus;

  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid == 0) {
    if (_mkdir("iputdir") < 0) {
      printf("%s: mkdir failed\n", s);
      _exit(1);
    }
    if (_chdir("iputdir") < 0) {
      printf("%s: child chdir failed\n", s);
      _exit(1);
    }
    if (_unlink("../iputdir") < 0) {
      printf("%s: unlink ../iputdir failed\n", s);
      _exit(1);
    }
    _exit(0);
  }
  _wait(&xstatus);
  _exit(xstatus);
}

// does the error path in _open() for attempt to write a
// directory call iput() in a transaction?
// needs a hacked kernel that pauses just after the namei()
// call in sys_open():
//    if((ip = namei(path)) == 0)
//      return -1;
//    {
//      int i;
//      for(i = 0; i < 10000; i++)
//        yield();
//    }
void openiputtest(char* s) {
  int pid, xstatus;

  if (_mkdir("oidir") < 0) {
    printf("%s: mkdir oidir failed\n", s);
    _exit(1);
  }
  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid == 0) {
    int fd = _open("oidir", O_RDWR);
    if (fd >= 0) {
      printf("%s: open directory for write succeeded\n", s);
      _exit(1);
    }
    _exit(0);
  }
  _sleep(1);
  if (_unlink("oidir") != 0) {
    printf("%s: unlink failed\n", s);
    _exit(1);
  }
  _wait(&xstatus);
  _exit(xstatus);
}

// simple file system tests

void opentest(char* s) {
  int fd;

  fd = _open("echo", 0);
  if (fd < 0) {
    printf("%s: open echo failed!\n", s);
    _exit(1);
  }
  _close(fd);
  fd = _open("doesnotexist", 0);
  if (fd >= 0) {
    printf("%s: open doesnotexist succeeded!\n", s);
    _exit(1);
  }
}

void writetest(char* s) {
  int fd;
  int i;
  enum { N = 100, SZ = 10 };

  fd = _open("small", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: error: creat small failed!\n", s);
    _exit(1);
  }
  for (i = 0; i < N; i++) {
    if (_write(fd, "aaaaaaaaaa", SZ) != SZ) {
      printf("%s: error: write aa %d new file failed\n", s, i);
      _exit(1);
    }
    if (_write(fd, "bbbbbbbbbb", SZ) != SZ) {
      printf("%s: error: write bb %d new file failed\n", s, i);
      _exit(1);
    }
  }
  _close(fd);
  fd = _open("small", O_RDONLY);
  if (fd < 0) {
    printf("%s: error: open small failed!\n", s);
    _exit(1);
  }
  i = _read(fd, buf, N * SZ * 2);
  if (i != N * SZ * 2) {
    printf("%s: read failed\n", s);
    _exit(1);
  }
  _close(fd);

  if (_unlink("small") < 0) {
    printf("%s: unlink small failed\n", s);
    _exit(1);
  }
}

void writebig(char* s) {
  int i, fd, n;

  fd = _open("big", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: error: creat big failed!\n", s);
    _exit(1);
  }

  for (i = 0; i < MAXFILE; i++) {
    ((int*)buf)[0] = i;
    if (_write(fd, buf, BSIZE) != BSIZE) {
      printf("%s: error: write big file failed i=%d\n", s, i);
      _exit(1);
    }
  }

  _close(fd);

  fd = _open("big", O_RDONLY);
  if (fd < 0) {
    printf("%s: error: open big failed!\n", s);
    _exit(1);
  }

  n = 0;
  for (;;) {
    i = _read(fd, buf, BSIZE);
    if (i == 0) {
      if (n != MAXFILE) {
        printf("%s: read only %d blocks from big", s, n);
        _exit(1);
      }
      break;
    } else if (i != BSIZE) {
      printf("%s: read failed %d\n", s, i);
      _exit(1);
    }
    if (((int*)buf)[0] != n) {
      printf("%s: read content of block %d is %d\n", s, n, ((int*)buf)[0]);
      _exit(1);
    }
    n++;
  }
  _close(fd);
  if (_unlink("big") < 0) {
    printf("%s: unlink big failed\n", s);
    _exit(1);
  }
}

// many creates, followed by unlink test
void createtest(char* s) {
  int i, fd;
  enum { N = 52 };

  char name[3];
  name[0] = 'a';
  name[2] = '\0';
  for (i = 0; i < N; i++) {
    name[1] = '0' + i;
    fd      = _open(name, O_CREATE | O_RDWR);
    _close(fd);
  }
  name[0] = 'a';
  name[2] = '\0';
  for (i = 0; i < N; i++) {
    name[1] = '0' + i;
    _unlink(name);
  }
}

void dirtest(char* s) {
  if (_mkdir("dir0") < 0) {
    printf("%s: mkdir failed\n", s);
    _exit(1);
  }

  if (_chdir("dir0") < 0) {
    printf("%s: chdir dir0 failed\n", s);
    _exit(1);
  }

  if (_chdir("..") < 0) {
    printf("%s: chdir .. failed\n", s);
    _exit(1);
  }

  if (_unlink("dir0") < 0) {
    printf("%s: unlink dir0 failed\n", s);
    _exit(1);
  }
}

void exectest(char* s) {
  int   fd, xstatus, pid;
  char* echoargv[] = {"echo", "OK", 0};
  char  buf[3];

  _unlink("echo-ok");
  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid == 0) {
    _close(1);
    fd = _open("echo-ok", O_CREATE | O_WRONLY);
    if (fd < 0) {
      printf("%s: create failed\n", s);
      _exit(1);
    }
    if (fd != 1) {
      printf("%s: wrong fd\n", s);
      _exit(1);
    }
    if (_exec("echo", echoargv) != 0) {
      printf("%s: exec echo failed\n", s);
      _exit(1);
    }
    // won't get to here
  }
  if (_wait(&xstatus) != pid) {
    printf("%s: wait failed!\n", s);
  }
  if (xstatus != 0) {
    _exit(xstatus);
  }

  fd = _open("echo-ok", O_RDONLY);
  if (fd < 0) {
    printf("%s: open failed\n", s);
    _exit(1);
  }
  if (_read(fd, buf, 2) != 2) {
    printf("%s: read failed\n", s);
    _exit(1);
  }
  _unlink("echo-ok");
  if (buf[0] == 'O' && buf[1] == 'K') {
    _exit(0);
  } else {
    printf("%s: wrong output\n", s);
    _exit(1);
  }
}

// simple fork and pipe read/write

void pipe1(char* s) {
  int fds[2], pid, xstatus;
  int seq, i, n, cc, total;
  enum { N = 5, SZ = 1033 };

  if (_pipe(fds) != 0) {
    printf("%s: _pipe() failed\n", s);
    _exit(1);
  }
  pid = _fork();
  seq = 0;
  if (pid == 0) {
    _close(fds[0]);
    for (n = 0; n < N; n++) {
      for (i = 0; i < SZ; i++) {
        buf[i] = seq++;
      }
      if (_write(fds[1], buf, SZ) != SZ) {
        printf("%s: pipe1 oops 1\n", s);
        _exit(1);
      }
    }
    _exit(0);
  } else if (pid > 0) {
    _close(fds[1]);
    total = 0;
    cc    = 1;
    while ((n = _read(fds[0], buf, cc)) > 0) {
      for (i = 0; i < n; i++) {
        if ((buf[i] & 0xff) != (seq++ & 0xff)) {
          printf("%s: pipe1 oops 2\n", s);
          return;
        }
      }
      total += n;
      cc = cc * 2;
      if (cc > sizeof(buf)) {
        cc = sizeof(buf);
      }
    }
    if (total != N * SZ) {
      printf("%s: pipe1 oops 3 total %d\n", s, total);
      _exit(1);
    }
    _close(fds[0]);
    _wait(&xstatus);
    _exit(xstatus);
  } else {
    printf("%s: _fork() failed\n", s);
    _exit(1);
  }
}

// test if child is killed (status = -1)
void killstatus(char* s) {
  int xst;

  for (int i = 0; i < 100; i++) {
    int pid1 = _fork();
    if (pid1 < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid1 == 0) {
      while (1) {
        _getpid();
      }
      _exit(0);
    }
    _sleep(1);
    _kill(pid1);
    _wait(&xst);
    if (xst != -1) {
      printf("%s: status should be -1\n", s);
      _exit(1);
    }
  }
  _exit(0);
}

// meant to be run w/ at most two CPUs
void preempt(char* s) {
  int pid1, pid2, pid3;
  int pfds[2];

  pid1 = _fork();
  if (pid1 < 0) {
    printf("%s: fork failed", s);
    _exit(1);
  }
  if (pid1 == 0) {
    for (;;)
      ;
  }

  pid2 = _fork();
  if (pid2 < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid2 == 0) {
    for (;;)
      ;
  }

  _pipe(pfds);
  pid3 = _fork();
  if (pid3 < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid3 == 0) {
    _close(pfds[0]);
    if (_write(pfds[1], "x", 1) != 1) {
      printf("%s: preempt write error", s);
    }
    _close(pfds[1]);
    for (;;)
      ;
  }

  _close(pfds[1]);
  if (_read(pfds[0], buf, sizeof(buf)) != 1) {
    printf("%s: preempt read error", s);
    return;
  }
  _close(pfds[0]);
  printf("kill... ");
  _kill(pid1);
  _kill(pid2);
  _kill(pid3);
  printf("wait... ");
  _wait(0);
  _wait(0);
  _wait(0);
}

// try to find any races between exit and wait
void exitwait(char* s) {
  int i, pid;

  for (i = 0; i < 100; i++) {
    pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid) {
      int xstate;
      if (_wait(&xstate) != pid) {
        printf("%s: wait wrong pid\n", s);
        _exit(1);
      }
      if (i != xstate) {
        printf("%s: wait wrong exit status\n", s);
        _exit(1);
      }
    } else {
      _exit(i);
    }
  }
}

// try to find races in the reparenting
// code that handles a parent exiting
// when it still has live children.
void reparent(char* s) {
  int master_pid = _getpid();
  for (int i = 0; i < 200; i++) {
    int pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid) {
      if (_wait(0) != pid) {
        printf("%s: wait wrong pid\n", s);
        _exit(1);
      }
    } else {
      int pid2 = _fork();
      if (pid2 < 0) {
        _kill(master_pid);
        _exit(1);
      }
      _exit(0);
    }
  }
  _exit(0);
}

// what if two children _exit() at the same time?
void twochildren(char* s) {
  for (int i = 0; i < 1000; i++) {
    int pid1 = _fork();
    if (pid1 < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid1 == 0) {
      _exit(0);
    } else {
      int pid2 = _fork();
      if (pid2 < 0) {
        printf("%s: fork failed\n", s);
        _exit(1);
      }
      if (pid2 == 0) {
        _exit(0);
      } else {
        _wait(0);
        _wait(0);
      }
    }
  }
}

// concurrent forks to try to expose locking bugs.
void forkfork(char* s) {
  enum { N = 2 };

  for (int i = 0; i < N; i++) {
    int pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed", s);
      _exit(1);
    }
    if (pid == 0) {
      for (int j = 0; j < 200; j++) {
        int pid1 = _fork();
        if (pid1 < 0) {
          _exit(1);
        }
        if (pid1 == 0) {
          _exit(0);
        }
        _wait(0);
      }
      _exit(0);
    }
  }

  int xstatus;
  for (int i = 0; i < N; i++) {
    _wait(&xstatus);
    if (xstatus != 0) {
      printf("%s: fork in child failed", s);
      _exit(1);
    }
  }
}

void forkforkfork(char* s) {
  _unlink("stopforking");

  int pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed", s);
    _exit(1);
  }
  if (pid == 0) {
    while (1) {
      int fd = _open("stopforking", 0);
      if (fd >= 0) {
        _exit(0);
      }
      if (_fork() < 0) {
        _close(_open("stopforking", O_CREATE | O_RDWR));
      }
    }

    _exit(0);
  }

  _sleep(20); // two seconds
  _close(_open("stopforking", O_CREATE | O_RDWR));
  _wait(0);
  _sleep(10); // one second
}

// regression test. does reparent() violate the parent-then-child
// locking order when giving away a child to init, so that _exit()
// deadlocks against init's _wait()? also used to trigger a "panic:
// release" due to _exit() releasing a different p->parent->lock than
// it acquired.
void reparent2(char* s) {
  for (int i = 0; i < 800; i++) {
    int pid1 = _fork();
    if (pid1 < 0) {
      printf("fork failed\n");
      _exit(1);
    }
    if (pid1 == 0) {
      _fork();
      _fork();
      _exit(0);
    }
    _wait(0);
  }

  _exit(0);
}

// allocate all mem, free it, and allocate again
void mem(char* s) {
  void *m1, *m2;
  int   pid;

  if ((pid = _fork()) == 0) {
    m1 = 0;
    while ((m2 = malloc(10001)) != 0) {
      *(char**)m2 = m1;
      m1          = m2;
    }
    while (m1) {
      m2 = *(char**)m1;
      free(m1);
      m1 = m2;
    }
    m1 = malloc(1024 * 20);
    if (m1 == 0) {
      printf("%s: couldn't allocate mem?!!\n", s);
      _exit(1);
    }
    free(m1);
    _exit(0);
  } else {
    int xstatus;
    _wait(&xstatus);
    if (xstatus == -1) {
      // probably page fault, so might be lazy lab,
      // so OK.
      _exit(0);
    }
    _exit(xstatus);
  }
}

// More file system tests

// two processes write to the same file descriptor
// is the offset shared? does inode locking work?
void sharedfd(char* s) {
  int fd, pid, i, n, nc, np;
  enum { N = 1000, SZ = 10 };
  char buf[SZ];

  _unlink("sharedfd");
  fd = _open("sharedfd", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: cannot open sharedfd for writing", s);
    _exit(1);
  }
  pid = _fork();
  memset(buf, pid == 0 ? 'c' : 'p', sizeof(buf));
  for (i = 0; i < N; i++) {
    if (_write(fd, buf, sizeof(buf)) != sizeof(buf)) {
      printf("%s: write sharedfd failed\n", s);
      _exit(1);
    }
  }
  if (pid == 0) {
    _exit(0);
  } else {
    int xstatus;
    _wait(&xstatus);
    if (xstatus != 0) {
      _exit(xstatus);
    }
  }

  _close(fd);
  fd = _open("sharedfd", 0);
  if (fd < 0) {
    printf("%s: cannot open sharedfd for reading\n", s);
    _exit(1);
  }
  nc = np = 0;
  while ((n = _read(fd, buf, sizeof(buf))) > 0) {
    for (i = 0; i < sizeof(buf); i++) {
      if (buf[i] == 'c') {
        nc++;
      }
      if (buf[i] == 'p') {
        np++;
      }
    }
  }
  _close(fd);
  _unlink("sharedfd");
  if (nc == N * SZ && np == N * SZ) {
    _exit(0);
  } else {
    printf("%s: nc/np test fails\n", s);
    _exit(1);
  }
}

// four processes write different files at the same
// time, to test block allocation.
void fourfiles(char* s) {
  int   fd, pid, i, j, n, total, pi;
  char* names[] = {"f0", "f1", "f2", "f3"};
  char* fname;
  enum { N = 12, NCHILD = 4, SZ = 500 };

  for (pi = 0; pi < NCHILD; pi++) {
    fname = names[pi];
    _unlink(fname);

    pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }

    if (pid == 0) {
      fd = _open(fname, O_CREATE | O_RDWR);
      if (fd < 0) {
        printf("%s: create failed\n", s);
        _exit(1);
      }

      memset(buf, '0' + pi, SZ);
      for (i = 0; i < N; i++) {
        if ((n = _write(fd, buf, SZ)) != SZ) {
          printf("write failed %d\n", n);
          _exit(1);
        }
      }
      _exit(0);
    }
  }

  int xstatus;
  for (pi = 0; pi < NCHILD; pi++) {
    _wait(&xstatus);
    if (xstatus != 0) {
      _exit(xstatus);
    }
  }

  for (i = 0; i < NCHILD; i++) {
    fname = names[i];
    fd    = _open(fname, 0);
    total = 0;
    while ((n = _read(fd, buf, sizeof(buf))) > 0) {
      for (j = 0; j < n; j++) {
        if (buf[j] != '0' + i) {
          printf("%s: wrong char\n", s);
          _exit(1);
        }
      }
      total += n;
    }
    _close(fd);
    if (total != N * SZ) {
      printf("wrong length %d\n", total);
      _exit(1);
    }
    _unlink(fname);
  }
}

// four processes create and delete different files in same directory
void createdelete(char* s) {
  enum { N = 20, NCHILD = 4 };
  int  pid, i, fd, pi;
  char name[32];

  for (pi = 0; pi < NCHILD; pi++) {
    pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }

    if (pid == 0) {
      name[0] = 'p' + pi;
      name[2] = '\0';
      for (i = 0; i < N; i++) {
        name[1] = '0' + i;
        fd      = _open(name, O_CREATE | O_RDWR);
        if (fd < 0) {
          printf("%s: create failed\n", s);
          _exit(1);
        }
        _close(fd);
        if (i > 0 && (i % 2) == 0) {
          name[1] = '0' + (i / 2);
          if (_unlink(name) < 0) {
            printf("%s: unlink failed\n", s);
            _exit(1);
          }
        }
      }
      _exit(0);
    }
  }

  int xstatus;
  for (pi = 0; pi < NCHILD; pi++) {
    _wait(&xstatus);
    if (xstatus != 0) {
      _exit(1);
    }
  }

  name[0] = name[1] = name[2] = 0;
  for (i = 0; i < N; i++) {
    for (pi = 0; pi < NCHILD; pi++) {
      name[0] = 'p' + pi;
      name[1] = '0' + i;
      fd      = _open(name, 0);
      if ((i == 0 || i >= N / 2) && fd < 0) {
        printf("%s: oops createdelete %s didn't exist\n", s, name);
        _exit(1);
      } else if ((i >= 1 && i < N / 2) && fd >= 0) {
        printf("%s: oops createdelete %s did exist\n", s, name);
        _exit(1);
      }
      if (fd >= 0) {
        _close(fd);
      }
    }
  }

  for (i = 0; i < N; i++) {
    for (pi = 0; pi < NCHILD; pi++) {
      name[0] = 'p' + pi;
      name[1] = '0' + i;
      _unlink(name);
    }
  }
}

// can I unlink a file and still read it?
void unlinkread(char* s) {
  enum { SZ = 5 };
  int fd, fd1;

  fd = _open("unlinkread", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: create unlinkread failed\n", s);
    _exit(1);
  }
  _write(fd, "hello", SZ);
  _close(fd);

  fd = _open("unlinkread", O_RDWR);
  if (fd < 0) {
    printf("%s: open unlinkread failed\n", s);
    _exit(1);
  }
  if (_unlink("unlinkread") != 0) {
    printf("%s: unlink unlinkread failed\n", s);
    _exit(1);
  }

  fd1 = _open("unlinkread", O_CREATE | O_RDWR);
  _write(fd1, "yyy", 3);
  _close(fd1);

  if (_read(fd, buf, sizeof(buf)) != SZ) {
    printf("%s: unlinkread read failed", s);
    _exit(1);
  }
  if (buf[0] != 'h') {
    printf("%s: unlinkread wrong data\n", s);
    _exit(1);
  }
  if (_write(fd, buf, 10) != 10) {
    printf("%s: unlinkread write failed\n", s);
    _exit(1);
  }
  _close(fd);
  _unlink("unlinkread");
}

void linktest(char* s) {
  enum { SZ = 5 };
  int fd;

  _unlink("lf1");
  _unlink("lf2");

  fd = _open("lf1", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: create lf1 failed\n", s);
    _exit(1);
  }
  if (_write(fd, "hello", SZ) != SZ) {
    printf("%s: write lf1 failed\n", s);
    _exit(1);
  }
  _close(fd);

  if (_link("lf1", "lf2") < 0) {
    printf("%s: link lf1 lf2 failed\n", s);
    _exit(1);
  }
  _unlink("lf1");

  if (_open("lf1", 0) >= 0) {
    printf("%s: unlinked lf1 but it is still there!\n", s);
    _exit(1);
  }

  fd = _open("lf2", 0);
  if (fd < 0) {
    printf("%s: open lf2 failed\n", s);
    _exit(1);
  }
  if (_read(fd, buf, sizeof(buf)) != SZ) {
    printf("%s: read lf2 failed\n", s);
    _exit(1);
  }
  _close(fd);

  if (_link("lf2", "lf2") >= 0) {
    printf("%s: link lf2 lf2 succeeded! oops\n", s);
    _exit(1);
  }

  _unlink("lf2");
  if (_link("lf2", "lf1") >= 0) {
    printf("%s: link non-existent succeeded! oops\n", s);
    _exit(1);
  }

  if (_link(".", "lf1") >= 0) {
    printf("%s: link . lf1 succeeded! oops\n", s);
    _exit(1);
  }
}

// test concurrent create/link/unlink of the same file
void concreate(char* s) {
  enum { N = 40 };
  char file[3];
  int  i, pid, n, fd;
  char fa[N];
  struct {
    ushort inum;
    char   name[DIRSIZ];
  } de;

  file[0] = 'C';
  file[2] = '\0';
  for (i = 0; i < N; i++) {
    file[1] = '0' + i;
    _unlink(file);
    pid = _fork();
    if (pid && (i % 3) == 1) {
      _link("C0", file);
    } else if (pid == 0 && (i % 5) == 1) {
      _link("C0", file);
    } else {
      fd = _open(file, O_CREATE | O_RDWR);
      if (fd < 0) {
        printf("concreate create %s failed\n", file);
        _exit(1);
      }
      _close(fd);
    }
    if (pid == 0) {
      _exit(0);
    } else {
      int xstatus;
      _wait(&xstatus);
      if (xstatus != 0) {
        _exit(1);
      }
    }
  }

  memset(fa, 0, sizeof(fa));
  fd = _open(".", 0);
  n  = 0;
  while (_read(fd, &de, sizeof(de)) > 0) {
    if (de.inum == 0) {
      continue;
    }
    if (de.name[0] == 'C' && de.name[2] == '\0') {
      i = de.name[1] - '0';
      if (i < 0 || i >= sizeof(fa)) {
        printf("%s: concreate weird file %s\n", s, de.name);
        _exit(1);
      }
      if (fa[i]) {
        printf("%s: concreate duplicate file %s\n", s, de.name);
        _exit(1);
      }
      fa[i] = 1;
      n++;
    }
  }
  _close(fd);

  if (n != N) {
    printf("%s: concreate not enough files in directory listing\n", s);
    _exit(1);
  }

  for (i = 0; i < N; i++) {
    file[1] = '0' + i;
    pid     = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (((i % 3) == 0 && pid == 0) || ((i % 3) == 1 && pid != 0)) {
      _close(_open(file, 0));
      _close(_open(file, 0));
      _close(_open(file, 0));
      _close(_open(file, 0));
      _close(_open(file, 0));
      _close(_open(file, 0));
    } else {
      _unlink(file);
      _unlink(file);
      _unlink(file);
      _unlink(file);
      _unlink(file);
      _unlink(file);
    }
    if (pid == 0) {
      _exit(0);
    } else {
      _wait(0);
    }
  }
}

// another concurrent link/unlink/create test,
// to look for deadlocks.
void linkunlink(char* s) {
  int pid, i;

  _unlink("x");
  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }

  unsigned int x = (pid ? 1 : 97);
  for (i = 0; i < 100; i++) {
    x = x * 1103515245 + 12345;
    if ((x % 3) == 0) {
      _close(_open("x", O_RDWR | O_CREATE));
    } else if ((x % 3) == 1) {
      _link("cat", "x");
    } else {
      _unlink("x");
    }
  }

  if (pid) {
    _wait(0);
  } else {
    _exit(0);
  }
}

void subdir(char* s) {
  int fd, cc;

  _unlink("ff");
  if (_mkdir("dd") != 0) {
    printf("%s: mkdir dd failed\n", s);
    _exit(1);
  }

  fd = _open("dd/ff", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: create dd/ff failed\n", s);
    _exit(1);
  }
  _write(fd, "ff", 2);
  _close(fd);

  if (_unlink("dd") >= 0) {
    printf("%s: unlink dd (non-empty dir) succeeded!\n", s);
    _exit(1);
  }

  if (_mkdir("/dd/dd") != 0) {
    printf("%s: subdir mkdir dd/dd failed\n", s);
    _exit(1);
  }

  fd = _open("dd/dd/ff", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: create dd/dd/ff failed\n", s);
    _exit(1);
  }
  _write(fd, "FF", 2);
  _close(fd);

  fd = _open("dd/dd/../ff", 0);
  if (fd < 0) {
    printf("%s: open dd/dd/../ff failed\n", s);
    _exit(1);
  }
  cc = _read(fd, buf, sizeof(buf));
  if (cc != 2 || buf[0] != 'f') {
    printf("%s: dd/dd/../ff wrong content\n", s);
    _exit(1);
  }
  _close(fd);

  if (_link("dd/dd/ff", "dd/dd/ffff") != 0) {
    printf("%s: link dd/dd/ff dd/dd/ffff failed\n", s);
    _exit(1);
  }

  if (_unlink("dd/dd/ff") != 0) {
    printf("%s: unlink dd/dd/ff failed\n", s);
    _exit(1);
  }
  if (_open("dd/dd/ff", O_RDONLY) >= 0) {
    printf("%s: open (unlinked) dd/dd/ff succeeded\n", s);
    _exit(1);
  }

  if (_chdir("dd") != 0) {
    printf("%s: chdir dd failed\n", s);
    _exit(1);
  }
  if (_chdir("dd/../../dd") != 0) {
    printf("%s: chdir dd/../../dd failed\n", s);
    _exit(1);
  }
  if (_chdir("dd/../../../dd") != 0) {
    printf("%s: chdir dd/../../../dd failed\n", s);
    _exit(1);
  }
  if (_chdir("./..") != 0) {
    printf("%s: chdir ./.. failed\n", s);
    _exit(1);
  }

  fd = _open("dd/dd/ffff", 0);
  if (fd < 0) {
    printf("%s: open dd/dd/ffff failed\n", s);
    _exit(1);
  }
  if (_read(fd, buf, sizeof(buf)) != 2) {
    printf("%s: read dd/dd/ffff wrong len\n", s);
    _exit(1);
  }
  _close(fd);

  if (_open("dd/dd/ff", O_RDONLY) >= 0) {
    printf("%s: open (unlinked) dd/dd/ff succeeded!\n", s);
    _exit(1);
  }

  if (_open("dd/ff/ff", O_CREATE | O_RDWR) >= 0) {
    printf("%s: create dd/ff/ff succeeded!\n", s);
    _exit(1);
  }
  if (_open("dd/xx/ff", O_CREATE | O_RDWR) >= 0) {
    printf("%s: create dd/xx/ff succeeded!\n", s);
    _exit(1);
  }
  if (_open("dd", O_CREATE) >= 0) {
    printf("%s: create dd succeeded!\n", s);
    _exit(1);
  }
  if (_open("dd", O_RDWR) >= 0) {
    printf("%s: open dd rdwr succeeded!\n", s);
    _exit(1);
  }
  if (_open("dd", O_WRONLY) >= 0) {
    printf("%s: open dd wronly succeeded!\n", s);
    _exit(1);
  }
  if (_link("dd/ff/ff", "dd/dd/xx") == 0) {
    printf("%s: link dd/ff/ff dd/dd/xx succeeded!\n", s);
    _exit(1);
  }
  if (_link("dd/xx/ff", "dd/dd/xx") == 0) {
    printf("%s: link dd/xx/ff dd/dd/xx succeeded!\n", s);
    _exit(1);
  }
  if (_link("dd/ff", "dd/dd/ffff") == 0) {
    printf("%s: link dd/ff dd/dd/ffff succeeded!\n", s);
    _exit(1);
  }
  if (_mkdir("dd/ff/ff") == 0) {
    printf("%s: mkdir dd/ff/ff succeeded!\n", s);
    _exit(1);
  }
  if (_mkdir("dd/xx/ff") == 0) {
    printf("%s: mkdir dd/xx/ff succeeded!\n", s);
    _exit(1);
  }
  if (_mkdir("dd/dd/ffff") == 0) {
    printf("%s: mkdir dd/dd/ffff succeeded!\n", s);
    _exit(1);
  }
  if (_unlink("dd/xx/ff") == 0) {
    printf("%s: unlink dd/xx/ff succeeded!\n", s);
    _exit(1);
  }
  if (_unlink("dd/ff/ff") == 0) {
    printf("%s: unlink dd/ff/ff succeeded!\n", s);
    _exit(1);
  }
  if (_chdir("dd/ff") == 0) {
    printf("%s: chdir dd/ff succeeded!\n", s);
    _exit(1);
  }
  if (_chdir("dd/xx") == 0) {
    printf("%s: chdir dd/xx succeeded!\n", s);
    _exit(1);
  }

  if (_unlink("dd/dd/ffff") != 0) {
    printf("%s: unlink dd/dd/ff failed\n", s);
    _exit(1);
  }
  if (_unlink("dd/ff") != 0) {
    printf("%s: unlink dd/ff failed\n", s);
    _exit(1);
  }
  if (_unlink("dd") == 0) {
    printf("%s: unlink non-empty dd succeeded!\n", s);
    _exit(1);
  }
  if (_unlink("dd/dd") < 0) {
    printf("%s: unlink dd/dd failed\n", s);
    _exit(1);
  }
  if (_unlink("dd") < 0) {
    printf("%s: unlink dd failed\n", s);
    _exit(1);
  }
}

// test writes that are larger than the log.
void bigwrite(char* s) {
  int fd, sz;

  _unlink("bigwrite");
  for (sz = 499; sz < (MAXOPBLOCKS + 2) * BSIZE; sz += 471) {
    fd = _open("bigwrite", O_CREATE | O_RDWR);
    if (fd < 0) {
      printf("%s: cannot create bigwrite\n", s);
      _exit(1);
    }
    int i;
    for (i = 0; i < 2; i++) {
      int cc = _write(fd, buf, sz);
      if (cc != sz) {
        printf("%s: _write(%d) ret %d\n", s, sz, cc);
        _exit(1);
      }
    }
    _close(fd);
    _unlink("bigwrite");
  }
}

void bigfile(char* s) {
  enum { N = 20, SZ = 600 };
  int fd, i, total, cc;

  _unlink("bigfile.dat");
  fd = _open("bigfile.dat", O_CREATE | O_RDWR);
  if (fd < 0) {
    printf("%s: cannot create bigfile", s);
    _exit(1);
  }
  for (i = 0; i < N; i++) {
    memset(buf, i, SZ);
    if (_write(fd, buf, SZ) != SZ) {
      printf("%s: write bigfile failed\n", s);
      _exit(1);
    }
  }
  _close(fd);

  fd = _open("bigfile.dat", 0);
  if (fd < 0) {
    printf("%s: cannot open bigfile\n", s);
    _exit(1);
  }
  total = 0;
  for (i = 0;; i++) {
    cc = _read(fd, buf, SZ / 2);
    if (cc < 0) {
      printf("%s: read bigfile failed\n", s);
      _exit(1);
    }
    if (cc == 0) {
      break;
    }
    if (cc != SZ / 2) {
      printf("%s: short read bigfile\n", s);
      _exit(1);
    }
    if (buf[0] != i / 2 || buf[SZ / 2 - 1] != i / 2) {
      printf("%s: read bigfile wrong data\n", s);
      _exit(1);
    }
    total += cc;
  }
  _close(fd);
  if (total != N * SZ) {
    printf("%s: read bigfile wrong total\n", s);
    _exit(1);
  }
  _unlink("bigfile.dat");
}

void fourteen(char* s) {
  int fd;

  // DIRSIZ is 14.

  if (_mkdir("12345678901234") != 0) {
    printf("%s: mkdir 12345678901234 failed\n", s);
    _exit(1);
  }
  if (_mkdir("12345678901234/123456789012345") != 0) {
    printf("%s: mkdir 12345678901234/123456789012345 failed\n", s);
    _exit(1);
  }
  fd = _open("123456789012345/123456789012345/123456789012345", O_CREATE);
  if (fd < 0) {
    printf(
        "%s: create 123456789012345/123456789012345/123456789012345 failed\n",
        s);
    _exit(1);
  }
  _close(fd);
  fd = _open("12345678901234/12345678901234/12345678901234", 0);
  if (fd < 0) {
    printf("%s: open 12345678901234/12345678901234/12345678901234 failed\n", s);
    _exit(1);
  }
  _close(fd);

  if (_mkdir("12345678901234/12345678901234") == 0) {
    printf("%s: mkdir 12345678901234/12345678901234 succeeded!\n", s);
    _exit(1);
  }
  if (_mkdir("123456789012345/12345678901234") == 0) {
    printf("%s: mkdir 12345678901234/123456789012345 succeeded!\n", s);
    _exit(1);
  }

  // clean up
  _unlink("123456789012345/12345678901234");
  _unlink("12345678901234/12345678901234");
  _unlink("12345678901234/12345678901234/12345678901234");
  _unlink("123456789012345/123456789012345/123456789012345");
  _unlink("12345678901234/123456789012345");
  _unlink("12345678901234");
}

void rmdot(char* s) {
  if (_mkdir("dots") != 0) {
    printf("%s: mkdir dots failed\n", s);
    _exit(1);
  }
  if (_chdir("dots") != 0) {
    printf("%s: chdir dots failed\n", s);
    _exit(1);
  }
  if (_unlink(".") == 0) {
    printf("%s: rm . worked!\n", s);
    _exit(1);
  }
  if (_unlink("..") == 0) {
    printf("%s: rm .. worked!\n", s);
    _exit(1);
  }
  if (_chdir("/") != 0) {
    printf("%s: chdir / failed\n", s);
    _exit(1);
  }
  if (_unlink("dots/.") == 0) {
    printf("%s: unlink dots/. worked!\n", s);
    _exit(1);
  }
  if (_unlink("dots/..") == 0) {
    printf("%s: unlink dots/.. worked!\n", s);
    _exit(1);
  }
  if (_unlink("dots") != 0) {
    printf("%s: unlink dots failed!\n", s);
    _exit(1);
  }
}

void dirfile(char* s) {
  int fd;

  fd = _open("dirfile", O_CREATE);
  if (fd < 0) {
    printf("%s: create dirfile failed\n", s);
    _exit(1);
  }
  _close(fd);
  if (_chdir("dirfile") == 0) {
    printf("%s: chdir dirfile succeeded!\n", s);
    _exit(1);
  }
  fd = _open("dirfile/xx", 0);
  if (fd >= 0) {
    printf("%s: create dirfile/xx succeeded!\n", s);
    _exit(1);
  }
  fd = _open("dirfile/xx", O_CREATE);
  if (fd >= 0) {
    printf("%s: create dirfile/xx succeeded!\n", s);
    _exit(1);
  }
  if (_mkdir("dirfile/xx") == 0) {
    printf("%s: mkdir dirfile/xx succeeded!\n", s);
    _exit(1);
  }
  if (_unlink("dirfile/xx") == 0) {
    printf("%s: unlink dirfile/xx succeeded!\n", s);
    _exit(1);
  }
  if (_link("README", "dirfile/xx") == 0) {
    printf("%s: link to dirfile/xx succeeded!\n", s);
    _exit(1);
  }
  if (_unlink("dirfile") != 0) {
    printf("%s: unlink dirfile failed!\n", s);
    _exit(1);
  }

  fd = _open(".", O_RDWR);
  if (fd >= 0) {
    printf("%s: open . for writing succeeded!\n", s);
    _exit(1);
  }
  fd = _open(".", 0);
  if (_write(fd, "x", 1) > 0) {
    printf("%s: write . succeeded!\n", s);
    _exit(1);
  }
  _close(fd);
}

// test that iput() is called at the end of _namei().
// also tests empty file names.
void iref(char* s) {
  int i, fd;

  for (i = 0; i < NINODE + 1; i++) {
    if (_mkdir("irefd") != 0) {
      printf("%s: mkdir irefd failed\n", s);
      _exit(1);
    }
    if (_chdir("irefd") != 0) {
      printf("%s: chdir irefd failed\n", s);
      _exit(1);
    }

    _mkdir("");
    _link("README", "");
    fd = _open("", O_CREATE);
    if (fd >= 0) {
      _close(fd);
    }
    fd = _open("xx", O_CREATE);
    if (fd >= 0) {
      _close(fd);
    }
    _unlink("xx");
  }

  // clean up
  for (i = 0; i < NINODE + 1; i++) {
    _chdir("..");
    _unlink("irefd");
  }

  _chdir("/");
}

// test that fork fails gracefully
// the forktest binary also does this, but it runs out of proc entries first.
// inside the bigger usertests binary, we run out of memory first.
void forktest(char* s) {
  enum { N = 1000 };
  int n, pid;

  for (n = 0; n < N; n++) {
    pid = _fork();
    if (pid < 0) {
      break;
    }
    if (pid == 0) {
      _exit(0);
    }
  }

  if (n == 0) {
    printf("%s: no fork at all!\n", s);
    _exit(1);
  }

  if (n == N) {
    printf("%s: fork claimed to work 1000 times!\n", s);
    _exit(1);
  }

  for (; n > 0; n--) {
    if (_wait(0) < 0) {
      printf("%s: wait stopped early\n", s);
      _exit(1);
    }
  }

  if (_wait(0) != -1) {
    printf("%s: wait got too many\n", s);
    _exit(1);
  }
}

void sbrkbasic(char* s) {
  enum { TOOMUCH = 1024 * 1024 * 1024 };
  int   i, pid, xstatus;
  char *c, *a, *b;

  // does _sbrk() return the expected failure value?
  pid = _fork();
  if (pid < 0) {
    printf("fork failed in sbrkbasic\n");
    _exit(1);
  }
  if (pid == 0) {
    a = _sbrk(TOOMUCH);
    if (a == (char*)0xffffffffffffffffL) {
      // it's OK if this fails.
      _exit(0);
    }

    for (b = a; b < a + TOOMUCH; b += 4096) {
      *b = 99;
    }

    // we should not get here! either _sbrk(TOOMUCH)
    // should have failed, or (with lazy allocation)
    // a pagefault should have killed this process.
    _exit(1);
  }

  _wait(&xstatus);
  if (xstatus == 1) {
    printf("%s: too much memory allocated!\n", s);
    _exit(1);
  }

  // can one _sbrk() less than a page?
  a = _sbrk(0);
  for (i = 0; i < 5000; i++) {
    b = _sbrk(1);
    if (b != a) {
      printf("%s: sbrk test failed %d %p %p\n", s, i, a, b);
      _exit(1);
    }
    *b = 1;
    a  = b + 1;
  }
  pid = _fork();
  if (pid < 0) {
    printf("%s: sbrk test fork failed\n", s);
    _exit(1);
  }
  c = _sbrk(1);
  c = _sbrk(1);
  if (c != a + 1) {
    printf("%s: sbrk test failed post-fork\n", s);
    _exit(1);
  }
  if (pid == 0) {
    _exit(0);
  }
  _wait(&xstatus);
  _exit(xstatus);
}

void sbrkmuch(char* s) {
  enum { BIG = 100 * 1024 * 1024 };
  char * c, *oldbrk, *a, *lastaddr, *p;
  uint64 amt;

  oldbrk = _sbrk(0);

  // can one grow address space to something big?
  a   = _sbrk(0);
  amt = BIG - (uint64)a;
  p   = _sbrk(amt);
  if (p != a) {
    printf("%s: sbrk test failed to grow big address space; enough phys mem?\n",
           s);
    _exit(1);
  }

  // touch each page to make sure it exists.
  char* eee = _sbrk(0);
  for (char* pp = a; pp < eee; pp += 4096) {
    *pp = 1;
  }

  lastaddr  = (char*)(BIG - 1);
  *lastaddr = 99;

  // can one de-allocate?
  a = _sbrk(0);
  c = _sbrk(-PGSIZE);
  if (c == (char*)0xffffffffffffffffL) {
    printf("%s: sbrk could not deallocate\n", s);
    _exit(1);
  }
  c = _sbrk(0);
  if (c != a - PGSIZE) {
    printf("%s: sbrk deallocation produced wrong address, a %p c %p\n", s, a,
           c);
    _exit(1);
  }

  // can one re-allocate that page?
  a = _sbrk(0);
  c = _sbrk(PGSIZE);
  if (c != a || _sbrk(0) != a + PGSIZE) {
    printf("%s: sbrk re-allocation failed, a %p c %p\n", s, a, c);
    _exit(1);
  }
  if (*lastaddr == 99) {
    // should be zero
    printf("%s: sbrk de-allocation didn't really deallocate\n", s);
    _exit(1);
  }

  a = _sbrk(0);
  c = _sbrk(-(_sbrk(0) - oldbrk));
  if (c != a) {
    printf("%s: sbrk downsize failed, a %p c %p\n", s, a, c);
    _exit(1);
  }
}

// can we read the kernel's memory?
void kernmem(char* s) {
  char* a;
  int   pid;

  for (a = (char*)(KERNBASE); a < (char*)(KERNBASE + 2000000); a += 50000) {
    pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid == 0) {
      printf("%s: oops could read %p = %x\n", s, a, *a);
      _exit(1);
    }
    int xstatus;
    _wait(&xstatus);
    if (xstatus != -1) { // did kernel kill child?
      _exit(1);
    }
  }
}

// user code should not be able to write to addresses above MAXVA.
void MAXVAplus(char* s) {
  volatile uint64 a = MAXVA;
  for (; a != 0; a <<= 1) {
    int pid;
    pid = _fork();
    if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    if (pid == 0) {
      *(char*)a = 99;
      printf("%s: oops wrote %p\n", s, (void*)a);
      _exit(1);
    }
    int xstatus;
    _wait(&xstatus);
    if (xstatus != -1) { // did kernel kill child?
      _exit(1);
    }
  }
}

// if we run the system out of memory, does it clean up the last
// failed allocation?
void sbrkfail(char* s) {
  enum { BIG = 100 * 1024 * 1024 };
  int   i, xstatus;
  int   fds[2];
  char  scratch;
  char *c, *a;
  int   pids[10];
  int   pid;

  if (_pipe(fds) != 0) {
    printf("%s: _pipe() failed\n", s);
    _exit(1);
  }
  for (i = 0; i < sizeof(pids) / sizeof(pids[0]); i++) {
    if ((pids[i] = _fork()) == 0) {
      // allocate a lot of memory
      _sbrk(BIG - (uint64)_sbrk(0));
      _write(fds[1], "x", 1);
      // sit around until killed
      for (;;) {
        _sleep(1000);
      }
    }
    if (pids[i] != -1) {
      _read(fds[0], &scratch, 1);
    }
  }

  // if those failed allocations freed up the pages they did allocate,
  // we'll be able to allocate here
  c = _sbrk(PGSIZE);
  for (i = 0; i < sizeof(pids) / sizeof(pids[0]); i++) {
    if (pids[i] == -1) {
      continue;
    }
    _kill(pids[i]);
    _wait(0);
  }
  if (c == (char*)0xffffffffffffffffL) {
    printf("%s: failed sbrk leaked memory\n", s);
    _exit(1);
  }

  // test running fork with the above allocated page
  pid = _fork();
  if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  if (pid == 0) {
    // allocate a lot of memory.
    // this should produce a page fault,
    // and thus not complete.
    a = _sbrk(0);
    _sbrk(10 * BIG);
    int n = 0;
    for (i = 0; i < 10 * BIG; i += PGSIZE) {
      n += *(a + i);
    }
    // print n so the compiler doesn't optimize away
    // the for loop.
    printf("%s: allocate a lot of memory succeeded %d\n", s, n);
    _exit(1);
  }
  _wait(&xstatus);
  if (xstatus != -1 && xstatus != 2) {
    _exit(1);
  }
}

// test reads/writes from/to allocated memory
void sbrkarg(char* s) {
  char* a;
  int   fd, n;

  a  = _sbrk(PGSIZE);
  fd = _open("sbrk", O_CREATE | O_WRONLY);
  _unlink("sbrk");
  if (fd < 0) {
    printf("%s: open sbrk failed\n", s);
    _exit(1);
  }
  if ((n = _write(fd, a, PGSIZE)) < 0) {
    printf("%s: write sbrk failed\n", s);
    _exit(1);
  }
  _close(fd);

  // test writes to allocated memory
  a = _sbrk(PGSIZE);
  if (_pipe((int*)a) != 0) {
    printf("%s: _pipe() failed\n", s);
    _exit(1);
  }
}

void validatetest(char* s) {
  int    hi;
  uint64 p;

  hi = 1100 * 1024;
  for (p = 0; p <= (uint)hi; p += PGSIZE) {
    // try to crash the kernel by passing in a bad string pointer
    if (_link("nosuchfile", (char*)p) != -1) {
      printf("%s: link should not succeed\n", s);
      _exit(1);
    }
  }
}

// does uninitialized data start out zero?
char uninit[10000];
void bsstest(char* s) {
  int i;

  for (i = 0; i < sizeof(uninit); i++) {
    if (uninit[i] != '\0') {
      printf("%s: bss test failed\n", s);
      _exit(1);
    }
  }
}

// does exec return an error if the arguments
// are larger than stack size? or does it write
// below the stack and wreck the instructions/data?
void bigargtest(char* s) {
  int pid, fd, xstatus;

  _unlink("bigarg-ok");
  pid = _fork();
  if (pid == 0) {
    static char* args[MAXARG];
    int          i;
    char         big[(USERSTACK * PGSIZE) / (MAXARG - 1) + 2];
    memset(big, ' ', sizeof(big));
    big[sizeof(big) - 1] = '\0';
    for (i = 0; i < MAXARG - 1; i++) {
      args[i] = big;
    }
    args[MAXARG - 1] = 0;
    // this _exec() should fail (and return) because the
    // arguments are too large.
    _exec("echo", args);
    fd = _open("bigarg-ok", O_CREATE);
    _close(fd);
    _exit(0);
  } else if (pid < 0) {
    printf("%s: bigargtest: fork failed\n", s);
    _exit(1);
  }

  _wait(&xstatus);
  if (xstatus != 0) {
    _exit(xstatus);
  }
  fd = _open("bigarg-ok", 0);
  if (fd < 0) {
    printf("%s: bigarg test failed!\n", s);
    _exit(1);
  }
  _close(fd);
}

// what happens when the file system runs out of blocks?
// answer: balloc panics, so this test is not useful.
void fsfull() {
  int nfiles;
  int fsblocks = 0;

  printf("fsfull test\n");

  for (nfiles = 0;; nfiles++) {
    char name[64];
    name[0] = 'f';
    name[1] = '0' + nfiles / 1000;
    name[2] = '0' + (nfiles % 1000) / 100;
    name[3] = '0' + (nfiles % 100) / 10;
    name[4] = '0' + (nfiles % 10);
    name[5] = '\0';
    printf("writing %s\n", name);
    int fd = _open(name, O_CREATE | O_RDWR);
    if (fd < 0) {
      printf("open %s failed\n", name);
      break;
    }
    int total = 0;
    while (1) {
      int cc = _write(fd, buf, BSIZE);
      if (cc < BSIZE) {
        break;
      }
      total += cc;
      fsblocks++;
    }
    printf("wrote %d bytes\n", total);
    _close(fd);
    if (total == 0) {
      break;
    }
  }

  while (nfiles >= 0) {
    char name[64];
    name[0] = 'f';
    name[1] = '0' + nfiles / 1000;
    name[2] = '0' + (nfiles % 1000) / 100;
    name[3] = '0' + (nfiles % 100) / 10;
    name[4] = '0' + (nfiles % 10);
    name[5] = '\0';
    _unlink(name);
    nfiles--;
  }

  printf("fsfull test finished\n");
}

void argptest(char* s) {
  int fd;
  fd = _open("init", O_RDONLY);
  if (fd < 0) {
    printf("%s: open failed\n", s);
    _exit(1);
  }
  _read(fd, _sbrk(0) - 1, -1);
  _close(fd);
}

// check that there's an invalid page beneath
// the user stack, to catch stack overflow.
void stacktest(char* s) {
  int pid;
  int xstatus;

  pid = _fork();
  if (pid == 0) {
    char* sp = (char*)r_sp();
    sp -= USERSTACK * PGSIZE;
    // the *sp should cause a trap.
    printf("%s: stacktest: read below stack %d\n", s, *sp);
    _exit(1);
  } else if (pid < 0) {
    printf("%s: fork failed\n", s);
    _exit(1);
  }
  _wait(&xstatus);
  if (xstatus == -1) { // kernel killed child?
    _exit(0);
  } else {
    _exit(xstatus);
  }
}

// check that writes to a few forbidden addresses
// cause a fault, e.g. process's text and TRAMPOLINE.
void nowrite(char* s) {
  int    pid;
  int    xstatus;
  uint64 addrs[] = {0,
                    0x80000000LL,
                    0x3fffffe000,
                    0x3ffffff000,
                    0x4000000000,
                    0xffffffffffffffff};

  for (int ai = 0; ai < sizeof(addrs) / sizeof(addrs[0]); ai++) {
    pid = _fork();
    if (pid == 0) {
      volatile int* addr = (int*)addrs[ai];
      *addr              = 10;
      printf("%s: write to %p did not fail!\n", s, (void*)addr);
      _exit(0);
    } else if (pid < 0) {
      printf("%s: fork failed\n", s);
      _exit(1);
    }
    _wait(&xstatus);
    if (xstatus == 0) {
      // kernel did not kill child!
      _exit(1);
    }
  }
  _exit(0);
}

// regression test. copyin(), copyout(), and copyinstr() used to cast
// the virtual page address to uint, which (with certain wild system
// call arguments) resulted in a kernel page faults.
void* big = (void*)0xeaeb0b5b00002f5e;
void  pgbug(char* s) {
  char* argv[1];
  argv[0] = 0;
  _exec(big, argv);
  _pipe(big);

  _exit(0);
}

// regression test. does the kernel panic if a process _sbrk()s its
// size to be less than a page, or zero, or reduces the break by an
// amount too small to cause a page to be freed?
void sbrkbugs(char* s) {
  int pid = _fork();
  if (pid < 0) {
    printf("fork failed\n");
    _exit(1);
  }
  if (pid == 0) {
    int sz = (uint64)_sbrk(0);
    // free all user memory; there used to be a bug that
    // would not adjust p->sz correctly in this case,
    // causing _exit() to panic.
    _sbrk(-sz);
    // user page fault here.
    _exit(0);
  }
  _wait(0);

  pid = _fork();
  if (pid < 0) {
    printf("fork failed\n");
    _exit(1);
  }
  if (pid == 0) {
    int sz = (uint64)_sbrk(0);
    // set the break to somewhere in the very first
    // page; there used to be a bug that would incorrectly
    // free the first page.
    _sbrk(-(sz - 3500));
    _exit(0);
  }
  _wait(0);

  pid = _fork();
  if (pid < 0) {
    printf("fork failed\n");
    _exit(1);
  }
  if (pid == 0) {
    // set the break in the middle of a page.
    _sbrk((10 * 4096 + 2048) - (uint64)_sbrk(0));

    // reduce the break a bit, but not enough to
    // cause a page to be freed. this used to cause
    // a panic.
    _sbrk(-10);

    _exit(0);
  }
  _wait(0);

  _exit(0);
}

// if process size was somewhat more than a page boundary, and then
// shrunk to be somewhat less than that page boundary, can the kernel
// still copyin() from addresses in the last page?
void sbrklast(char* s) {
  uint64 top = (uint64)_sbrk(0);
  if ((top % 4096) != 0) {
    _sbrk(4096 - (top % 4096));
  }
  _sbrk(4096);
  _sbrk(10);
  _sbrk(-20);
  top     = (uint64)_sbrk(0);
  char* p = (char*)(top - 64);
  p[0]    = 'x';
  p[1]    = '\0';
  int fd  = _open(p, O_RDWR | O_CREATE);
  _write(fd, p, 1);
  _close(fd);
  fd   = _open(p, O_RDWR);
  p[0] = '\0';
  _read(fd, p, 1);
  if (p[0] != 'x') {
    _exit(1);
  }
}

// does sbrk handle signed int32 wrap-around with
// negative arguments?
void sbrk8000(char* s) {
  _sbrk(0x80000004);
  volatile char* top = _sbrk(0);
  *(top - 1)         = *(top - 1) + 1;
}

// regression test. test whether _exec() leaks memory if one of the
// arguments is invalid. the test passes if the kernel doesn't panic.
void badarg(char* s) {
  for (int i = 0; i < 50000; i++) {
    char* argv[2];
    argv[0] = (char*)0xffffffff;
    argv[1] = 0;
    _exec("echo", argv);
  }

  _exit(0);
}

struct test {
  void (*f)(char*);
  char* s;
} quicktests[] = {
    {copyin, "copyin"},
    {copyout, "copyout"},
    {copyinstr1, "copyinstr1"},
    {copyinstr2, "copyinstr2"},
    {copyinstr3, "copyinstr3"},
    {rwsbrk, "rwsbrk"},
    {truncate1, "truncate1"},
    {truncate2, "truncate2"},
    {truncate3, "truncate3"},
    {openiputtest, "openiput"},
    {exitiputtest, "exitiput"},
    {iputtest, "iput"},
    {opentest, "opentest"},
    {writetest, "writetest"},
    {writebig, "writebig"},
    {createtest, "createtest"},
    {dirtest, "dirtest"},
    {exectest, "exectest"},
    {pipe1, "pipe1"},
    {killstatus, "killstatus"},
    {preempt, "preempt"},
    {exitwait, "exitwait"},
    {reparent, "reparent"},
    {twochildren, "twochildren"},
    {forkfork, "forkfork"},
    {forkforkfork, "forkforkfork"},
    {reparent2, "reparent2"},
    {mem, "mem"},
    {sharedfd, "sharedfd"},
    {fourfiles, "fourfiles"},
    {createdelete, "createdelete"},
    {unlinkread, "unlinkread"},
    {linktest, "linktest"},
    {concreate, "concreate"},
    {linkunlink, "linkunlink"},
    {subdir, "subdir"},
    {bigwrite, "bigwrite"},
    {bigfile, "bigfile"},
    {fourteen, "fourteen"},
    {rmdot, "rmdot"},
    {dirfile, "dirfile"},
    {iref, "iref"},
    {forktest, "forktest"},
    {sbrkbasic, "sbrkbasic"},
    {sbrkmuch, "sbrkmuch"},
    {kernmem, "kernmem"},
    {MAXVAplus, "MAXVAplus"},
    {sbrkfail, "sbrkfail"},
    {sbrkarg, "sbrkarg"},
    {validatetest, "validatetest"},
    {bsstest, "bsstest"},
    {bigargtest, "bigargtest"},
    {argptest, "argptest"},
    {stacktest, "stacktest"},
    {nowrite, "nowrite"},
    {pgbug, "pgbug"},
    {sbrkbugs, "sbrkbugs"},
    {sbrklast, "sbrklast"},
    {sbrk8000, "sbrk8000"},
    {badarg, "badarg"},

    {0, 0},
};

//
// Section with tests that take a fair bit of time
//

// directory that uses indirect blocks
void bigdir(char* s) {
  enum { N = 500 };
  int  i, fd;
  char name[10];

  _unlink("bd");

  fd = _open("bd", O_CREATE);
  if (fd < 0) {
    printf("%s: bigdir create failed\n", s);
    _exit(1);
  }
  _close(fd);

  for (i = 0; i < N; i++) {
    name[0] = 'x';
    name[1] = '0' + (i / 64);
    name[2] = '0' + (i % 64);
    name[3] = '\0';
    if (_link("bd", name) != 0) {
      printf("%s: bigdir i=%d _link(bd, %s) failed\n", s, i, name);
      _exit(1);
    }
  }

  _unlink("bd");
  for (i = 0; i < N; i++) {
    name[0] = 'x';
    name[1] = '0' + (i / 64);
    name[2] = '0' + (i % 64);
    name[3] = '\0';
    if (_unlink(name) != 0) {
      printf("%s: bigdir unlink failed", s);
      _exit(1);
    }
  }
}

// concurrent writes to try to provoke deadlock in the virtio disk
// driver.
void manywrites(char* s) {
  int nchildren = 4;
  int howmany   = 30; // increase to look for deadlock

  for (int ci = 0; ci < nchildren; ci++) {
    int pid = _fork();
    if (pid < 0) {
      printf("fork failed\n");
      _exit(1);
    }

    if (pid == 0) {
      char name[3];
      name[0] = 'b';
      name[1] = 'a' + ci;
      name[2] = '\0';
      _unlink(name);

      for (int iters = 0; iters < howmany; iters++) {
        for (int i = 0; i < ci + 1; i++) {
          int fd = _open(name, O_CREATE | O_RDWR);
          if (fd < 0) {
            printf("%s: cannot create %s\n", s, name);
            _exit(1);
          }
          int sz = sizeof(buf);
          int cc = _write(fd, buf, sz);
          if (cc != sz) {
            printf("%s: _write(%d) ret %d\n", s, sz, cc);
            _exit(1);
          }
          _close(fd);
        }
        _unlink(name);
      }

      _unlink(name);
      _exit(0);
    }
  }

  for (int ci = 0; ci < nchildren; ci++) {
    int st = 0;
    _wait(&st);
    if (st != 0) {
      _exit(st);
    }
  }
  _exit(0);
}

// regression test. does _write() with an invalid buffer pointer cause
// a block to be allocated for a file that is then not freed when the
// file is deleted? if the kernel has this bug, it will panic: balloc:
// out of blocks. assumed_free may need to be raised to be more than
// the number of free blocks. this test takes a long time.
void badwrite(char* s) {
  int assumed_free = 600;

  _unlink("junk");
  for (int i = 0; i < assumed_free; i++) {
    int fd = _open("junk", O_CREATE | O_WRONLY);
    if (fd < 0) {
      printf("open junk failed\n");
      _exit(1);
    }
    _write(fd, (char*)0xffffffffffL, 1);
    _close(fd);
    _unlink("junk");
  }

  int fd = _open("junk", O_CREATE | O_WRONLY);
  if (fd < 0) {
    printf("open junk failed\n");
    _exit(1);
  }
  if (_write(fd, "x", 1) != 1) {
    printf("write failed\n");
    _exit(1);
  }
  _close(fd);
  _unlink("junk");

  _exit(0);
}

// test the _exec() code that cleans up if it runs out
// of memory. it's really a test that such a condition
// doesn't cause a panic.
void execout(char* s) {
  for (int avail = 0; avail < 15; avail++) {
    int pid = _fork();
    if (pid < 0) {
      printf("fork failed\n");
      _exit(1);
    } else if (pid == 0) {
      // allocate all of memory.
      while (1) {
        uint64 a = (uint64)_sbrk(4096);
        if (a == 0xffffffffffffffffLL) {
          break;
        }
        *(char*)(a + 4096 - 1) = 1;
      }

      // free a few pages, in order to let _exec() make some
      // progress.
      for (int i = 0; i < avail; i++) {
        _sbrk(-4096);
      }

      _close(1);
      char* args[] = {"echo", "x", 0};
      _exec("echo", args);
      _exit(0);
    } else {
      _wait((int*)0);
    }
  }

  _exit(0);
}

// can the kernel tolerate running out of disk space?
void diskfull(char* s) {
  int fi;
  int done = 0;

  _unlink("diskfulldir");

  for (fi = 0; done == 0 && '0' + fi < 0177; fi++) {
    char name[32];
    name[0] = 'b';
    name[1] = 'i';
    name[2] = 'g';
    name[3] = '0' + fi;
    name[4] = '\0';
    _unlink(name);
    int fd = _open(name, O_CREATE | O_RDWR | O_TRUNC);
    if (fd < 0) {
      // oops, ran out of inodes before running out of blocks.
      printf("%s: could not create file %s\n", s, name);
      done = 1;
      break;
    }
    for (int i = 0; i < MAXFILE; i++) {
      char buf[BSIZE];
      if (_write(fd, buf, BSIZE) != BSIZE) {
        done = 1;
        _close(fd);
        break;
      }
    }
    _close(fd);
  }

  // now that there are no free blocks, test that dirlink()
  // merely fails (doesn't panic) if it can't extend
  // directory content. one of these file creations
  // is expected to fail.
  int nzz = 128;
  for (int i = 0; i < nzz; i++) {
    char name[32];
    name[0] = 'z';
    name[1] = 'z';
    name[2] = '0' + (i / 32);
    name[3] = '0' + (i % 32);
    name[4] = '\0';
    _unlink(name);
    int fd = _open(name, O_CREATE | O_RDWR | O_TRUNC);
    if (fd < 0) {
      break;
    }
    _close(fd);
  }

  // this _mkdir() is expected to fail.
  if (_mkdir("diskfulldir") == 0) {
    printf("%s: _mkdir(diskfulldir) unexpectedly succeeded!\n", s);
  }

  _unlink("diskfulldir");

  for (int i = 0; i < nzz; i++) {
    char name[32];
    name[0] = 'z';
    name[1] = 'z';
    name[2] = '0' + (i / 32);
    name[3] = '0' + (i % 32);
    name[4] = '\0';
    _unlink(name);
  }

  for (int i = 0; '0' + i < 0177; i++) {
    char name[32];
    name[0] = 'b';
    name[1] = 'i';
    name[2] = 'g';
    name[3] = '0' + i;
    name[4] = '\0';
    _unlink(name);
  }
}

void outofinodes(char* s) {
  int nzz = 32 * 32;
  for (int i = 0; i < nzz; i++) {
    char name[32];
    name[0] = 'z';
    name[1] = 'z';
    name[2] = '0' + (i / 32);
    name[3] = '0' + (i % 32);
    name[4] = '\0';
    _unlink(name);
    int fd = _open(name, O_CREATE | O_RDWR | O_TRUNC);
    if (fd < 0) {
      // failure is eventually expected.
      break;
    }
    _close(fd);
  }

  for (int i = 0; i < nzz; i++) {
    char name[32];
    name[0] = 'z';
    name[1] = 'z';
    name[2] = '0' + (i / 32);
    name[3] = '0' + (i % 32);
    name[4] = '\0';
    _unlink(name);
  }
}

struct test slowtests[] = {
    {bigdir, "bigdir"},
    {manywrites, "manywrites"},
    {badwrite, "badwrite"},
    {execout, "execout"},
    {diskfull, "diskfull"},
    {outofinodes, "outofinodes"},

    {0, 0},
};

//
// drive tests
//

// run each test in its own process. run returns 1 if child's _exit()
// indicates success.
int run(void f(char*), char* s) {
  int pid;
  int xstatus;

  printf("test %s: ", s);
  if ((pid = _fork()) < 0) {
    printf("runtest: fork error\n");
    _exit(1);
  }
  if (pid == 0) {
    f(s);
    _exit(0);
  } else {
    _wait(&xstatus);
    if (xstatus != 0) {
      printf("FAILED\n");
    } else {
      printf("OK\n");
    }
    return xstatus == 0;
  }
}

int runtests(struct test* tests, char* justone, int continuous) {
  for (struct test* t = tests; t->s != 0; t++) {
    if ((justone == 0) || strcmp(t->s, justone) == 0) {
      if (!run(t->f, t->s)) {
        if (continuous != 2) {
          printf("SOME TESTS FAILED\n");
          return 1;
        }
      }
    }
  }
  return 0;
}

//
// use _sbrk() to count how many free physical memory pages there are.
// touches the pages to force allocation.
// because out of memory with lazy allocation results in the process
// taking a fault and being killed, fork and report back.
//
int countfree() {
  int fds[2];

  if (_pipe(fds) < 0) {
    printf("_pipe() failed in countfree()\n");
    _exit(1);
  }

  int pid = _fork();

  if (pid < 0) {
    printf("fork failed in countfree()\n");
    _exit(1);
  }

  if (pid == 0) {
    _close(fds[0]);

    while (1) {
      uint64 a = (uint64)_sbrk(4096);
      if (a == 0xffffffffffffffff) {
        break;
      }

      // modify the memory to make sure it's really allocated.
      *(char*)(a + 4096 - 1) = 1;

      // report back one more page.
      if (_write(fds[1], "x", 1) != 1) {
        printf("_write() failed in countfree()\n");
        _exit(1);
      }
    }

    _exit(0);
  }

  _close(fds[1]);

  int n = 0;
  while (1) {
    char c;
    int  cc = _read(fds[0], &c, 1);
    if (cc < 0) {
      printf("_read() failed in countfree()\n");
      _exit(1);
    }
    if (cc == 0) {
      break;
    }
    n += 1;
  }

  _close(fds[0]);
  _wait((int*)0);

  return n;
}

int drivetests(int quick, int continuous, char* justone) {
  do {
    printf("usertests starting\n");
    int free0 = countfree();
    int free1 = 0;
    if (runtests(quicktests, justone, continuous)) {
      if (continuous != 2) {
        return 1;
      }
    }
    if (!quick) {
      if (justone == 0) {
        printf("usertests slow tests starting\n");
      }
      if (runtests(slowtests, justone, continuous)) {
        if (continuous != 2) {
          return 1;
        }
      }
    }
    if ((free1 = countfree()) < free0) {
      printf("FAILED -- lost some free pages %d (out of %d)\n", free1, free0);
      if (continuous != 2) {
        return 1;
      }
    }
  } while (continuous);
  return 0;
}

int main(int argc, char* argv[]) {
  int   continuous = 0;
  int   quick      = 0;
  char* justone    = 0;

  if (argc == 2 && strcmp(argv[1], "-q") == 0) {
    quick = 1;
  } else if (argc == 2 && strcmp(argv[1], "-c") == 0) {
    continuous = 1;
  } else if (argc == 2 && strcmp(argv[1], "-C") == 0) {
    continuous = 2;
  } else if (argc == 2 && argv[1][0] != '-') {
    justone = argv[1];
  } else if (argc > 1) {
    printf("Usage: usertests [-c] [-C] [-q] [testname]\n");
    _exit(1);
  }
  if (drivetests(quick, continuous, justone)) {
    _exit(1);
  }
  printf("ALL TESTS PASSED\n");
  _exit(0);
}
