#include "kernel/stat.h"
#include "kernel/types.h"
#include "user/user.h"

int main(int argc, char* argv[]) {
  if (argc != 3) {
    fprintf(2, "Usage: ln old new\n");
    _exit(1);
  }
  if (_link(argv[1], argv[2]) < 0)
    fprintf(2, "link %s %s: failed\n", argv[1], argv[2]);
  _exit(0);
}
