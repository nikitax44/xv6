#include "kernel/types.h"
#include "user/user.h"

int main(int argc, char* argv[]) {
  int i;

  if (argc < 2) {
    fdprintf(stderr, "Usage: rm files...\n");
    _exit(1);
  }

  for (i = 1; i < argc; i++) {
    if (_unlink(argv[i]) < 0) {
      fdprintf(stderr, "rm: %s failed to delete\n", argv[i]);
      break;
    }
  }

  _exit(0);
}
