// Demonstrate that moving the "acquire" in iderw after the loop that
// appends to the idequeue results in a race.

// For this to work, you should also add a spin within iderw's
// idequeue traversal loop.  Adding the following demonstrated a panic
// after about 5 runs of stressfs in QEMU on a 2.1GHz CPU:
//    for (i = 0; i < 40000; i++)
//      asm volatile("");

#include "kernel/fcntl.h"
#include "kernel/fs.h"
#include "kernel/stat.h"
#include "kernel/types.h"
#include "user/user.h"

int main() {
  int  fd, i;
  char path[] = "stressfs0";
  char data[512];

  printf("stressfs starting\n");
  memset(data, 'a', sizeof(data));

  for (i = 0; i < 4; i++)
    if (_fork() > 0)
      break;

  printf("write %d\n", i);

  path[8] += i;
  fd = _open(path, O_CREATE | O_RDWR);
  for (i = 0; i < 20; i++)
    //    printf(fd, "%d\n", i);
    _write(fd, data, sizeof(data));
  _close(fd);

  printf("read\n");

  fd = _open(path, O_RDONLY);
  for (i = 0; i < 20; i++)
    _read(fd, data, sizeof(data));
  _close(fd);

  _wait(0);

  _exit(0);
}
