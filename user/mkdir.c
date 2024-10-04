#include "user/user.h"

int main(int argc, char* argv[]) {
  int i;

  if (argc < 2) {
    fdprintf(stderr, "Usage: mkdir files...\n");
    _exit(1);
  }

  for (i = 1; i < argc; i++) {
    if (_mkdir(argv[i]) < 0) {
      fdprintf(stderr, "mkdir: %s failed to create\n", argv[i]);
      break;
    }
  }

  _exit(0);
}
