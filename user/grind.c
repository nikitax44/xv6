//
// run random system calls in parallel forever.
//

#include "kernel/fcntl.h"
#include "kernel/fs.h"
#include "kernel/memlayout.h"
#include "kernel/param.h"
#include "kernel/riscv.h"
#include "kernel/stat.h"
#include "kernel/syscall.h"
#include "kernel/types.h"
#include "user/user.h"

// from FreeBSD.
int do_rand(unsigned long* ctx) {
  /*
   * Compute x = (7^5 * x) mod (2^31 - 1)
   * without overflowing 31 bits:
   *      (2^31 - 1) = 127773 * (7^5) + 2836
   * From "Random number generators: good ones are hard to find",
   * Park and Miller, Communications of the ACM, vol. 31, no. 10,
   * October 1988, p. 1195.
   */
  long hi, lo, x;

  /* Transform to [1, 0x7ffffffe] range. */
  x  = (*ctx % 0x7ffffffe) + 1;
  hi = x / 127773;
  lo = x % 127773;
  x  = 16807 * lo - 2836 * hi;
  if (x < 0) {
    x += 0x7fffffff;
  }
  /* Transform to [0, 0x7ffffffd] range. */
  x--;
  *ctx = x;
  return (x);
}

unsigned long rand_next = 1;

int rand(void) { return (do_rand(&rand_next)); }

