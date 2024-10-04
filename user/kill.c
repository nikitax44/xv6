#include "user/user.h"

int main(int argc, char** argv) {
  int i;

  if (argc < 2) {
    fdprintf(stderr, "usage: kill pid...\n");
    _exit(1);
  }
  for (i = 1; i < argc; i++) {
    _kill(atoi(argv[i]));
  }
  _exit(0);
}
