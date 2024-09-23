// Test that fork fails gracefully.
// Tiny executable so that the limit can be filling the proc table.

#include "kernel/stat.h"
#include "kernel/types.h"
#include "user/user.h"

#define N 1000

void print(const char* s) { _write(1, s, strlen(s)); }

void forktest(void) {
  int n, pid;

  print("fork test\n");

  for (n = 0; n < N; n++) {
    pid = _fork();
    if (pid < 0)
      break;
    if (pid == 0)
      _exit(0);
  }

  if (n == N) {
    print("fork claimed to work N times!\n");
    _exit(1);
  }

  for (; n > 0; n--) {
    if (_wait(0) < 0) {
      print("wait stopped early\n");
      _exit(1);
    }
  }

  if (_wait(0) != -1) {
    print("wait got too many\n");
    _exit(1);
  }

  print("fork test OK\n");
}

int main(void) {
  forktest();
  _exit(0);
}