void go(int which_child) {
  int         fd = -1;
  static char buf[999];
  char*       break0 = _sbrk(0);
  uint64      iters  = 0;

  _mkdir("grindir");
  if (_chdir("grindir") != 0) {
    printf("grind: chdir grindir failed\n");
    _exit(1);
  }
  _chdir("/");

  while (1) {
    iters++;
    if ((iters % 500) == 0) {
      _write(1, which_child ? "B" : "A", 1);
    }
    int what = rand() % 23;
    if (what == 1) {
      _close(_open("grindir/../a", O_CREATE | O_RDWR));
    } else if (what == 2) {
      _close(_open("grindir/../grindir/../b", O_CREATE | O_RDWR));
    } else if (what == 3) {
      _unlink("grindir/../a");
    } else if (what == 4) {
      if (_chdir("grindir") != 0) {
        printf("grind: chdir grindir failed\n");
        _exit(1);
      }
      _unlink("../b");
      _chdir("/");
    } else if (what == 5) {
      _close(fd);
      fd = _open("/grindir/../a", O_CREATE | O_RDWR);
    } else if (what == 6) {
      _close(fd);
      fd = _open("/./grindir/./../b", O_CREATE | O_RDWR);
    } else if (what == 7) {
      _write(fd, buf, sizeof(buf));
    } else if (what == 8) {
      _read(fd, buf, sizeof(buf));
    } else if (what == 9) {
      _mkdir("grindir/../a");
      _close(_open("a/../a/./a", O_CREATE | O_RDWR));
      _unlink("a/a");
    } else if (what == 10) {
      _mkdir("/../b");
      _close(_open("grindir/../b/b", O_CREATE | O_RDWR));
      _unlink("b/b");
    } else if (what == 11) {
      _unlink("b");
      _link("../grindir/./../a", "../b");
    } else if (what == 12) {
      _unlink("../grindir/../a");
      _link(".././b", "/grindir/../a");
    } else if (what == 13) {
      int pid = _fork();
      if (pid == 0) {
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      _wait(0);
    } else if (what == 14) {
      int pid = _fork();
      if (pid == 0) {
        _fork();
        _fork();
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      _wait(0);
    } else if (what == 15) {
      _sbrk(6011);
    } else if (what == 16) {
      if (_sbrk(0) > break0) {
        _sbrk(-(_sbrk(0) - break0));
      }
    } else if (what == 17) {
      int pid = _fork();
      if (pid == 0) {
        _close(_open("a", O_CREATE | O_RDWR));
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      if (_chdir("../grindir/..") != 0) {
        printf("grind: chdir failed\n");
        _exit(1);
      }
      _kill(pid);
      _wait(0);
    } else if (what == 18) {
      int pid = _fork();
      if (pid == 0) {
        _kill(_getpid());
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      _wait(0);
    } else if (what == 19) {
      int fds[2];
      if (_pipe(fds) < 0) {
        printf("grind: pipe failed\n");
        _exit(1);
      }
      int pid = _fork();
      if (pid == 0) {
        _fork();
        _fork();
        if (_write(fds[1], "x", 1) != 1) {
          printf("grind: pipe write failed\n");
        }
        char c;
        if (_read(fds[0], &c, 1) != 1) {
          printf("grind: pipe read failed\n");
        }
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      _close(fds[0]);
      _close(fds[1]);
      _wait(0);
    } else if (what == 20) {
      int pid = _fork();
      if (pid == 0) {
        _unlink("a");
        _mkdir("a");
        _chdir("a");
        _unlink("../a");
        fd = _open("x", O_CREATE | O_RDWR);
        _unlink("x");
        _exit(0);
      } else if (pid < 0) {
        printf("grind: fork failed\n");
        _exit(1);
      }
      _wait(0);
    } else if (what == 21) {
      _unlink("c");
      // should always succeed. check that there are free i-nodes,
      // file descriptors, blocks.
      int fd1 = _open("c", O_CREATE | O_RDWR);
      if (fd1 < 0) {
        printf("grind: create c failed\n");
        _exit(1);
      }
      if (_write(fd1, "x", 1) != 1) {
        printf("grind: write c failed\n");
        _exit(1);
      }
      struct stat st;
      if (_fstat(fd1, &st) != 0) {
        printf("grind: fstat failed\n");
        _exit(1);
      }
      if (st.size != 1) {
        printf("grind: fstat reports wrong size %d\n", (int)st.size);
        _exit(1);
      }
      if (st.ino > 200) {
        printf("grind: fstat reports crazy i-number %d\n", st.ino);
        _exit(1);
      }
      _close(fd1);
      _unlink("c");
    } else if (what == 22) {
      // echo hi | cat
      int aa[2], bb[2];
      if (_pipe(aa) < 0) {
        fdprintf(stderr, "grind: pipe failed\n");
        _exit(1);
      }
      if (_pipe(bb) < 0) {
        fdprintf(stderr, "grind: pipe failed\n");
        _exit(1);
      }
      int pid1 = _fork();
      if (pid1 == 0) {
        _close(bb[0]);
        _close(bb[1]);
        _close(aa[0]);
        _close(1);
        if (_dup(aa[1]) != 1) {
          fdprintf(stderr, "grind: dup failed\n");
          _exit(1);
        }
        _close(aa[1]);
        char* args[3] = {"echo", "hi", 0};
        _exec("grindir/../echo", args);
        fdprintf(stderr, "grind: echo: not found\n");
        _exit(2);
      } else if (pid1 < 0) {
        fdprintf(stderr, "grind: fork failed\n");
        _exit(3);
      }
      int pid2 = _fork();
      if (pid2 == 0) {
        _close(aa[1]);
        _close(bb[0]);
        _close(0);
        if (_dup(aa[0]) != 0) {
          fdprintf(stderr, "grind: dup failed\n");
          _exit(4);
        }
        _close(aa[0]);
        _close(1);
        if (_dup(bb[1]) != 1) {
          fdprintf(stderr, "grind: dup failed\n");
          _exit(5);
        }
        _close(bb[1]);
        char* args[2] = {"cat", 0};
        _exec("/cat", args);
        fdprintf(stderr, "grind: cat: not found\n");
        _exit(6);
      } else if (pid2 < 0) {
        fdprintf(stderr, "grind: fork failed\n");
        _exit(7);
      }
      _close(aa[0]);
      _close(aa[1]);
      _close(bb[1]);
      char buf[4] = {0, 0, 0, 0};
      _read(bb[0], buf + 0, 1);
      _read(bb[0], buf + 1, 1);
      _read(bb[0], buf + 2, 1);
      _close(bb[0]);
      int st1, st2;
      _wait(&st1);
      _wait(&st2);
      if (st1 != 0 || st2 != 0 || strcmp(buf, "hi\n") != 0) {
        printf("grind: exec pipeline failed %d %d \"%s\"\n", st1, st2, buf);
        _exit(1);
      }
    }
  }
}

void iter() {
  _unlink("a");
  _unlink("b");

  int pid1 = _fork();
  if (pid1 < 0) {
    printf("grind: fork failed\n");
    _exit(1);
  }
  if (pid1 == 0) {
    rand_next ^= 31;
    go(0);
    _exit(0);
  }

  int pid2 = _fork();
  if (pid2 < 0) {
    printf("grind: fork failed\n");
    _exit(1);
  }
  if (pid2 == 0) {
    rand_next ^= 7177;
    go(1);
    _exit(0);
  }

  int st1 = -1;
  _wait(&st1);
  if (st1 != 0) {
    _kill(pid1);
    _kill(pid2);
  }
  int st2 = -1;
  _wait(&st2);

  _exit(0);
}

int main() {
  while (1) {
    int pid = _fork();
    if (pid == 0) {
      iter();
      _exit(0);
    }
    if (pid > 0) {
      _wait(0);
    }
    _sleep(20);
    rand_next += 1;
  }
}
