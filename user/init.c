// init: The initial user-level program

#include "kernel/fcntl.h"
#include "kernel/file.h"
#include "kernel/fs.h"
#include "kernel/sleeplock.h"
#include "kernel/spinlock.h"
#include "kernel/stat.h"
#include "kernel/types.h"
#include "user/user.h"

char* argv[] = {"sh", 0};

int main(void) {
  int pid, wpid;

  if (_open("console", O_RDWR) < 0) {
    _mknod("console", CONSOLE, 0);
    _open("console", O_RDWR);
  }
  _dup(0); // stdout
  _dup(0); // stderr

  for (;;) {
    printf("init: starting sh\n");
    pid = _fork();
    if (pid < 0) {
      printf("init: fork failed\n");
      _exit(1);
    }
    if (pid == 0) {
      _exec("sh", argv);
      printf("init: exec sh failed\n");
      _exit(1);
    }

    for (;;) {
      // this call to _wait() returns if the shell exits,
      // or if a parentless process exits.
      wpid = _wait((int*)0);
      if (wpid == pid) {
        // the shell exited; restart it.
        break;
      } else if (wpid < 0) {
        printf("init: wait returned an error\n");
        _exit(1);
      } else {
        // it was a parentless process; do nothing.
      }
    }
  }
}
